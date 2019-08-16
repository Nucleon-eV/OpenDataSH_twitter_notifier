use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use futures_core::{Future, Poll};
use futures_core::task::Context;
use futures_locks::Mutex;
use futures_util::compat::Future01CompatExt;
use futures_util::future::FutureExt;
use futures_util::TryStreamExt;
use hyper::{Body, Client, Response};
use hyper::client::{HttpConnector, ResponseFuture};
use hyper_tls::HttpsConnector;
use serde::de;

use crate::twitter::Twitter;

type HttpsClient = Client<HttpsConnector<HttpConnector>, hyper::Body>;

#[derive(Serialize, Deserialize)]
pub struct PackageListResult {
    pub help: String,
    pub success: bool,
    pub result: Vec<String>,
}

impl PackageListResult {
    async fn deserialize(res: Response<Body>) -> serde_json::Result<Self>
    where
        for<'de> Self: de::Deserialize<'de>,
    {
        // Error handling
        let body = res.into_body().try_concat().await.unwrap();
        let result = serde_json::from_slice(&body)?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct CkanAPI {
    /// hyper http client to build requests with.
    http: HttpsClient,
    twitter: Arc<Mutex<Twitter>>,
}

impl CkanAPI {
    pub fn new(twitter: Arc<Mutex<Twitter>>) -> Self {
        let http = {
            let connector = HttpsConnector::new(4).unwrap();

            Client::builder().build(connector)
        };

        CkanAPI { http, twitter }
    }

    pub fn getPackageList(&self) -> impl Future<Output = ()> + Send {
        debug!("1");
        GetPackageListFuture::new(self.http.clone(), self.twitter.clone())
    }
}

pub struct GetPackageListFuture {
    response: ResponseFuture,
    twitter: Arc<Mutex<Twitter>>,
}

impl GetPackageListFuture {
    fn new(http: HttpsClient, twitter: Arc<Mutex<Twitter>>) -> Self {
        let uri = "https://opendata.schleswig-holstein.de/api/3/action/package_list"
            .parse()
            .unwrap();

        debug!("2");
        Self {
            response: http.get(uri),
            twitter,
        }
    }
}

impl Future for GetPackageListFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        debug!("3");
        match self.response.poll_unpin(cx) {
            Poll::Ready(res) => match PackageListResult::deserialize(res.unwrap())
                .boxed()
                .poll_unpin(cx)
            {
                Poll::Ready(Ok(data)) => {
                    debug!("4");
                    let mut added_datasets: HashSet<String> = HashSet::new();
                    let mut removed_datasets: HashSet<String> = HashSet::new();
                    if !Path::new("./data/").exists() {
                        fs::create_dir_all("./data/");
                    }
                    if Path::new("./data/latestPackageList.json").exists() {
                        let cache_file: String =
                            fs::read_to_string("./data/latestPackageList.json").unwrap();
                        let cache: HashSet<String> =
                            serde_json::from_str::<Vec<String>>(cache_file.as_str())
                                .unwrap()
                                .iter()
                                .cloned()
                                .collect();
                        let newdata: HashSet<String> = data.result.iter().cloned().collect();

                        removed_datasets = cache.difference(&newdata).cloned().collect();
                        added_datasets = newdata.difference(&cache).cloned().collect();
                    }
                    let serialized = serde_json::to_string(&data.result).unwrap();
                    fs::write("./data/latestPackageList.json", serialized)
                        .expect("Unable to write latestPackageList");

                    debug!("5");
                    match self.twitter.lock().compat().boxed().poll_unpin(cx) {
                        Poll::Ready(Ok(twitter)) => {
                            match twitter
                                .post_changed_datasets(added_datasets, removed_datasets)
                                .boxed()
                                .poll_unpin(cx)
                            {
                                Poll::Ready(_) => Poll::Ready(()),
                                Poll::Pending => Poll::Pending,
                            }
                        }
                        Poll::Pending => Poll::Pending,
                        Poll::Ready(Err(_)) => Poll::Ready(()),
                    }
                }
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(_)) => Poll::Ready(()),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
