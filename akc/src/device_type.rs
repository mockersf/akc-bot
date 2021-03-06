use hyper::Url;
use futures::future::*;

use oauth2;

use Akc;
use error::AkcClientError;
use helpers;

paginated_wrapper!(DataDeviceTypes, DeviceTypes, device_types, DeviceType);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeviceType {
    id: String,
    #[serde(rename = "uniqueName")]
    unique_name: String,
    name: String,
}

impl Akc {
    pub fn device_types_parallel(token: oauth2::Token) -> Box<Future<Item = Vec<DeviceType>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/devicetypes", Self::base_url())).unwrap();
        Self::get_all_pages_async_parallel::<DataDeviceTypes>(token, url)
    }
    pub fn device_types_sequential(token: oauth2::Token) -> Box<Future<Item = Vec<DeviceType>, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/devicetypes", Self::base_url())).unwrap();
        Self::get_all_pages_async_sequential::<DataDeviceTypes>(token, url)
    }
}
