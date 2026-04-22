//! `pengu` provides low-level types and client helpers for the QQ Open Platform.
//!
//! The crate currently exposes the `bot` module, which covers three main areas:
//!
//! - `BotClient`: token refresh, signature validation, and direct message sending.
//! - `callback_payload`: inbound callback and event payload models.
//! - `messaging`: outbound message payload models for direct messages.
//!
//! The `bot` feature is enabled by default.

#[cfg(feature = "bot")]
pub mod bot;
