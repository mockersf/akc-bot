use hyper::Url;
use hyper::header::{Headers, Authorization};
use futures::future::*;
use std::io::Read;
use std::collections::HashMap;
use serde_json;

use clients::future_request;

use CONFIGURATION;


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Value {
    value: String,
    confidence: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(skip_serializing)]
    msg_id: String,
    #[serde(skip_serializing)]
    _text: String,
    entities: HashMap<String, Vec<Value>>,
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

pub struct WitAi {}

impl WitAi {
    pub fn get(query: &String) -> Box<Future<Item = Response, Error = Error>> {
        let quer = query.clone();
        let mut url = Url::parse("https://api.wit.ai/message").unwrap();
        url.query_pairs_mut()
            .append_pair("v", &CONFIGURATION.version)
            .append_pair("q", &quer);
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", CONFIGURATION.wit_ai_token).to_owned()));

        future_request::get_async(url, headers)
            .map(move |mut response| -> Response {
                     let mut s = String::new();
                     response.read_to_string(&mut s).unwrap();
                     serde_json::from_str(&s).unwrap()
                 })
            .map_err(|e| Error { msg: format!("error getting response from wit.ai: {:?}", e) })
            .boxed()
    }
}
