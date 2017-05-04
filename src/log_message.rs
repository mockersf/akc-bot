use std::fmt;
use serde_json;

#[derive(Debug, Serialize)]
pub struct LogMessage {
    details: String,
    url: Option<String>,
}

impl LogMessage {
    pub fn new(details: &str) -> LogMessage {
        LogMessage {
            details: details.to_string(),
            url: None,
        }
    }
}

impl fmt::Display for LogMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}
