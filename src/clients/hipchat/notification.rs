#[derive(Deserialize, Debug, Clone)]
struct User {
    name: String,
    id: u32,
    mention_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Room {
    pub name: String,
    pub id: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Message {
    pub message: String,
    id: String,
    from: User,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub room: Option<Room>,
    pub message: Option<Message>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Notification {
    pub event: EventType,
    pub item: Item,
    pub oauth_client_id: String,
    pub webhook_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    RoomMessage,
}
