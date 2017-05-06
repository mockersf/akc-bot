#[macro_use]
mod helpers;

mod error;
mod user;
mod device;
mod device_type;

#[derive(Debug, Clone)]
pub struct Akc {}

impl Akc {
    pub fn base_url<'a>() -> &'a str {
        "https://api.artik.cloud/v1.1"
    }
}
