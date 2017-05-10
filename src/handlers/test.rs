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
    let akc_access_token = {
        let locked = DATABASE.lock().unwrap();
        locked.get_token(from.clone()).cloned()
    };
    if let Some(akc_access_token) = akc_access_token {
        Ok(Response::with((status::Ok,
                           serde_json::to_string(&::clients::akc::Akc::user_self(akc_access_token.clone())
                                                      .wait()
                                                      .unwrap())
                                   .unwrap())))
    } else {
        Ok(Response::with((status::BadRequest, "context not fount")))
    }
});

create_handler!(GetDevicesFromContext,
                |_: &GetDevicesFromContext, req: &mut Request| {
    let from = get_path_param!(req, "from").to_string();
    let akc_access_token = {
        let locked = DATABASE.lock().unwrap();
        locked.get_token(from.clone()).cloned()
    };
    if let Some(akc_access_token) = akc_access_token {
        let uid = ::clients::akc::Akc::user_self(akc_access_token.clone())
            .wait()
            .unwrap()
            .id;
        let future = match get_query_param!(req, "sequential") {
            Some(_) => ::clients::akc::Akc::devices_sequential(akc_access_token.clone(), &uid),
            None => ::clients::akc::Akc::devices_parallel(akc_access_token.clone(), &uid),
        };

        Ok(Response::with((status::Ok, serde_json::to_string(&future.wait().unwrap()).unwrap())))
    } else {
        Ok(Response::with((status::BadRequest, "context not fount")))
    }

});

create_handler!(GetDeviceTypesFromContext,
                |_: &GetDeviceTypesFromContext, req: &mut Request| {
    let from = get_path_param!(req, "from").to_string();
    let akc_access_token = {
        let locked = DATABASE.lock().unwrap();
        locked.get_token(from.clone()).cloned()
    };
    if let Some(akc_access_token) = akc_access_token {
        let future = match get_query_param!(req, "sequential") {
            Some(_) => ::clients::akc::Akc::device_types_sequential(akc_access_token.clone()),
            None => ::clients::akc::Akc::device_types_parallel(akc_access_token.clone()),
        };
        Ok(Response::with((status::Ok, serde_json::to_string(&future.wait().unwrap()).unwrap())))
    } else {
        Ok(Response::with((status::BadRequest, "context not fount")))
    }

});

create_handler!(GetSnapshotFromContext,
                |_: &GetSnapshotFromContext, req: &mut Request| {
    let from = get_path_param!(req, "from").to_string();
    let sdid = get_path_param!(req, "sdid").to_string();
    let akc_access_token = {
        let locked = DATABASE.lock().unwrap();
        locked.get_token(from.clone()).cloned()
    };
    if let Some(akc_access_token) = akc_access_token {
        let future = ::clients::akc::Akc::snapshots(akc_access_token.clone(), vec![sdid]);
        Ok(Response::with((status::Ok, serde_json::to_string(&future.wait().unwrap()).unwrap())))
    } else {
        Ok(Response::with((status::BadRequest, "context not fount")))
    }
});
