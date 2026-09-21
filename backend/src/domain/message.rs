use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub r#type: String,
    pub widget_id: Option<String>,
    pub author: Option<String>,
    pub content: Option<String>,
}

impl ChatMessage {
    pub fn mock(widget_id: &str) -> Self {
        Self {
            r#type: "mock_message".to_string(),
            widget_id: Some(widget_id.to_string()),
            author: Some("MockBot".to_string()),
            content: Some("This is a dummy message".to_string()),
        }
    }
}
