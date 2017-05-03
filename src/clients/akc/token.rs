#[derive(Debug, Clone)]
pub struct Token {
    access_token: String,
}

impl Token {
    pub fn new(token: String) -> Token {
        Token { access_token: token }
    }
    pub fn bearer(self: &Self) -> &str {
        &self.access_token
    }
}
