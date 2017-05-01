use hyper::Url;
use hyper::header::{Headers, Authorization};
use futures::future::*;
use std::io::Read;
use serde_json;

use clients::future_request;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DataUser {
    data: User,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    id: String,
    #[serde(rename = "fullName")]
    full_name: String,
    email: String,
}

#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
}

pub struct Akc {
    pub token: String,
}

impl Akc {
    pub fn user_self(self: &Akc) -> Box<Future<Item = User, Error = Error>> {
        let url = Url::parse("https://api.artik.cloud/v1.1/users/self").unwrap();
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", self.token)));

        read_body!(future_request::get_async(url, headers),
                   DataUser,
                   "error getting response from AKC: {:?}")
                .map(|data_user| data_user.unwrap().data)
                .boxed()
    }
}
