use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub r#type: String,
    pub widget_id: Option<String>,
    pub platform: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub badges: Option<Vec<String>>,
    pub message: Option<String>,
}

impl ChatMessage {
    pub fn mock(widget_id: &str) -> Self {
        Self {
            r#type: "mock_message".to_string(),
            widget_id: Some(widget_id.to_string()),
            platform: Some("mock".to_string()),
            username: Some("MockBot".to_string()),
            avatar_url: None,
            badges: Some(vec![]),
            message: Some("This is a dummy message".to_string()),
        }
    }
}
