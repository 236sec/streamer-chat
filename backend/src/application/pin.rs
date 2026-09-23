use crate::domain::message::ChatMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PinCommand {
    Pin { message: Box<ChatMessage> },
    Unpin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PinEvent {
    pub r#type: String,
    pub revision: i64,
    pub message: Option<ChatMessage>,
}

pub fn validate_command(command: &PinCommand, widget_id: &str) -> bool {
    match command {
        PinCommand::Unpin => true,
        PinCommand::Pin { message } => {
            message.r#type == "chat_message"
                && message.widget_id.as_deref() == Some(widget_id)
                && !message.id.trim().is_empty()
                && message.id.len() <= 256
                && !message.author.trim().is_empty()
                && message.author.len() <= 128
                && !message.content.trim().is_empty()
                && message.content.len() <= 2000
                && message.fragments.len() <= 100
                && message
                    .platform
                    .as_ref()
                    .is_some_and(|p| matches!(p.as_str(), "twitch" | "youtube" | "kick"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_accepts_normalized_message_for_widget() {
        let id = uuid::Uuid::new_v4().to_string();
        let mut message = ChatMessage::mock(&id);
        message.platform = Some("twitch".to_string());
        assert!(validate_command(
            &PinCommand::Pin {
                message: Box::new(message.clone())
            },
            &id
        ));
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &uuid::Uuid::new_v4().to_string()
        ));
    }

    #[test]
    fn pin_rejects_invalid_content_and_platform() {
        let id = uuid::Uuid::new_v4().to_string();
        let mut message = ChatMessage::mock(&id);
        message.content.clear();
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &id
        ));
        let message = ChatMessage::mock(&id);
        assert!(!validate_command(
            &PinCommand::Pin {
                message: Box::new(message)
            },
            &id
        ));
    }
}
