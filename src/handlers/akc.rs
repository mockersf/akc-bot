use std::collections::HashMap;

use iron::{Handler, status, IronResult, Response, Request};
use iron::headers::ContentType;
use urlencoded::UrlEncodedQuery;

use iron::prelude::*;

use DATABASE;
use CONFIGURATION;
use oauth2;

create_handler!(ExchangeToken, |_: &ExchangeToken, req: &mut Request| {
    let params = get_query_params!(req, "code", "state");
    match (params.get("code"), params.get("state"), params.get("error")) {
        (Some(code), Some(state), _) => {
            match oauth2::Oauth2::new(CONFIGURATION.akc_appid.to_owned(),
                                      CONFIGURATION.akc_appsecret.to_owned(),
                                      "https://accounts.artik.cloud/token")
                          .unwrap()
                          .exchange_token(oauth2::AuthorizationCode { code: code[0].to_owned() }) {
                Ok(token) => {
                    DATABASE
                        .lock()
                        .unwrap()
                        .add_token(state[0].to_string(), token);
                    let mut response = Response::with((status::Ok, "You can now return to hipchat"));
                    response.headers.set(ContentType::html());
                    Ok(response)
                }
                Err(err) => {
                    warn!("{:?}", err);
                    Ok(Response::with((status::BadRequest, format!("failed to exchange tokens: {:?}", err))))
                }
            }
        }
        (_, _, Some(error)) => Ok(Response::with((status::BadRequest, format!("failed to exchange tokens: {:?}", error)))),
        (_, _, _) => Ok(Response::with((status::BadRequest, "failed to exchange tokens"))),
    }
});
