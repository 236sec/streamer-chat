# Feature Spec — 06: YouTube Chat Ingestion

## Problem Statement
The chat stream application needs to display real-time chat messages from YouTube Live on the streamer's OBS widget. The Twitch integration is working, but YouTube has no implementation yet. YouTube uses a polling API (not WebSocket), and the streamer may open the widget before going live.

## Solution
Implement a YouTube Live Chat polling client in the Rust backend. When a widget connects and the user has a YouTube token, spawn an independent task that polls `liveBroadcasts.list` every 30 seconds until an active broadcast is found, then switches to polling `liveChatMessages.list` at the interval specified by the API. Messages are normalized into the existing `ChatMessage` + `MessageFragment` schema and broadcast to the widget. Token refresh is handled automatically on 401.

## Language
- **Broadcast discovery loop** — the 30-second polling loop that checks `liveBroadcasts.list?broadcastStatus=active` for an active stream before chat polling can begin.
- **Chat polling loop** — once a `liveChatId` is found, the loop that calls `liveChatMessages.list` at the `pollingIntervalMillis` returned by YouTube's API.
- **Token refresh** — when the YouTube API returns 401, use the stored `provider_refresh_token` to obtain a new access token from Google's token endpoint, update it in memory, and retry.

## User Stories
1. As a streamer, I want YouTube Live chat messages to appear on my OBS widget alongside Twitch messages, without any manual configuration beyond connecting my Google account.
2. As a streamer, I want to open my widget before going live on YouTube and have chat messages appear automatically once my broadcast starts.
3. As a system operator, I want YouTube polling to respect the API's `pollingIntervalMillis` to avoid rate limiting.

## Implementation Decisions
- **D1 — Broadcast discovery:** Poll `liveBroadcasts.list` every 30 seconds until an active broadcast with a `liveChatId` is found. If the broadcast ends, return to discovery polling.
- **D3 — Token refresh:** Save Google's `provider_refresh_token` in a new `encrypted_refresh_token` column in `platform_tokens`. The backend refreshes the access token on 401 by calling Google's token endpoint with the refresh token.
- **D4 — Active live chat fallback:** Prioritize broadcasts with `status.lifeCycleStatus == "live"`. If `snippet.liveChatId` is missing or fails, query `videos.list?part=liveStreamingDetails` for `activeLiveChatId`.
- **D5 — Live chat startup 404 retry:** YouTube live chat takes 5-15 seconds to spin up when a stream goes live. The polling loop retries 404 responses up to 5 times with a 2-second backoff before dropping back to the 30-second discovery loop.
- **D6 — Worker lifecycle on reconnect:** Maintain an active worker registry `widget_workers` in `AppState`. When a client reconnects or reloads, automatically re-spawn background ingestion workers if previous workers terminated.
- **Independent tasks:** Each platform client (Twitch, YouTube, Kick) must run as its own `tokio::spawn` task. Fix the current `state.rs` which blocks on Twitch before any other platform could start.

## Implementation Steps

### Prerequisites (shared with Kick)
1. **Migration:** Add `encrypted_refresh_token TEXT` column to `platform_tokens` table.
2. **Frontend callback:** Update `auth/callback/route.ts` to also encrypt and save `session.provider_refresh_token` when present.
3. **Backend db.rs:** Update `PlatformTokens` struct and `fetch_tokens` to include refresh tokens.
4. **Backend state.rs:** Refactor `get_or_create_channel` to spawn each platform client as an independent `tokio::spawn` task (not `.await` sequentially).

### YouTube-specific
5. **Backend `youtube.rs`:** Create `spawn_youtube_client(widget_id, token, refresh_token, tx)` with:
   - Broadcast discovery loop (30s interval)
   - Chat polling loop (respecting `pollingIntervalMillis`)
   - Token refresh on 401
   - `pageToken` tracking to avoid duplicate messages
6. **Backend `normalize.rs`:** YouTube normalization already exists — verify it handles the `liveChatMessages.list` response shape.
7. **Backend `state.rs`:** Wire `spawn_youtube_client` alongside Twitch in the channel setup.
8. **Tests:** Unit test for YouTube message normalization. Integration test with mock HTTP server for broadcast discovery + chat polling lifecycle.

## Testing Decisions
- **Unit tests:** Test YouTube `liveChatMessages` payload parsing into `ChatMessage` schema.
- **Integration tests:** Mock HTTP server returning broadcast list, then chat messages. Verify the polling lifecycle, including the transition from discovery to chat polling.

## Out of Scope
- YouTube Super Chat / Super Sticker rendering
- YouTube membership events
- Chat message sending/replying
