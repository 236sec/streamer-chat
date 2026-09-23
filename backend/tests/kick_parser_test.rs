use backend::domain::normalize::normalize_message;
use serde_json::json;

#[test]
fn test_normalize_kick_pusher_event() {
    let payload = json!({
        "event": "App\\Events\\ChatMessageEvent",
        "data": "{\"id\":\"msg-1\",\"chatroom_id\":123,\"content\":\"hello from kick\",\"sender\":{\"id\":1,\"username\":\"kick_user\",\"slug\":\"kick_user\"}}"
    });

    let msg = normalize_message("kick", "widget_kick", &payload).unwrap();
    assert_eq!(msg.author, "kick_user");
    assert_eq!(msg.content, "hello from kick");
    assert_eq!(msg.platform.unwrap(), "kick");
}
