use hyper::Client;
use hyper::Url;
use hyper::header::{Headers, Authorization};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use std::io::Read;
use std::collections::HashMap;

use serde_json;

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


pub struct WitAi {}

impl WitAi {
    pub fn get(query: &String) -> Response {
        let mut url = Url::parse("https://api.wit.ai/message").unwrap();
        url.query_pairs_mut()
            .append_pair("v", &CONFIGURATION.version)
            .append_pair("q", &query);
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);
        let mut s = String::new();
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", CONFIGURATION.wit_ai_token).to_owned()));
        client
            .get(url)
            .headers(headers)
            .send()
            .unwrap()
            .read_to_string(&mut s)
            .unwrap();
        let deserialized: Response = serde_json::from_str(&s).unwrap();
        info!("{:?}", deserialized);

        deserialized
    }
}
