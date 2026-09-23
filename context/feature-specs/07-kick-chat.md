## Problem Statement
Streamers want their Kick chat messages to appear in their unified OBS widget alongside Twitch and YouTube, but currently, only Twitch and YouTube are supported. The system needs to ingest Kick's real-time chat data to provide a complete multi-streaming chat experience.

## Solution
We will connect the backend to Kick's real-time chat service using their underlying Pusher WebSockets. The system will dynamically resolve the streamer's Kick username to their internal chatroom ID, establish a persistent connection, and seamlessly merge Kick messages into the unified chat feed displayed on their widget.

## User Stories
1. As a streamer, I want my Kick chat messages to appear in the OBS widget in real time, so that I can see all my audiences in one place.
2. As a streamer, I want the system to automatically recover from temporary Kick connection drops, so that my chat widget remains reliable without manual intervention.
3. As a streamer, I want my Kick messages to look consistent with messages from other platforms in the widget, so that my chat overlay looks cohesive on stream.

## Implementation Decisions
- **Kick API Client**: We will introduce a lightweight HTTP client to call Kick's public API (`https://kick.com/api/v1/channels/{username}`) on worker startup to resolve the internal `chatroom_id`.
- **WebSocket Client**: Instead of relying on a third-party Pusher crate, we will implement a custom, lightweight Pusher client over `tokio-tungstenite`. This client will handle the Pusher protocol (connecting, sending the JSON subscribe event to `chatrooms.{chatroom_id}.v2`, and responding to ping/pong keep-alives).
- **Message Parsing**: We will map Kick's specific JSON chat payload (typically found inside the `App\Events\ChatMessageEvent` Pusher event) into our existing normalized `ChatMessage` schema.
- **Worker Lifecycle**: The Kick chat consumer will run as an isolated Tokio task per active widget connection. It will include an exponential backoff loop to handle network failures or Kick service interruptions, matching the reliability patterns established for YouTube and Twitch workers.
- **Message Routing**: Parsed `ChatMessage` objects will be published to the existing Redis Pub/Sub infrastructure, ensuring the frontend widget receives them identically to messages from other platforms.

## Testing Decisions
- **API Mocking**: We will use existing HTTP mocking infrastructure (e.g., `wiremock`) to simulate the Kick public API for `chatroom_id` resolution.
- **Unit Tests**: We will write unit tests for the message parser to ensure Kick's Pusher event JSON is correctly transformed into the `ChatMessage` struct, including edge cases like missing fields or unexpected event types.
- **Integration Tests**: We will utilize the existing worker test harness to verify the full lifecycle: resolving the room ID, establishing the WebSocket connection, parsing a simulated message, and successfully publishing to Redis.

## Out of Scope
- Kick chat moderation actions (deleting messages, timing out users).
- Kick-specific emotes parsing (beyond standard text representation if applicable).
- Polling as a fallback if Kick changes their Pusher implementation (we will strictly rely on WebSockets for this iteration).

## Further Notes
- Kick's API and Pusher implementation are technically undocumented/unofficial. If Kick updates their schema or connection requirements, the parsing or subscription logic may need to be adjusted.
