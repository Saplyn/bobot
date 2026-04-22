use std::collections::HashMap;

use serde::Serialize;

pub mod direct_message;

#[derive(Debug)]
pub enum Markdown {
    Text {
        content: String,
    },
    Template {
        custom_template_id: String,
        params: HashMap<String, Vec<String>>,
    },
}

impl Serialize for Markdown {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct MarkdownSer<'s> {
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<&'s str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_template_id: Option<&'s str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            params: Option<Vec<MarkdownSerParam<'s>>>,
        }
        #[derive(Serialize)]
        struct MarkdownSerParam<'s> {
            key: &'s str,
            values: &'s [String],
        }

        let ser = match self {
            Markdown::Text { content } => MarkdownSer {
                content: Some(content.as_str()),
                custom_template_id: None,
                params: None,
            },
            Markdown::Template {
                custom_template_id,
                params,
            } => {
                let params = params
                    .iter()
                    .map(|(key, values)| MarkdownSerParam { key, values })
                    .collect();
                MarkdownSer {
                    content: None,
                    custom_template_id: Some(custom_template_id.as_str()),
                    params: Some(params),
                }
            }
        };

        ser.serialize(serializer)
    }
}

#[derive(Debug, Serialize)]
pub struct MessageReference {
    message_id: String,
    ignore_get_message_error: bool,
}

#[derive(Debug, Serialize)]
pub struct Ark {} // TODO:

#[derive(Debug, Serialize)]
pub struct Keyboard {} // TODO:

#[derive(Debug, Serialize)]
pub struct Media {} // TODO:
