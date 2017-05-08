use hyper::Url;
use futures::future::*;

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;

paginated_wrapper!(DataDevices, Devices, devices, Device);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Device {
    pub id: String,
    pub dtid: String,
    pub name: String,
}

impl Akc {
    pub fn devices_parallel(token: ::clients::oauth2::Token, uid: &str) -> Box<Future<Item = Vec<Device>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/{}/devices", Self::base_url(), uid)).unwrap();
        Self::get_all_pages_async_parallel::<DataDevices>(token, url)
    }
    pub fn devices_sequential(token: ::clients::oauth2::Token, uid: &str) -> Box<Future<Item = Vec<Device>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/users/{}/devices", Self::base_url(), uid)).unwrap();
        Self::get_all_pages_async_sequential::<DataDevices>(token, url)
    }
}
