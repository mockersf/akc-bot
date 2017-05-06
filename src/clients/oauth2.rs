use std::time::{Duration, SystemTime};
use std::fmt;

use hyper;
use hyper::{Url, Client};
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use hyper::header::{Headers, Authorization, Basic, Bearer, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};
use hyper::status::StatusCode;
use iron;
use serde_json;

#[derive(Default, Debug, Clone)]
pub struct Token {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<SystemTime>,
}
impl Token {
    fn new(tokens: ExternalToken) -> Token {
        Token {
            access_token: tokens.access_token,
            refresh_token: Some(tokens.refresh_token),
            expires_at: Some(SystemTime::now() + Duration::from_secs(tokens.expires_in)),
        }
    }
    pub fn from_access_token(access_token: String) -> Token {
        Token {
            access_token,
            ..Default::default()
        }
    }

    pub fn access_token(self: &Self) -> &str {
        &self.access_token
    }

    pub fn bearer(self: &Self) -> hyper::header::Authorization<hyper::header::Bearer> {
        Authorization(Bearer { token: self.access_token.to_owned() })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ExternalToken {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum Oauth2ErrorMessage {
    InvalidRequest,
    InvalidClient,
    InvalidGrant,
    UnauthorizedClient,
    UnsupportedGrantType,
    InvalidScope,
}
impl fmt::Display for Oauth2ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Deserialize, Debug)]
pub struct Oauth2Error {
    error: Oauth2ErrorMessage,
    error_description: Option<String>,
    error_uri: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    CommunicationError(String),
    Oauth2Error(Oauth2Error),
}
impl From<hyper::Error> for Error {
    fn from(err: hyper::Error) -> Error {
        Error::CommunicationError(format!("error communicating with server: {:?}", err))
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::CommunicationError(format!("error parsing response: {:?}", err))
    }
}
impl From<Oauth2Error> for Error {
    fn from(err: Oauth2Error) -> Error {
        Error::Oauth2Error(err)
    }
}

pub trait Grant {}
pub struct AuthorizationCode {
    pub code: String,
}
impl Grant for AuthorizationCode {}
impl fmt::Display for AuthorizationCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "grant_type=authorization_code&code={}", self.code)
    }
}

pub struct Oauth2 {
    app_id: String,
    app_secret: String,
    token_url: Url,
}
impl Oauth2 {
    pub fn new(app_id: String,
               app_secret: String,
               token_url: &str)
               -> Result<Oauth2, iron::url::ParseError> {
        Ok(Oauth2 {
               app_id,
               app_secret,
               token_url: try!(Url::parse(token_url)),
           })
    }
    pub fn exchange_token<T>(self: Self, grant: T) -> Result<Token, Error>
        where T: Grant + fmt::Display
    {
        let mut headers = Headers::new();
        headers.set(Authorization(Basic {
                                      username: self.app_id,
                                      password: Some(self.app_secret),
                                  }));
        headers.set(ContentType(Mime(TopLevel::Application, SubLevel::WwwFormUrlEncoded, vec![])));

        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);
        let response = client
            .post(self.token_url)
            .headers(headers)
            .body(&grant.to_string())
            .send()?;
        match StatusCode::from_u16(response.status_raw().0) {
            StatusCode::Ok => {
                let token: Result<ExternalToken, serde_json::Error> =
                    serde_json::from_reader(response);
                match token {
                    Ok(data) => Ok(Token::new(data)),
                    Err(error) => Err(error)?,
                }
            }
            _ => {
                let token: Result<Oauth2Error, serde_json::Error> =
                    serde_json::from_reader(response);
                match token {
                    Ok(error) => Err(error)?,
                    Err(error) => Err(error)?,
                }
            }
        }
    }
}
