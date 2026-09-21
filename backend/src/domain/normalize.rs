use crate::domain::message::ChatMessage;
use serde_json::Value;

pub fn normalize_message(platform: &str, widget_id: &str, payload: &Value) -> Option<ChatMessage> {
    match platform {
        "twitch" => {
            // Very simplified mock for Twitch IRC/WS format
            let username = payload.get("source")?.get("nick")?.as_str()?;
            let message = payload.get("parameters")?.as_array()?.first()?.as_str()?;
            Some(ChatMessage {
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("twitch".to_string()),
                username: Some(username.to_string()),
                avatar_url: None, // Simplified
                badges: Some(vec![]),
                message: Some(message.to_string()),
            })
        }
        "youtube" => {
            // Simplified YouTube format
            let snippet = payload.get("snippet")?;
            let author = payload.get("authorDetails")?;
            let username = author.get("displayName")?.as_str()?;
            let message = snippet.get("displayMessage")?.as_str()?;
            Some(ChatMessage {
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("youtube".to_string()),
                username: Some(username.to_string()),
                avatar_url: author
                    .get("profileImageUrl")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                badges: Some(vec![]),
                message: Some(message.to_string()),
            })
        }
        "kick" => {
            // Simplified Kick format
            let sender = payload.get("sender")?;
            let username = sender.get("username")?.as_str()?;
            let message = payload.get("content")?.as_str()?;
            Some(ChatMessage {
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("kick".to_string()),
                username: Some(username.to_string()),
                avatar_url: None,
                badges: Some(vec![]),
                message: Some(message.to_string()),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_twitch() {
        let payload = json!({
            "source": { "nick": "twitch_user" },
            "parameters": ["hello twitch"]
        });
        let msg = normalize_message("twitch", "widget_1", &payload).unwrap();
        assert_eq!(msg.username.unwrap(), "twitch_user");
        assert_eq!(msg.message.unwrap(), "hello twitch");
        assert_eq!(msg.platform.unwrap(), "twitch");
    }

    #[test]
    fn test_normalize_youtube() {
        let payload = json!({
            "snippet": { "displayMessage": "hello youtube" },
            "authorDetails": { "displayName": "yt_user", "profileImageUrl": "http://img" }
        });
        let msg = normalize_message("youtube", "widget_1", &payload).unwrap();
        assert_eq!(msg.username.unwrap(), "yt_user");
        assert_eq!(msg.message.unwrap(), "hello youtube");
        assert_eq!(msg.avatar_url.unwrap(), "http://img");
        assert_eq!(msg.platform.unwrap(), "youtube");
    }

    #[test]
    fn test_normalize_kick() {
        let payload = json!({
            "sender": { "username": "kick_user" },
            "content": "hello kick"
        });
        let msg = normalize_message("kick", "widget_1", &payload).unwrap();
        assert_eq!(msg.username.unwrap(), "kick_user");
        assert_eq!(msg.message.unwrap(), "hello kick");
        assert_eq!(msg.platform.unwrap(), "kick");
    }
}
