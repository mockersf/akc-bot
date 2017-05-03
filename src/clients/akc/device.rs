use hyper::Url;
use futures::future::*;

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;

paginated_wrapper!(DataDevices, Devices, devices, Device);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Device {
    id: String,
    dtid: String,
    name: String,
}

impl Akc {
    pub fn devices_wait(self: &Akc, uid: &String) -> Result<Vec<Device>, AkcClientError> {
        let url = Url::parse(&format!("{}/users/{}/devices", self.base_url, uid)).unwrap();
        self.get_all_pages_sync_parallel::<DataDevices>(url.clone())
    }
}
