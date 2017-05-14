use iron::{Handler, status, IronResult, Response, Request};


create_handler!(HomePage,
                |_: &HomePage, _: &mut Request| Ok(Response::with((status::Ok, "hello".to_string()))));
