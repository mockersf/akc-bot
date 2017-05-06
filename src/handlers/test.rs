use iron::{Handler, status, IronResult, Response, Request};
use bodyparser;
use router::Router;
use urlencoded::UrlEncodedQuery;

use iron::prelude::*;
use serde_json;
use futures::Future;

use DATABASE;

use handlers::lib::my_error::MyError;

#[derive(Deserialize, Debug, Clone)]
struct Token {
    access_token: String,
}

create_handler!(SetTokenForContext,
                |_: &SetTokenForContext, req: &mut Request| {
    let struct_body = req.get::<bodyparser::Struct<Token>>();
    match struct_body {
        Ok(Some(struct_body)) => {
            let from = get_path_param!(req, "from").to_string();
            let token = ::clients::oauth2::Token::from_access_token(struct_body.access_token);
            DATABASE.lock().unwrap().add_token(from, token);

            Ok(Response::with(status::Created))
        }
        Ok(None) => MyError::http_error(status::BadRequest, "missing body"),
        Err(err) => {
            MyError::http_error(status::BadRequest,
                                &format!("invalid JSON: {:?}", err).to_string())
        }
    }
});

create_handler!(GetUserFromContext,
                |_: &GetUserFromContext, req: &mut Request| {
                    let from = get_path_param!(req, "from").to_string();
                    Ok(Response::with((status::Ok,
            serde_json::to_string(&::clients::akc::Akc::user_self(from).wait().unwrap()).unwrap())))
                });

create_handler!(GetDevicesFromContext,
                |_: &GetDevicesFromContext, req: &mut Request| {
    let from = get_path_param!(req, "from").to_string();
    let uid = ::clients::akc::Akc::user_self(from.clone())
        .wait()
        .unwrap()
        .id;
    let future = match get_query_param!(req, "sequential") {
        Some(_) => ::clients::akc::Akc::devices_sequential(from, &uid),
        None => ::clients::akc::Akc::devices_parallel(from, &uid),
    };

    Ok(Response::with((status::Ok, serde_json::to_string(&future.wait().unwrap()).unwrap())))
});

create_handler!(GetDeviceTypesFromContext,
                |_: &GetDeviceTypesFromContext, req: &mut Request| {
    let from = get_path_param!(req, "from").to_string();
    let future = match get_query_param!(req, "sequential") {
        Some(_) => ::clients::akc::Akc::device_types_sequential(from),
        None => ::clients::akc::Akc::device_types_parallel(from),
    };
    Ok(Response::with((status::Ok, serde_json::to_string(&future.wait().unwrap()).unwrap())))

});
