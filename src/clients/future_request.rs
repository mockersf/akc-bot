use hyper::Client;
use hyper::Url;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper;
use hyper::header::Headers;
use hyper::client::Response;
use futures::Future;
use std;

use REQUEST_CPU_POOL;

pub fn get_async<Error: 'static>
    (url: Url,
     headers: Headers)
     -> Box<Future<Item = Response, Error = Error> + std::marker::Send>
    where Error: From<hyper::Error> + std::marker::Send
{
    REQUEST_CPU_POOL
        .spawn_fn(|| {
                      let ssl = NativeTlsClient::new().unwrap();
                      let connector = HttpsConnector::new(ssl);
                      let client = Client::with_connector(connector);
                      info!("calling GET {:?}", url);
                      Ok(try!(client.get(url).headers(headers).send()))
                  })
        .boxed()
}

pub fn post_async<Error: 'static>
    (url: Url,
     headers: Headers,
     body: String)
     -> Box<Future<Item = Response, Error = Error> + std::marker::Send>
    where Error: From<hyper::Error> + std::marker::Send
{
    REQUEST_CPU_POOL
        .spawn_fn(move || {
                      let ssl = NativeTlsClient::new().unwrap();
                      let connector = HttpsConnector::new(ssl);
                      let client = Client::with_connector(connector);
                      info!("calling POST {:?}", url);
                      Ok(try!(client.post(url).headers(headers).body(&body).send()))
                  })
        .boxed()
}
