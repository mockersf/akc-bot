use hyper::Client;
use hyper::Url;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::Error;
use hyper::header::Headers;
use hyper::client::Response;
use futures::Future;
use std;

use REQUEST_CPU_POOL;

pub fn get_async(url: Url,
                 headers: Headers)
                 -> Box<Future<Item = Response, Error = Error> + std::marker::Send> {
    REQUEST_CPU_POOL
        .spawn_fn(|| {
                      let ssl = NativeTlsClient::new().unwrap();
                      let connector = HttpsConnector::new(ssl);
                      let client = Client::with_connector(connector);
                      client.get(url).headers(headers).send()
                  })
        .boxed()
}
