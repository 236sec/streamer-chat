# Feature Spec — 07: Kick Chat Ingestion

## Problem Statement
The chat stream application needs to display real-time chat messages from Kick on the streamer's OBS widget. Kick doesn't offer an official OAuth API through Supabase, but provides a public Pusher WebSocket for chat that requires only a chatroom ID.

## Solution
Implement a Kick chat WebSocket client in the Rust backend. The streamer enters their Kick channel username on the dashboard. On widget connect, the backend resolves the chatroom ID via Kick's public API, connects to the Pusher WebSocket, subscribes to the chatroom channel, and normalizes incoming messages into the existing `ChatMessage` schema.

## Language
- **Kick channel username** — the streamer's Kick username (e.g. `xqc`), entered manually on the accounts page. Not an OAuth token.
- **Chatroom ID** — numeric ID returned by `https://kick.com/api/v2/channels/{username}` in `chatroom.id`. Required to subscribe to the Pusher WebSocket channel.
- **Pusher WebSocket** — Kick's public chat WebSocket at `wss://ws-us2.pusher.com/app/eb1d5f283081a78b932c`. Subscribes to `channel.{chatroom_id}` to receive chat events.

## User Stories
1. As a streamer, I want to enter my Kick channel username and see Kick chat messages on my OBS widget alongside Twitch and YouTube messages.
2. As a system operator, I want Kick connections to use the public WebSocket (no stored secrets needed beyond the username).

## Implementation Decisions
- **D2 — Authentication:** No OAuth. The accounts page shows a text input for Kick username instead of an OAuth button. The username is stored in `platform_tokens.encrypted_token` (encrypted for code path consistency, even though it's not a secret).
- **Independent tasks:** Same pattern as Twitch and YouTube — `tokio::spawn` as an independent task.
- **On-demand lifecycle:** Connect to the Pusher WebSocket only when a widget is active. Tear down when the widget disconnects.

## Implementation Steps

### Frontend
1. **Accounts page:** Replace the Kick OAuth button with a text input + save button. On save, encrypt the username and upsert it into `platform_tokens` with `platform = 'kick'`.
2. **API route or client-side save:** Encrypt and save the Kick username to `platform_tokens` (reuse the same encryption logic from the OAuth callback).

### Backend
3. **Backend `kick.rs`:** Create `spawn_kick_client(widget_id, kick_username, tx)` with:
   - Resolve chatroom ID via `GET https://kick.com/api/v2/channels/{username}`
   - Connect to Pusher WebSocket (`wss://ws-us2.pusher.com/app/eb1d5f283081a78b932c?protocol=7`)
   - Send Pusher subscribe message for `chatrooms.{chatroom_id}.v2`
   - Parse incoming `App\Events\ChatMessageEvent` payloads
   - On-demand lifecycle tied to widget receiver count
4. **Backend `normalize.rs`:** Kick normalization already exists — verify it handles the Pusher event payload shape from the real WebSocket.
5. **Backend `state.rs`:** Wire `spawn_kick_client` alongside Twitch and YouTube. Decrypt the stored username and pass it to the spawn function.
6. **Tests:** Unit test for Kick message normalization. Integration test with mock Pusher WebSocket.

## Testing Decisions
- **Unit tests:** Test Kick Pusher event payload parsing into `ChatMessage` schema.
- **Integration tests:** Mock Pusher WebSocket server. Verify connection, subscription, message parsing, and teardown lifecycle.

## Out of Scope
- Kick subscriber/gifted sub events
- Kick emote rendering (use text fallback initially)
- Chat message sending/replying
