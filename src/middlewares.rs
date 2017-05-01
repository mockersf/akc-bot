extern crate iron;
extern crate router;

use iron::{Request, Response, IronResult, AfterMiddleware};
use iron::error::IronError;
use iron::status;
use iron::headers::ContentType;
use router::NoRoute;


pub struct Default404;
impl AfterMiddleware for Default404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        if let Some(_) = err.error.downcast::<NoRoute>() {
            Ok(Response::with((status::NotFound, "{\"error\": \"path not found\"}")))
        } else {
            Err(err)
        }
    }
}

pub struct JsonResponse;
impl AfterMiddleware for JsonResponse {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        res.headers.set(ContentType::json());
        Ok(res)
    }
}

pub struct ErrorLogger;
impl AfterMiddleware for ErrorLogger {
    fn after(&self, _: &mut Request, res: Response) -> IronResult<Response> {
        Ok(match res.status {
               Some(status::Ok) => res,
               Some(status::Created) => res,
               Some(other) => {
            let mut body: Vec<u8> = Vec::new();
            match res.body.unwrap().write_body(&mut body) {
                Ok(_) => (),
                Err(e) => error!("error reading response body: {}", e),
            };
            let body = String::from_utf8(body).unwrap();
            warn!("there was an error: {:?}", body);
            Response::with((other, body))
        }
               None => res,
           })
    }
}
