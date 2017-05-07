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
