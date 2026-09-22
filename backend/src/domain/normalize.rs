use crate::domain::message::{ChatMessage, MessageFragment};
use serde_json::Value;

pub fn normalize_message(platform: &str, widget_id: &str, payload: &Value) -> Option<ChatMessage> {
    match platform {
        "twitch" => {
            // Updated to parse Twitch EventSub payload
            let event = payload.get("event")?;
            let author = event.get("chatter_user_name")?.as_str()?;
            let message_obj = event.get("message")?;
            let content = message_obj.get("text")?.as_str()?;
            let color = event
                .get("color")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut fragments = Vec::new();
            if let Some(frags) = message_obj.get("fragments").and_then(|v| v.as_array()) {
                for frag in frags {
                    if let Some(t) = frag.get("type").and_then(|v| v.as_str()) {
                        let text = frag
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if t == "emote" {
                            if let Some(emote) = frag.get("emote") {
                                if let Some(emote_id) = emote.get("id").and_then(|v| v.as_str()) {
                                    fragments.push(MessageFragment::Emote {
                                        text,
                                        emote_id: emote_id.to_string(),
                                    });
                                    continue;
                                }
                            }
                        }
                        fragments.push(MessageFragment::Text { text });
                    }
                }
            }

            Some(ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("twitch".to_string()),
                author: author.to_string(),
                avatar_url: None,
                color,
                badges: Some(vec![]),
                content: content.to_string(),
                fragments,
            })
        }
        "youtube" => {
            let snippet = payload.get("snippet")?;
            let author = payload.get("authorDetails")?;
            let author_name = author.get("displayName")?.as_str()?;
            let message = snippet
                .get("displayMessage")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    snippet
                        .get("textMessageDetails")
                        .and_then(|t| t.get("messageText"))
                        .and_then(|v| v.as_str())
                })?;
            Some(ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("youtube".to_string()),
                author: author_name.to_string(),
                avatar_url: author
                    .get("profileImageUrl")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                color: None,
                badges: Some(vec![]),
                content: message.to_string(),
                fragments: vec![MessageFragment::Text {
                    text: message.to_string(),
                }],
            })
        }
        "kick" => {
            let sender = payload.get("sender")?;
            let author_name = sender.get("username")?.as_str()?;
            let message = payload.get("content")?.as_str()?;
            Some(ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                r#type: "chat_message".to_string(),
                widget_id: Some(widget_id.to_string()),
                platform: Some("kick".to_string()),
                author: author_name.to_string(),
                avatar_url: None,
                color: None,
                badges: Some(vec![]),
                content: message.to_string(),
                fragments: vec![MessageFragment::Text {
                    text: message.to_string(),
                }],
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
            "event": {
                "chatter_user_name": "twitch_user",
                "color": "#FF0000",
                "message": {
                    "text": "hello Kappa",
                    "fragments": [
                        { "type": "text", "text": "hello " },
                        { "type": "emote", "text": "Kappa", "emote": { "id": "25" } }
                    ]
                }
            }
        });
        let msg = normalize_message("twitch", "widget_1", &payload).unwrap();
        assert_eq!(msg.author, "twitch_user");
        assert_eq!(msg.content, "hello Kappa");
        assert_eq!(msg.color.unwrap(), "#FF0000");
        assert_eq!(msg.platform.unwrap(), "twitch");

        let frags = msg.fragments;
        assert_eq!(frags.len(), 2);
        assert_eq!(
            frags[0],
            MessageFragment::Text {
                text: "hello ".to_string()
            }
        );
        assert_eq!(
            frags[1],
            MessageFragment::Emote {
                text: "Kappa".to_string(),
                emote_id: "25".to_string(),
            }
        );
    }

    #[test]
    fn test_normalize_youtube() {
        let payload = json!({
            "kind": "youtube#liveChatMessage",
            "id": "yt_msg_123",
            "snippet": {
                "type": "textMessageEvent",
                "displayMessage": "Hello YouTube chat!"
            },
            "authorDetails": {
                "displayName": "YT Viewer",
                "profileImageUrl": "https://yt3.ggpht.com/avatar.jpg"
            }
        });
        let msg = normalize_message("youtube", "widget_yt", &payload).unwrap();
        assert_eq!(msg.author, "YT Viewer");
        assert_eq!(msg.content, "Hello YouTube chat!");
        assert_eq!(msg.platform.unwrap(), "youtube");
        assert_eq!(msg.avatar_url.unwrap(), "https://yt3.ggpht.com/avatar.jpg");
        assert_eq!(
            msg.fragments,
            vec![MessageFragment::Text {
                text: "Hello YouTube chat!".to_string()
            }]
        );
    }
}
