use futures::Future;

pub enum Status {
    Info,
    Confirmation,
    Error,
    ActionRequired,
}

pub struct MessageToUser {
    pub message: String,
    pub status: Status,
}
pub fn generate_response(from: String,
                         wit_ai_response: ::clients::witai::Response)
                         -> MessageToUser {
    info!("from: {:?}", from);
    info!("wit_ai_response: {:?}", wit_ai_response);
    MessageToUser {
        message: ::clients::akc::Akc::user_self(from)
            .wait()
            .unwrap()
            .full_name,
        status: Status::Info,
    }
}
