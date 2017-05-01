use hyper::Url;
use futures::future::*;
use std::io::Read;
use serde_json;

use clients::akc::error::{AkcClientError, ErrorWrapper};

use clients::future_request;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct DataUser {
    data: User,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    id: String,
    #[serde(rename = "fullName")]
    full_name: String,
    email: String,
}

use clients::akc::Akc;

impl Akc {
    pub fn user_self(self: &Akc) -> Box<Future<Item = User, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/self", self.base_url)).unwrap();

        future_request::get_async::<AkcClientError>(url, self.auth_header())
            .and_then(move |mut response| {
                let mut s = String::new();
                response.read_to_string(&mut s)?;
                let user_wrapper: DataUser = match serde_json::from_str(&s) {
                    Ok(data_user) => data_user,
                    Err(_) => {
                        let error: ErrorWrapper = serde_json::from_str(&s)?;
                        return Err(error)?;
                    }
                };
                Ok(user_wrapper.data)
            })
            .boxed()
    }
}
