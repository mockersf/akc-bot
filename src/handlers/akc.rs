use std::collections::HashMap;

use hyper::header::{Headers, Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::Url;
use hyper;

use iron::{Handler, status, IronResult, Response, Request};
use urlencoded::UrlEncodedQuery;

use iron::prelude::*;
use serde_json;

use futures::Future;

use DATABASE;
use CONFIGURATION;
use clients::future_request;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AkcToken {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u32,
}

create_handler!(ExchangeToken, |_: &ExchangeToken, req: &mut Request| {
    let params = get_query_params!(req, "code", "state");
    match (params.get("code"), params.get("state")) {
        (Some(code), Some(state)) => {
            let url = Url::parse("https://accounts.artik.cloud/token").unwrap();
            let mut headers = Headers::new();
            headers.set(Authorization(Basic {
                                          username: CONFIGURATION.akc_appid.to_owned(),
                                          password: Some(CONFIGURATION.akc_appsecret.to_owned()),
                                      }));
            headers.set(
                ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![]))
            );
            match future_request::post_async::<hyper::Error>(url, headers, format!("grant_type=authorization_code&code={}", code[0]))
            .map(|response| {
                match serde_json::from_reader::<_, AkcToken>(response) {
                    Ok(token) =>  {
                        let token = ::clients::akc::token::Token::new(token.access_token);
                        DATABASE.lock().unwrap().add_token(state[0].to_string(), token);
                        true
                    },
                    Err(err) => {
                        warn!("{:?}", err);
                        false
                    },
                }
            }).wait() {
                Ok(true) => Ok(Response::with((status::Ok, "You can now return to hipchat"))),
                _ => Ok(Response::with((status::BadRequest, "failed to exchange tokens"))),
            }
        }
        (_, _) => Ok(Response::with((status::BadRequest, "failed to exchange tokens"))),
    }
});
