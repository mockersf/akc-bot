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
    pub fn user_self(self: &Akc) -> Box<Future<Item = User, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/self", self.base_url)).unwrap();

        self.get::<DataUser>(url)
    }
}
