use sami::{input, process, Intent};

#[derive(Debug)]
pub enum Status {
    Info,
    Confirmation,
    Error,
    ActionRequired,
}

#[derive(Debug)]
pub struct MessageToUser {
    pub intent: Intent,
    pub data: Vec<String>,
    pub status: Status,
}

impl MessageToUser {
    pub fn from(akc_token: ::clients::oauth2::Token, nlp_response: input::NlpResponse) -> MessageToUser {
        process::generate_response(akc_token, nlp_response)
    }
}