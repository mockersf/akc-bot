use iron::{Handler, status, IronResult, Response, Request};
use iron::headers::ContentType;

use CONFIGURATION;

create_handler!(HomePage, |_: &HomePage, _: &mut Request| {
    let mut response = Response::with((status::Ok,
                                       format!("url for hipchat descriptor: {}/hipchat",
                                               &CONFIGURATION.self_url)));
    response.headers.set(ContentType::html());
    Ok(response)
});
