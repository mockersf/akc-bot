#[macro_use]
mod helpers;

mod error;
mod user;
mod device;
mod device_type;
pub mod token;

#[derive(Debug, Clone)]
pub struct Akc {
    token: String,
    base_url: String,
}

impl Akc {
    pub fn base_url<'a>() -> &'a str {
        "https://api.artik.cloud/v1.1"
    }
}
