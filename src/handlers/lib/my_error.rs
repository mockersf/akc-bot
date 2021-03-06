use uuid::Uuid;
use iron::{status, IronResult, Response};
use serde_json;

#[derive(Clone, Debug, Serialize)]
pub struct MyError {
    id: String,
    error: String,
}

impl MyError {
    pub fn new(error_message: &str) -> MyError {
        MyError {
            error: error_message.to_string(),
            id: Uuid::new_v4().hyphenated().to_string(),
        }
    }

    pub fn as_http_error(&self, status_code: status::Status) -> IronResult<Response> {
        let payload = serde_json::to_string(self).unwrap();
        Ok(Response::with((status_code, payload)))
    }

    pub fn http_error(status_code: status::Status, error_message: &str) -> IronResult<Response> {
        let error = MyError::new(error_message);
        error.as_http_error(status_code)
    }
}
