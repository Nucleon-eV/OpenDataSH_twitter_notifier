use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use futures::{Async, Poll};
use futures::future::Future;
use futures::stream::Stream;
use hyper::client::{HttpConnector, ResponseFuture};
use hyper::Client;
use hyper_tls::HttpsConnector;

use crate::twitter::Twitter;

type HttpsClient = Client<HttpsConnector<HttpConnector>, hyper::Body>;

#[derive(Serialize, Deserialize)]
pub struct PackageListResult {
    pub help: String,
    pub success: bool,
    pub result: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CkanAPI {
    /// hyper http client to build requests with.
    http: HttpsClient,
}

impl CkanAPI {
    pub fn new() -> Self {
        let http = {
            let connector = HttpsConnector::new(4).unwrap();

            Client::builder().build(connector)
        };

        CkanAPI { http }
    }

    pub fn getPackageList(&self) -> ResponseFuture {
        let uri = "https://opendata.schleswig-holstein.de/api/3/action/package_list"
            .parse()
            .unwrap();

        self.http.get(uri)
    }
}

pub struct GetPackageList {
    pub response: ResponseFuture,
    pub twitter: Arc<Mutex<Twitter>>,
}

impl Future for GetPackageList {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.response.poll() {
            Ok(Async::Ready(res)) => match res.into_body().poll() {
                Ok(Async::Ready(body)) => {
                    let data: PackageListResult = serde_json::from_slice(&body.unwrap()).unwrap();
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

                    self.twitter
                        .lock()
                        .unwrap()
                        .post_changed_datasets(added_datasets, removed_datasets);
                    Ok(Async::Ready(()))
                }
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Err(e) => {
                    println!("failed to get body: {}", e);
                    Ok(Async::Ready(()))
                }
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => {
                println!("failed to get response: {}", e);
                Ok(Async::Ready(()))
            }
        }
    }
}
