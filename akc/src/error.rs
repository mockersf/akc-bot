use std;
use hyper;
use serde_json;


#[derive(Deserialize, Debug, Clone)]
pub struct ErrorWrapper {
    error: AkcError,
}
#[derive(Deserialize, Debug, Clone)]
struct AkcError {
    code: u32,
    message: String,
}

#[derive(Debug, Clone)]
struct InternalError {
    msg: String,
}

#[derive(Debug, Clone)]
pub enum AkcClientError {
    InternalError(String),
    AkcError(u32, String),
}

impl From<hyper::Error> for AkcClientError {
    fn from(err: hyper::Error) -> AkcClientError {
        AkcClientError::InternalError(format!("couldn't contact AKC: {:?}", err))
    }
}
impl From<std::io::Error> for AkcClientError {
    fn from(err: std::io::Error) -> AkcClientError {
        AkcClientError::InternalError(format!("couldn't read response from AKC: {:?}", err))
    }
}
impl From<serde_json::Error> for AkcClientError {
    fn from(err: serde_json::Error) -> AkcClientError {
        AkcClientError::InternalError(format!("error parsing json: {:?}", err))
    }
}
impl From<ErrorWrapper> for AkcClientError {
    fn from(err: ErrorWrapper) -> AkcClientError {
        AkcClientError::AkcError(err.error.code, err.error.message)
    }
}
