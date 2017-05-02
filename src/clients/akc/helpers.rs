use std;
use hyper;
use hyper::header::{Headers, Authorization};
use serde;
use serde_json;
use futures::future::*;
use hyper::status::StatusCode;


use clients::akc::Akc;
use clients::akc::error::{AkcClientError, ErrorWrapper};
use clients::future_request;

impl Akc {
    pub fn auth_header(self: &Akc) -> Headers {
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", self.token)));
        headers
    }

    pub fn get<Wrapper>
        (self: &Akc,
         url: hyper::Url)
         -> Box<Future<Item = Wrapper::Data, Error = AkcClientError> + std::marker::Send>
        where Wrapper: DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: 'static
    {
        future_request::get_async::<AkcClientError>(url, self.auth_header())
            .and_then(move |response| match StatusCode::from_u16(response.status_raw().0) {
                          StatusCode::Ok => {
                let data_wrapper: Wrapper = match serde_json::from_reader(response) {
                    Ok(data_wrapper) => data_wrapper,
                    Err(error) => Err(error)?,
                };
                Ok(data_wrapper.data())
            }
                          _ => {
                let error_wrapper: ErrorWrapper = match serde_json::from_reader(response) {
                    Ok(error_wrapper) => error_wrapper,
                    Err(error) => Err(error)?,
                };
                Err(error_wrapper)?
            }
                      })
            .boxed()
    }
}

pub trait DataWrapper {
    type Data: std::marker::Send;
    fn data(self: Self) -> Self::Data;
}

macro_rules! data_wrapper {
    ( $w:ident, $d:ident ) => {
        #[derive(Deserialize, Serialize, Debug, Clone)]
        struct $w {
            data: $d,
        }

        impl helpers::DataWrapper for $w {
            type Data = $d;
            fn data(self: Self) -> Self::Data {
                self.data
            }
        }
    }
}
