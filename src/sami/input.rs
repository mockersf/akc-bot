
#[derive(Default, Debug)]
pub struct NlpResponse {
    pub intent: ::sami::Intent,
    pub device: Option<Vec<String>>,
    pub value: Option<String>,
    pub field: Option<String>,
    pub meta: Option<Vec<String>>,
}
