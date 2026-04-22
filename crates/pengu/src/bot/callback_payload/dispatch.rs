use serde::Deserialize;
use serde_repr::Deserialize_repr as DeserializeRepr;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DispatchData {
    ContactAdd(ContactAddData),
    ContactDel(ContactDelData),
    DirectMessage(DirectMessageData),
    DirectMessageReceive(DirectMessageReceiveData),
    DirectMessageReject(DirectMessageRejectData),
    // GroupMention(GroupMentionData),
    // GroupJoin(GroupJoinData),
    // GroupLeave(GroupLeaveData),
    // GroupMessageReceive(GroupMessageReceiveData),
    // GroupMessageReject(GroupMessageRejectData),
    // GroupSubscribeChange(GroupSubscribeChangeData),
}

// LYN: Contact Add

#[derive(Debug, Deserialize)]
pub struct ContactAddData {
    pub timestamp: String,
    pub openid: String,
    pub scene: ContactAddScene,
    pub scene_param: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, DeserializeRepr)]
#[repr(u16)]
pub enum ContactAddScene {
    Default = 1000,
    SearchAll = 1001,
    SearchBot = 1002,
    GroupChat = 1003,
    QZone = 1004,
    ShareInternal = 2001,
    ShareExternal = 2002,
    DevShareInternal = 2003,
    DevShareExternal = 2004,
}

// LYN: Contact Delete

#[derive(Debug, Deserialize)]
pub struct ContactDelData {
    pub timestamp: String,
    pub openid: String,
}

// LYN: Direct Message Receive

#[derive(Debug, Deserialize)]
pub struct DirectMessageReceiveData {
    pub timestamp: String,
    pub openid: String,
}

// LYN: Direct Message Reject

#[derive(Debug, Deserialize)]
pub struct DirectMessageRejectData {
    pub timestamp: String,
    pub openid: String,
}

// LYN: Direct Message

#[derive(Debug, Deserialize)]
pub struct DirectMessageData {
    pub author: Author,
    pub content: String,
    pub id: String,
    pub timestamp: String,
    #[serde(default)]
    pub attachments: Vec<serde_json::Value>, // FIXME:
}

#[derive(Debug, Deserialize)]
pub struct Author {
    pub id: String,
    pub union_openid: String,
    pub user_openid: String,
}
