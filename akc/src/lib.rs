extern crate hyper;
extern crate hyper_native_tls;
extern crate url;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate futures;

extern crate oauth2;
extern crate future_request;

#[macro_use]
mod helpers;

mod error;
pub mod user;
pub mod device;
pub mod device_type;
pub mod snapshot;

#[derive(Debug, Clone)]
pub struct Akc {}

impl Akc {
    pub fn base_url<'a>() -> &'a str {
        "https://api.artik.cloud/v1.1"
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
