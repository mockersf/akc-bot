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

macro_rules! read_body {
    ( $future:expr, $type:ident, $error_message:expr ) => {
        $future
        .map(move |mut response| -> Result<$type, serde_json::Error> {
                 let mut s = String::new();
                 response.read_to_string(&mut s).unwrap();
                 let parse = serde_json::from_str(&s);
                 parse.or_else(|err| {
                     warn!("error parsing response: {:?}", err);
                     Err(err)
                 })
             })
        .map_err(|e| Error { msg: format!($error_message, e) })
    }
}
