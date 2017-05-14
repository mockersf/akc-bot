#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    SendNotification,
}

#[derive(Serialize, Debug)]
pub struct HipchatApiConsumer {
    pub from_name: String,
    pub scopes: Vec<Scope>,
}

use clients::hipchat::notification;

#[derive(Serialize, Debug)]
pub struct Webhook {
    pub authentication: String,
    pub event: notification::EventType,
    pub name: String,
    pub pattern: String,
    pub url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Installable {
    pub allow_global: bool,
    pub allow_room: bool,
    pub callback_url: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(skip_serializing_if="Option::is_none")]
    pub webhook: Option<Vec<Webhook>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub installable: Option<Installable>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub hipchat_api_consumer: Option<HipchatApiConsumer>,
}

#[derive(Serialize, Debug)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_url: String,
    pub homepage: String,
}

#[derive(Serialize, Debug)]
pub struct Vendor {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Debug)]
pub struct Descriptor {
    pub name: String,
    pub key: String,
    pub description: String,
    pub links: Links,
    #[serde(skip_serializing_if="Option::is_none")]
    pub vendor: Option<Vendor>,
    pub capabilities: Capabilities,
}
