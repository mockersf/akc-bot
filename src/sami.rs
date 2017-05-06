use futures::Future;

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

pub fn generate_response(from: String, nlp_response: NlpResponse) -> MessageToUser {
    info!("{:?}", nlp_response);
    match nlp_response.intent {
        intent @ Intent::GetSelf => {
            MessageToUser {
                intent: intent,
                data: vec![::clients::akc::Akc::user_self(from)
                               .wait()
                               .unwrap()
                               .full_name],
                status: Status::Info,
            }
        }
        intent => {
            MessageToUser {
                intent,
                data: nlp_response.meta.unwrap_or(vec![]),
                status: Status::Error,
            }
        }
    }
}

#[derive(Debug)]
pub enum Intent {
    SetField,
    GetField,
    FindDeviceType,
    Logout,
    GetSelf,
    Unknown,
}

impl Default for Intent {
    fn default() -> Intent {
        Intent::Unknown
    }
}

#[derive(Default, Debug)]
pub struct NlpResponse {
    pub intent: Intent,
    pub device: Option<String>,
    pub value: Option<String>,
    pub field: Option<String>,
    pub meta: Option<Vec<String>>,
}
