use hyper::header::{Headers, Authorization};

mod error;
mod user;

pub struct Akc {
    token: String,
    base_url: String,
}

impl Akc {
    pub fn new(token: String) -> Akc {
        Akc {
            token,
            base_url: "https://api.artik.cloud/v1.1".to_string(),
        }
    }

    fn auth_header(self: &Akc) -> Headers {
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", self.token)));
        headers
    }
}
