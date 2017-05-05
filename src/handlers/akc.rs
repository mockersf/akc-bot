use std::collections::HashMap;

use iron::{Handler, status, IronResult, Response, Request};
use urlencoded::UrlEncodedQuery;

use iron::prelude::*;

use futures::Future;

use DATABASE;
use CONFIGURATION;
use clients::akc::Akc;

create_handler!(ExchangeToken, |_: &ExchangeToken, req: &mut Request| {
    let params = get_query_params!(req, "code", "state");
    match (params.get("code"), params.get("state")) {
        (Some(code), Some(state)) => {
            match Akc::exchange_token(CONFIGURATION.akc_appid.to_owned(),
                                      CONFIGURATION.akc_appsecret.to_owned(),
                                      code[0].to_owned())
                          .and_then(|token| {
                                        DATABASE
                                            .lock()
                                            .unwrap()
                                            .add_token(state[0].to_string(), token);
                                        Ok(())
                                    })
                          .wait() {
                Ok(()) => Ok(Response::with((status::Ok, "You can now return to hipchat"))),
                Err(err) => {
                    warn!("{:?}", err);
                    Ok(Response::with((status::BadRequest, "failed to exchange tokens")))
                }
            }
        }
        (_, _) => Ok(Response::with((status::BadRequest, "failed to exchange tokens"))),
    }
});
