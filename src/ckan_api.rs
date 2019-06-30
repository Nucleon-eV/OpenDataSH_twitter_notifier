use futures::future::Future;
use futures::stream::Stream;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

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

    pub fn getPackageList(
        &self,
    ) -> Box<dyn Future<Item = PackageListResult, Error = hyper::Error> + Send> {
        let uri = "https://opendata.schleswig-holstein.de/api/3/action/package_list"
            .parse()
            .unwrap();

        let f = self.http.get(uri).and_then(|res| {
            res.into_body().concat2().and_then(move |body| {
                let value: PackageListResult = serde_json::from_slice(&body).unwrap();
                Ok(value)
            })
        });
        Box::new(f)
    }
}
