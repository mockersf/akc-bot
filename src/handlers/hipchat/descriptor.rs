use iron::{Handler, status, IronResult, Response, Request};
use serde_json;

use clients::hipchat::configuration::*;
use clients::hipchat::notification;

use CONFIGURATION;

create_handler!(AddOnDescriptor, |_: &AddOnDescriptor, _: &mut Request| {
    let self_url = &CONFIGURATION.self_url;
    let webhook = Webhook {
        authentication: "jwt".to_string(),
        event: notification::EventType::RoomMessage,
        name: "Sami webhook".to_string(),
        pattern: "^\\/sami .*$".to_string(),
        url: format!("{}/hipchat/notification", self_url),
    };
    let installable = Installable {
        allow_global: false,
        allow_room: true,
        callback_url: format!("{}/hipchat/installation", self_url),
    };
    let api_consumer = HipchatApiConsumer {
        from_name: "Sami AddOn".to_string(),
        scopes: vec![Scope::SendNotification],
    };
    let capabilities = Capabilities {
        webhook: Some(vec![webhook]),
        installable: Some(installable),
        hipchat_api_consumer: Some(api_consumer),
    };
    let links = Links {
        self_url: format!("{}/hipchat", self_url),
        homepage: "https://artik.cloud/".to_string(),
    };
    let descriptor = Descriptor {
        name: "Sami AddOn".to_string(),
        key: "sami-add-on".to_string(),
        description: "Interact with AKC devices".to_string(),
        capabilities: capabilities,
        vendor: None,
        links: links,
    };
    Ok(Response::with((status::Ok, serde_json::to_string(&descriptor).unwrap())))
});
