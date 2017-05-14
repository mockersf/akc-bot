#[derive(Serialize, Debug, Clone)]
pub struct NotificationResponse {
    pub message: String,
    pub color: Color,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    Green,
    Yellow,
    Purple,
    Red,
}
