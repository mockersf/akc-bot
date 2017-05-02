use std::io::Read;
use hyper;
use hyper::header::{Headers, Authorization};
use serde;
use serde_json;

use clients::akc::Akc;
use clients::akc::error::{AkcClientError, ErrorWrapper};

impl Akc {
    pub fn auth_header(self: &Akc) -> Headers {
        let mut headers = Headers::new();
        headers.set(Authorization(format!("Bearer {}", self.token)));
        headers
    }
}

pub trait DataWrapper {
    type Data;
    fn data(self: Self) -> Self::Data;
}

pub fn response_to_string(mut response: hyper::client::Response) -> String {
    let mut s = String::new();
    response.read_to_string(&mut s);
    s
}

pub fn extract<'de, Wrapper: DataWrapper>(response: &'de str)
                                          -> Result<Wrapper::Data, AkcClientError>
    where Wrapper: serde::Deserialize<'de>
{
    let data_wrapper: Wrapper = match serde_json::from_str(&response) {
        Ok(data_wrapper) => data_wrapper,
        Err(_) => {
            let error: ErrorWrapper = serde_json::from_str(&response)?;
            return Err(error)?;
        }
    };
    Ok(data_wrapper.data())
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
