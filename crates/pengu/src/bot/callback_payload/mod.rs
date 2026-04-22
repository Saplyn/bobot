use serde::Deserialize;
use serde_repr::Deserialize_repr as DeserializeRepr;

use crate::bot::callback_payload::{dispatch::DispatchData, validation::ValidationData};

pub mod dispatch;
pub mod validation;

impl CallbackPayload {
    pub fn valid(&self) -> bool {
        if matches!(self.data, CallbackData::Arbitrary(_)) {
            return true; // FIXME:
        }
        match self.op_code {
            OpCode::Dispatch => matches!(self.data, CallbackData::Dispatch(_)),
            OpCode::Heartbeat => false,           // FIXME:
            OpCode::Identify => false,            // FIXME:
            OpCode::Resume => false,              // FIXME:
            OpCode::Reconnect => false,           // FIXME:
            OpCode::InvalidSession => false,      // FIXME:
            OpCode::Hello => false,               // FIXME:
            OpCode::HeartbeatAck => false,        // FIXME:
            OpCode::CallbackAcknowledge => false, // FIXME:
            OpCode::CallbackValidation => matches!(self.data, CallbackData::CallbackValidation(_)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CallbackPayload {
    #[serde(rename = "op")]
    pub op_code: OpCode,
    #[serde(rename = "id")]
    pub event_id: Option<String>,
    #[serde(rename = "d")]
    pub data: CallbackData,
    #[serde(rename = "s")]
    pub serial: Option<usize>,
    #[serde(rename = "t")]
    pub event_type: Option<EventType>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, DeserializeRepr)]
#[repr(u8)]
pub enum OpCode {
    Dispatch = 0,
    Heartbeat = 1,
    Identify = 2,
    Resume = 6,
    Reconnect = 7,
    InvalidSession = 9,
    Hello = 10,
    HeartbeatAck = 11,
    CallbackAcknowledge = 12,
    CallbackValidation = 13,
}

#[derive(Debug, Deserialize)]
pub enum EventType {
    #[serde(rename = "FRIEND_ADD")]
    ContactAdd,
    #[serde(rename = "FRIEND_DEL")]
    ContactDel,
    #[serde(rename = "C2C_MESSAGE_CREATE")]
    DirectMessage,
    #[serde(rename = "C2C_MSG_RECEIVE")]
    DirectMessageReceive,
    #[serde(rename = "C2C_MSG_REJECT")]
    DirectMessageReject,
    #[serde(rename = "GROUP_AT_MESSAGE_CREATE")]
    GroupMention,
    #[serde(rename = "GROUP_ADD_ROBOT")]
    GroupJoin,
    #[serde(rename = "GROUP_DEL_ROBOT")]
    GroupLeave,
    #[serde(rename = "GROUP_MSG_RECEIVE")]
    GroupMessageReceive,
    #[serde(rename = "GROUP_MSG_REJECT")]
    GroupMessageReject,
    #[serde(rename = "SUBSCRIBE_MESSAGE_STATUS")]
    GroupSubscribeChange,
    #[serde(rename = "INTERACTION_CREATE")]
    InteractionCreate,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CallbackData {
    Dispatch(DispatchData),
    // Heartbeat(()),
    // Identify(()),
    // Resume(()),
    // Reconnect(()),
    // InvalidSession(()),
    // Hello(()),
    // HeartbeatAck(()),
    // CallbackAcknowledge(()),
    CallbackValidation(ValidationData),
    Arbitrary(serde_json::Value),
}
