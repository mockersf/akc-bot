use std::time::{Duration, SystemTime};

use hyper::Url;
use futures::future::*;

use clients::akc::Akc;
use clients::akc::error::AkcClientError;
use clients::akc::helpers;

#[derive(Debug, Clone)]
pub struct Token {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<SystemTime>,
}

impl Token {
    pub fn access_token(access_token: String) -> Token {
        Token {
            access_token,
            refresh_token: None,
            expires_at: None,
        }
    }

    pub fn expiring(access_token: String, expires_in: Duration) -> Token {
        Token {
            access_token,
            refresh_token: None,
            expires_at: Some(SystemTime::now() + expires_in),
        }
    }

    pub fn refreshable_token(access_token: String,
                             refresh_token: String,
                             expires_in: Duration)
                             -> Token {
        Token {
            access_token,
            refresh_token: Some(refresh_token),
            expires_at: Some(SystemTime::now() + expires_in),
        }
    }

    pub fn new(tokens: AkcToken) -> Token {
        Token {
            access_token: tokens.access_token,
            refresh_token: Some(tokens.refresh_token),
            expires_at: Some(SystemTime::now() + Duration::from_secs(tokens.expires_in)),
        }
    }
}

impl Token {
    pub fn bearer(self: &Self) -> &str {
        &self.access_token
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AkcToken {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
}

use hyper::header::{Headers, Authorization, Basic, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use clients::future_request;
use serde_json;

impl Akc {
    pub fn exchange_token(app_id: String,
                          app_secret: String,
                          code: String)
                          -> Box<Future<Item = Token, Error = AkcClientError>> {
        let url = Url::parse(&format!("{}/token", Self::accounts_url::<'static>())).unwrap();

        let mut headers = Headers::new();
        headers.set(Authorization(Basic {
                                      username: app_id,
                                      password: Some(app_secret),
                                  }));
        headers.set(ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![])));
        future_request::post_async::<::clients::akc::error::AkcClientError>(url,
                                                   headers,
                                                   format!("grant_type=authorization_code&code={}",
                                                           code))
                .and_then(|response| {
                    let token: Result<AkcToken, serde_json::Error> = serde_json::from_reader(response);
                    match token {
                        Ok(data) => Ok(Token::new(data)),
                        Err(error) => Err(error)?,
                    }
                          })
                .boxed()
    }
}
