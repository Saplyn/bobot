use serde::Serialize;
use serde_repr::Serialize_repr as SerializeRepr;

use crate::bot::{
    BotClient, BotClientError,
    messaging::{Ark, Keyboard, Markdown, Media, MessageReference},
};

#[derive(Debug)]
pub enum DirectMessageContent {
    PlainText(String),
    Markdown {
        markdown: Markdown,
        keyboard: Option<Keyboard>,
    },
    Ark(Ark),
    Media(Media),
}

#[derive(Debug)]
pub struct DirectMessage {
    pub content: DirectMessageContent,
    pub message_reference: Option<MessageReference>,
    pub event_id: Option<DirectMessageEvent>,
    pub msg_marker: Option<DirectMessageMarker>,
}

impl Serialize for DirectMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Default, SerializeRepr)]
        #[repr(u8)]
        enum DirectMessageTypeSer {
            #[default]
            PlainText = 0,
            Markdown = 2,
            Ark = 3,
            Media = 7,
        }
        #[derive(Default, Serialize)]
        struct DirectMessageSer<'s> {
            msg_type: DirectMessageTypeSer,
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<&'s str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            markdown: Option<&'s Markdown>,
            #[serde(skip_serializing_if = "Option::is_none")]
            keyboard: Option<&'s Keyboard>,
            #[serde(skip_serializing_if = "Option::is_none")]
            ark: Option<&'s Ark>,
            #[serde(skip_serializing_if = "Option::is_none")]
            media: Option<&'s Media>,
            #[serde(skip_serializing_if = "Option::is_none")]
            message_reference: Option<&'s MessageReference>,
            #[serde(skip_serializing_if = "Option::is_none")]
            event_id: Option<DirectMessageEvent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg_id: Option<&'s str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            msg_req: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            is_wakeup: Option<bool>,
        }

        let mut ser = DirectMessageSer::default();
        let Self {
            content,
            message_reference,
            event_id,
            msg_marker,
        } = self;
        match content {
            DirectMessageContent::PlainText(content) => {
                ser.msg_type = DirectMessageTypeSer::PlainText;
                ser.content = Some(content.as_str());
            }
            DirectMessageContent::Markdown { markdown, keyboard } => {
                ser.msg_type = DirectMessageTypeSer::Markdown;
                ser.markdown = Some(markdown);
                ser.keyboard = keyboard.as_ref();
            }
            DirectMessageContent::Ark(ark) => {
                ser.msg_type = DirectMessageTypeSer::Ark;
                ser.ark = Some(ark);
            }
            DirectMessageContent::Media(media) => {
                ser.msg_type = DirectMessageTypeSer::Media;
                ser.media = Some(media);
            }
        }
        ser.message_reference = message_reference.as_ref();
        ser.event_id = event_id.to_owned();
        match msg_marker {
            Some(DirectMessageMarker::Reply { msg_id, msg_req }) => {
                ser.msg_id = msg_id.as_ref().map(|s| s.as_str());
                ser.msg_req = msg_req.to_owned();
            }
            Some(DirectMessageMarker::WakeUpRecall) => {
                ser.is_wakeup = Some(true);
            }
            None => {}
        }

        ser.serialize(serializer)
    }
}

#[derive(Debug)]
pub enum DirectMessageMarker {
    Reply {
        msg_id: Option<String>,
        msg_req: Option<u32>,
    },
    WakeUpRecall,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DirectMessageEvent {
    #[serde(rename = "FRIEND_ADD")]
    ContactAdd,
    #[serde(rename = "C2C_MSG_RECEIVE")]
    DirectMessageReceive,
    #[serde(rename = "INTERACTION_CREATE")]
    InteractionCreate,
}

impl BotClient {
    pub async fn direct_message_to(
        &self,
        user_openid: &str,
        message: &DirectMessage,
    ) -> Result<reqwest::Response, BotClientError> {
        let url = format!("{}/v2/users/{user_openid}/messages", Self::OPENAPI_URL);

        let resp = self
            .req_client
            .post(url)
            .header(
                "Authorization",
                format!("QQBot {}", self.access_token().await?),
            )
            .json(message)
            .send()
            .await?;

        Ok(resp)
    }
}
