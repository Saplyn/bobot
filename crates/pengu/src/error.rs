use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub struct QQErrorResp {
    pub code: u32,
    pub msg: String,
}

impl std::fmt::Display for QQErrorResp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.msg, self.code)
    }
}
