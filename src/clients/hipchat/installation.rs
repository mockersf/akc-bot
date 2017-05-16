#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Installation {
    pub oauth_id: String,
    pub oauth_secret: String,
    pub room_id: i32,
    pub group_id: i32,
    pub capabilities_url: String,
}
