use iron::{Handler, status, IronResult, Response, Request};
use bodyparser;

use iron::prelude::*;
use serde_json;

use clients::witai::WitAi;
use futures::Future;

use handlers::lib::my_error::MyError;

use DATABASE;

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
    Yellow,
    Blue,
}
use handlers::hipchat::Color::*;

use CONFIGURATION;

fn generate_response(from: String, wit_ai_response: ::clients::witai::Response) -> String {
    info!("from: {:?}", from);
    info!("wit_ai_response: {:?}", wit_ai_response);
    ::clients::akc::Akc::user_self(from).wait().unwrap().full_name
}

create_handler!(ReceiveNotification,
                |_: &ReceiveNotification, req: &mut Request| {
    let struct_body = req.get::<bodyparser::Struct<Notification>>();
    match struct_body {
        Ok(Some(struct_body)) => {
            info!("Parsed body: {:?}", struct_body);
            let context_identifier = format!("hipchatroom-{}-{}",
                                             struct_body.oauth_client_id,
                                             struct_body.item.room.unwrap().id);
            //wrapped to release lock but keep info on presence
            let res = {
                let locked = DATABASE.lock().unwrap();
                locked.get_token(context_identifier.clone()).is_some()
            };
            if res {
                let trigger = struct_body.item.message.unwrap().message;
                let wit_ai_response_future = WitAi::get(&trigger);
                let wit_ai_response = wit_ai_response_future.wait().unwrap();
                let message = generate_response(context_identifier, wit_ai_response);
                Ok(Response::with((status::Ok,
                                   serde_json::to_string(&NotificationResponse {
                                                              message,
                                                              color: Green,
                                                          })
                                           .unwrap())))
            } else {
                let signin_message = format!("This room is not authenticated. Please <a href=\"https://accounts.artik.cloud/authorize?client_id={}&amp;state={}&amp;response_type=code\">sign in</a>.", CONFIGURATION.akc_appid, context_identifier);
                Ok(Response::with((status::Ok,
                                   serde_json::to_string(&NotificationResponse {
                                                              message: signin_message,
                                                              color: Yellow,
                                                          })
                                           .unwrap())))
            }
        }
        Ok(None) => MyError::http_error(status::BadRequest, "missing body"),
        Err(err) => {
            MyError::http_error(status::BadRequest,
                                &format!("invalid JSON: {:?}", err).to_string())
        }
    }

});
