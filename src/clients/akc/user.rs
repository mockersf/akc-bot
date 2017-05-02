use hyper::Url;
use futures::future::*;

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;
use clients::future_request;

data_wrapper!(DataUser, User);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    id: String,
    #[serde(rename = "fullName")]
    full_name: String,
    email: String,
}

impl Akc {
    pub fn user_self(self: &Akc) -> Box<Future<Item = User, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/self", self.base_url)).unwrap();

        future_request::get_async::<AkcClientError>(url, self.auth_header())
            .map(move |mut response| helpers::response_to_string(response))
            .and_then(move |response| helpers::extract::<DataUser>(&response))
            .boxed()
    }
}
