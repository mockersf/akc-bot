use std::collections::HashMap;
use std::fmt;

use hyper::Url;
use futures::future::*;

use oauth2;

use Akc;
use error::AkcClientError;
use helpers;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DataSnapshot {
    data: Vec<Snapshot>,
}
impl helpers::DataWrapper for DataSnapshot {
    type Data = Vec<Snapshot>;
    fn data(self: Self) -> Self::Data {
        self.data
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Snapshot {
    pub sdid: String,
    pub data: FieldData,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum FieldData {
    Field { ts: Option<u64>, value: FieldValue },
    Group(HashMap<String, Box<FieldData>>),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum FieldValue {
    Float(f64),
    Int(i64),
    String(String),
    Boolean(bool),
}
impl fmt::Display for FieldValue {
    fn fmt(&self, fm: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldValue::Float(f) => write!(fm, "{:?}", f),
            FieldValue::Int(i) => write!(fm, "{:?}", i),
            FieldValue::String(ref s) => write!(fm, "{:?}", s),
            FieldValue::Boolean(b) => write!(fm, "{:?}", b),
        }
    }
}

impl Akc {
    pub fn snapshots(token: oauth2::Token, sdid: Vec<String>) -> Box<Future<Item = Vec<Snapshot>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/messages/snapshots", Self::base_url::<'static>())).unwrap();

        Self::get_with_params::<DataSnapshot>(token, url, vec![("sdids".to_string(), sdid.join(","))])
    }
}
