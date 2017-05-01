use iron::{Handler, status, IronResult, Response, Request};
use bodyparser;

use iron::prelude::*;
use serde_json;

use witai::WitAi;

use handlers::lib::my_error::MyError;

#[derive(Deserialize, Debug, Clone)]
struct User {
    name: String,
    id: u32,
    mention_name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Room {
    name: String,
    id: u32,
}

#[derive(Deserialize, Debug, Clone)]
struct Message {
    message: String,
    id: String,
    from: User,
}

#[derive(Deserialize, Debug, Clone)]
struct Item {
    room: Option<Room>,
    message: Option<Message>,
}

#[derive(Deserialize, Debug, Clone)]
struct Notification {
    event: EventType,
    item: Item,
    oauth_client_id: String,
    webhook_id: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum EventType {
    RoomMessage,
}

#[derive(Serialize, Debug, Clone)]
struct NotificationResponse {
    message: String,
    color: Color,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum Color {
    Green,
}
use handlers::hipchat::Color::*;

create_handler!(ReceiveNotification,
                [],
                |_: &ReceiveNotification, req: &mut Request| {
    let struct_body = req.get::<bodyparser::Struct<Notification>>();
    match struct_body {
        Ok(Some(struct_body)) => {
            info!("Parsed body:\n{:?}", struct_body);
            let wit_ai_response = WitAi::get(&struct_body.item.message.unwrap().message);
            info!("{:?}", wit_ai_response);
            let message = serde_json::to_string(&wit_ai_response).unwrap();
            Ok(Response::with((status::Ok,
                               serde_json::to_string(&NotificationResponse {
                                                          message,
                                                          color: Green,
                                                      })
                                       .unwrap())))
        }
        Ok(None) => MyError::http_error(status::BadRequest, "missing body"),
        Err(err) => {
            MyError::http_error(status::BadRequest,
                                &format!("invalid JSON: {:?}", err).to_string())
        }
    }

});
