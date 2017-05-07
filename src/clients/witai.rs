use hyper::Url;
use hyper::header::{Headers, Authorization};
use futures::future::*;
use std::collections::HashMap;
use serde_json;
use hyper;
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

impl From<Response> for ::sami::NlpResponse {
    fn from(response: Response) -> ::sami::NlpResponse {
        info!("{:?}", response);
        match response.entities.get("intent") {
            Some(values) => {
                let values = values
                    .iter()
                    .map(|value| value.value.clone())
                    .collect::<Vec<String>>();
                match values {
                    ref intent_self if intent_self.len() == 1 && intent_self[0] == "get_self" => {
                        ::sami::NlpResponse {
                            intent: ::sami::Intent::GetSelf,
                            ..Default::default()
                        }
                    }
                    ref intent_self if intent_self.len() == 1 && intent_self[0] == "logout" => {
                        ::sami::NlpResponse {
                            intent: ::sami::Intent::Logout,
                            ..Default::default()
                        }
                    }
                    ref intent_self if intent_self.len() == 1 && intent_self[0] == "get_field" => {
                        ::sami::NlpResponse {
                            intent: ::sami::Intent::GetField,
                            device: response
                                .entities
                                .get("target")
                                .map(|values| {
                                         values
                                             .iter()
                                             .map(|value| value.value.to_lowercase().clone())
                                             .collect::<Vec<String>>()
                                     }),
                            field: response
                                .entities
                                .get("field")
                                .and_then(|values| {
                                              values
                                                  .get(0)
                                                  .map(|value| {
                                                           value.value.to_lowercase().clone()
                                                       })
                                          }),
                            ..Default::default()
                        }
                    }
                    intents => {
                        ::sami::NlpResponse {
                            intent: ::sami::Intent::Unknown,
                            meta: Some(intents),
                            ..Default::default()
                        }
                    }
                }
            }
            None => {
                ::sami::NlpResponse {
                    intent: ::sami::Intent::Unknown,
                    ..Default::default()
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct WitAiError {
    msg: String,
}
impl From<hyper::Error> for WitAiError {
    fn from(err: hyper::Error) -> WitAiError {
        WitAiError { msg: format!("couldn't contact wit.ai: {:?}", err) }
    }
}
impl From<serde_json::Error> for WitAiError {
    fn from(err: serde_json::Error) -> WitAiError {
        WitAiError { msg: format!("error parsing json: {:?}", err) }
    }
}

pub struct WitAi {}

impl WitAi {
    pub fn get(query: &str) -> Box<Future<Item = Response, Error = WitAiError>> {
        let mut url = Url::parse("https://api.wit.ai/message").unwrap();
        url.query_pairs_mut()
            .append_pair("v", &CONFIGURATION.witai_version)
            .append_pair("q", query);
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", CONFIGURATION.witai_token).to_owned()));

        future_request::get_async::<WitAiError>(url, headers)
            .and_then(|response| match serde_json::from_reader(response) {
                          Ok(response) => Ok(response),
                          Err(err) => Err(err)?,
                      })
            .boxed()
    }
}
