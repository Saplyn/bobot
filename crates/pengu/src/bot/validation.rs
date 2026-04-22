use ed25519_dalek::Signer;

use crate::bot::BotClient;

impl BotClient {
    pub const HEADER_SIGNATURE_STRING: &'static str = "X-Signature-Ed25519";
    pub const HEADER_SIGNATURE_TIMESTAMP: &'static str = "X-Signature-Timestamp";
    pub fn validate_signature(&self, message: &[u8], signature: &str) -> bool {
        self.compute_signature(message) == signature
    }
    pub fn compute_signature(&self, message: &[u8]) -> String {
        let signature = self.signing_key.sign(message);
        format!("{:x}", signature)
    }
}
