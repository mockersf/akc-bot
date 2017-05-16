use iron::{Handler, status, IronResult, Response, Request};
use bodyparser;
use iron::prelude::*;
use serde_json;
use futures::Future;
use hyper::header::{Authorization, Scheme};
use hyper::error::Error;
use std::str::FromStr;
use jwt::{Header, Registered, Token};
use crypto::sha2::Sha256;

use clients::witai::WitAi;
use sami;

use handlers::lib::my_error::MyError;

use DATABASE;
use handlers::hipchat::HC_DATABASE;

use CONFIGURATION;

use clients::hipchat::message::*;
use clients::hipchat::notification::*;

fn notification_from_message(message: sami::output::MessageToUser) -> NotificationResponse {
    info!("{:?}", message);
    NotificationResponse {
        message: match message.intent {
            ::sami::Intent::GetSelf => format!("You are connected as {}.", message.data[0]),
            ::sami::Intent::Logout => {
                DATABASE
                    .lock()
                    .unwrap()
                    .remove_token(message.data[0].clone());
                "You are now logged out.".to_string()
            }
            ::sami::Intent::ForcedLogout => {
                DATABASE
                    .lock()
                    .unwrap()
                    .remove_token(message.data[0].clone());
                "Error communicating with ARTIK Cloud. You have been logged out.".to_string()
            }
            ::sami::Intent::GetField => {
                match message.data.len() {
                    1 => format!("No device found for '{}'.", message.data[0]),
                    2 => {
                        format!("No field '{}' found for device '{}'.",
                                message.data[1],
                                message.data[0])
                    }
                    3 => {
                        format!("{}'s {} is {}.",
                                message.data[0],
                                message.data[1],
                                message.data[2])
                    }
                    _ => "uuuh ?".to_string(),
                }
            }
            ::sami::Intent::Unknown => {
                format!("Unknown intent: {:?}",
                        if !message.data.is_empty() {
                            message.data[0].clone()
                        } else {
                            "'no intent found'".to_string()
                        })
            }
            intent => format!("{:?} not yet done", intent),
        },
        color: match message.status {
            sami::output::Status::Info => Color::Purple,
            sami::output::Status::Confirmation => Color::Green,
            sami::output::Status::Error => Color::Red,
            sami::output::Status::ActionRequired => Color::Yellow,
        },
    }
}

#[derive(Clone, Debug)]
struct JWT {
    iss: String,
    sub: String,
    exp: u64,
    iat: u64,
    jti: String,
}
use std::fmt;
impl Scheme for JWT {
    fn scheme() -> Option<&'static str> {
        Some("JWT")
    }

    fn fmt_scheme(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.iss)
    }
}
impl FromStr for JWT {
    type Err = Error;
    fn from_str(s: &str) -> Result<JWT, Error> {
        let token = Token::<Header, Registered>::parse(s).unwrap();
        let secret = {
            let locked = HC_DATABASE.lock().unwrap();
            let installation = locked.get_installation(token.claims.iss.clone().unwrap());
            if !installation.is_some() {
                return Err(Error::Header);
            }
            installation.unwrap().clone().oauth_secret
        };
        if token.verify(secret.as_bytes(), Sha256::new()) {
            Ok(JWT {
                   iss: token.claims.iss.unwrap(),
                   sub: token.claims.sub.unwrap(),
                   exp: token.claims.exp.unwrap(),
                   iat: token.claims.iat.unwrap(),
                   jti: token.claims.jti.unwrap(),
               })
        } else {
            Err(Error::Header)
        }
    }
}

create_handler!(ReceiveNotification,
                |_: &ReceiveNotification, req: &mut Request| {
    {
        let auth = req.headers.get::<Authorization<JWT>>();
        if !auth.is_some() {
            return MyError::http_error(status::Unauthorized, "invalid JWT");
        }
    }
    let struct_body = req.get::<bodyparser::Struct<Notification>>();
    match struct_body {
        Ok(Some(struct_body)) => {
            let context_identifier = format!("hipchatroom-{}-{}",
                                             struct_body.oauth_client_id,
                                             struct_body.item.room.unwrap().id);
            //wrapped to release lock but keep info on presence
            let akc_access_token = {
                let locked = DATABASE.lock().unwrap();
                locked.get_token(context_identifier.clone()).cloned()
            };
            if let Some(akc_access_token) = akc_access_token {
                let trigger = &struct_body.item.message.unwrap().message[(CONFIGURATION.hipchat_command.len() + 1)..];
                let wit_ai_response_future = WitAi::get(trigger);
                let wit_ai_response = sami::input::NlpResponse::from(wit_ai_response_future.wait().unwrap());
                let message = sami::output::MessageToUser::from(akc_access_token.clone(), wit_ai_response);
                Ok(Response::with((status::Ok, serde_json::to_string(&notification_from_message(message)).unwrap())))
            } else {
                let signin_message = format!("This room is not authenticated.
                Please <a href=\"https://accounts.artik.cloud/authorize?client_id={}&amp;state={}&amp;response_type=code\">sign in</a>.",
                                             CONFIGURATION.akc_appid,
                                             context_identifier);

                Ok(Response::with((status::Ok,
                                   serde_json::to_string(&NotificationResponse {
                                                             message: signin_message,
                                                             color: Color::Yellow,
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
