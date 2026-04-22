use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ValidationData {
    pub plain_token: String,
    pub event_ts: String,
}

impl ValidationData {
    pub fn bytes_iter(&self) -> impl Iterator<Item = u8> {
        self.event_ts
            .as_bytes()
            .iter()
            .copied()
            .chain(self.plain_token.as_bytes().iter().copied())
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub plain_token: String,
    pub signature: String,
}
