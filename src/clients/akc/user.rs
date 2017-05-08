use hyper::Url;
use futures::future::*;

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;

data_wrapper!(DataUser, User);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
}

impl Akc {
    pub fn user_self(token: ::clients::oauth2::Token) -> Box<Future<Item = User, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/self", Self::base_url::<'static>())).unwrap();

        Self::get::<DataUser>(token, url)
    }
}
