use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum MessageFragment {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "emote")]
    Emote { text: String, emote_id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub r#type: String,
    pub widget_id: Option<String>,
    pub platform: Option<String>,
    pub author: String,
    pub avatar_url: Option<String>,
    pub color: Option<String>,
    pub badges: Option<Vec<String>>,
    pub content: String,
    pub fragments: Vec<MessageFragment>,
}

impl ChatMessage {
    pub fn mock(widget_id: &str) -> Self {
        Self {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            r#type: "chat_message".to_string(),
            widget_id: Some(widget_id.to_string()),
            platform: Some("mock".to_string()),
            author: "MockBot".to_string(),
            avatar_url: None,
            color: Some("#ff0000".to_string()),
            badges: Some(vec![]),
            content: "This is a dummy message".to_string(),
            fragments: vec![MessageFragment::Text {
                text: "This is a dummy message".to_string(),
            }],
        }
    }
}
