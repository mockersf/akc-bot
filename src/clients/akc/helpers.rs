use std;
use hyper;
use hyper::header::{Headers, Authorization, Bearer};
use serde;
use serde_json;
use futures::future::*;
use hyper::status::StatusCode;

use futures::stream::{self, Stream};

use clients::akc::Akc;
use clients::akc::error::{AkcClientError, ErrorWrapper};
use clients::future_request;

use DATABASE;

impl Akc {
    fn auth_header(from: String) -> Headers {
        let mut headers = Headers::new();
        headers.set(
           Authorization(
               Bearer {
                   token: DATABASE.lock().unwrap().get_token(from)
                       .map(|token| token.bearer()).unwrap_or("missing_bearer").to_owned()
               }
           )
        );
        headers
    }

    pub fn get<Wrapper>
        (from: String,
         url: hyper::Url)
         -> Box<Future<Item = Wrapper::Data, Error = AkcClientError> + std::marker::Send>
        where Wrapper: DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: 'static
    {
        Self::get_with_params::<Wrapper>(from, url, vec![])
    }

    pub fn get_with_params<Wrapper>
        (from: String,
         mut url: hyper::Url,
         query_params: Vec<(String, String)>)
         -> Box<Future<Item = Wrapper::Data, Error = AkcClientError> + std::marker::Send>
        where Wrapper: DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: 'static
    {
        url.query_pairs_mut().extend_pairs(query_params);
        future_request::get_async::<AkcClientError>(url, Self::auth_header(from))
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


    pub fn get_paginated_with_params<Wrapper>
        (from: String,
         mut url: hyper::Url,
         query_params: Vec<(String, String)>)
         -> Box<Future<Item = (Wrapper::Data, u32), Error = AkcClientError> + std::marker::Send>
        where Wrapper: PaginatedWrapper + DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: 'static
    {
        url.query_pairs_mut().extend_pairs(query_params);
        future_request::get_async::<AkcClientError>(url, Self::auth_header(from))
            .and_then(move |response| match StatusCode::from_u16(response.status_raw().0) {
                          StatusCode::Ok => {
                let data_wrapper: Wrapper = match serde_json::from_reader(response) {
                    Ok(data_wrapper) => data_wrapper,
                    Err(error) => Err(error)?,
                };
                let total = data_wrapper.total();
                Ok((data_wrapper.data(), total))
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

    pub fn get_paginated_with_pagination_info<Wrapper>
         (from: String,
             mut url: hyper::Url,
         offset: u32,
     page_count: u32)
         -> Box<Future<Item = (Wrapper::Data, PageInformation),
          Error = AkcClientError> + std::marker::Send>
        where Wrapper: PaginatedWrapper + DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: 'static
    {
        url.query_pairs_mut()
            .append_pair("offset", &offset.to_string())
            .append_pair("count", &page_count.to_string());
        future_request::get_async::<AkcClientError>(url, Self::auth_header(from))
            .and_then(move |response| match StatusCode::from_u16(response.status_raw().0) {
                          StatusCode::Ok => {
                let data_wrapper: Wrapper = match serde_json::from_reader(response) {
                    Ok(data_wrapper) => data_wrapper,
                    Err(error) => Err(error)?,
                };
                let count = data_wrapper.count();
                Ok((data_wrapper.data(),
                    PageInformation {
                        offset: offset + page_count,
                        last_page_count: count,
                    }))
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

    pub fn get_all_pages_async_sequential<Wrapper>
         (from: String, url: hyper::Url)
         -> Box<Future<Item = Vec<<<Wrapper as DataWrapper>::Data as Collection>::Collected>,
         Error = AkcClientError> + std::marker::Send>
                 where Wrapper: PaginatedWrapper + DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: Collection + 'static,
    {
        let page_count = 100;
        let stream = stream::unfold(PageInformation {
            offset: 0,
            last_page_count: page_count,
        }, move |state| if state.last_page_count < page_count {
            None
        } else {
            let fut = Self::get_paginated_with_pagination_info::<Wrapper>(from.clone(),
                                                                          url.clone(),
                                                                          state.offset,
                                                                          page_count);
            Some(fut)
        });

        stream
            .collect()
            .and_then(|pages| {
                          let mut acc = vec![];
                          for page in pages {
                              acc.extend(page.items());
                          }

                          Ok(acc)
                      })
            .boxed()
    }

    pub fn get_all_pages_async_parallel<Wrapper>
        (from: String,
         url: hyper::Url)
            -> Box<Future<Item = Vec<<<Wrapper as DataWrapper>::Data as Collection>::Collected>,
            Error = AkcClientError> + std::marker::Send>
        where Wrapper: PaginatedWrapper + DataWrapper,
              Wrapper: serde::de::DeserializeOwned,
              Wrapper::Data: Collection + 'static,
    {
        let page_count = 100;
        let page_0_future = Self::get_paginated_with_params::<Wrapper>(from.clone(),
                                                                       url.clone(),
                                                                       vec![("offset".to_string(),
                                                            "0".to_string()),
                                                           ("count".to_string(),
                                                            page_count.to_string())]);
        page_0_future
            .and_then(move |page_0| {
                let mut future_pages = vec![];
                for page in 0..(page_0.1 / page_count) {
                    let params = vec![("offset".to_string(),
                                       ((page + 1) * page_count).to_string()),
                                      ("count".to_string(), page_count.to_string())];
                    future_pages.push(Self::get_paginated_with_params::<Wrapper>(from.clone(),
                                                                                 url.clone(),
                                                                                 params)
                                              .and_then(move |page| Ok(page.0)))
                }
                join_all(future_pages).map(move |pages| (page_0.0, pages))
            })
            .and_then(|pages| {
                let mut acc = vec![];
                acc.extend(pages.0.items());
                for page in pages.1 {
                    acc.extend(page.items());
                }

                Ok(acc)
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

pub struct PageInformation {
    offset: u32,
    last_page_count: u32,
}

pub trait PaginatedWrapper {
    type Collected: std::marker::Send;
    fn total(self: &Self) -> u32;
    fn count(self: &Self) -> u32;
}
pub trait Collection {
    type Collected: std::marker::Send;
    fn items(self: Self) -> Vec<Self::Collected>;
}

macro_rules! paginated_wrapper {
    ( $w:ident, $d:ident, $n:ident, $i:ident ) => {
        #[derive(Deserialize, Serialize, Debug, Clone)]
        struct $w {
            data: $d,
            total: u32,
            count: u32,
        }

        impl helpers::DataWrapper for $w {
            type Data = $d;
            fn data(self: Self) -> Self::Data {
                self.data
            }
        }

        impl helpers::PaginatedWrapper for $w {
            type Collected = $i;
            fn total(self: &Self) -> u32 {
                self.total
            }
            fn count(self: &Self) -> u32 {
                self.count
            }
        }

        #[derive(Deserialize, Serialize, Debug, Clone)]
        #[serde(rename_all = "camelCase")]
        pub struct $d {
            $n: Vec<$i>,
        }

        impl helpers::Collection for $d {
            type Collected = $i;
            fn items(self: Self) -> Vec<Self::Collected> {
                self.$n
            }
        }
    }
}
