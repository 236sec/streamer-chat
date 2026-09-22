## Problem Statement
The chat stream application needs to display real-time chat messages from Twitch on the streamer's OBS widget. Currently, the system can accept connections and broadcast dummy messages, but it lacks integration with actual Twitch chat.

## Solution
Implement an on-demand Twitch EventSub WebSocket client in the Rust backend. When a streamer's OBS widget connects, the backend will spawn a task to connect to Twitch EventSub, listen for channel chat messages, normalize them into a unified, fragment-based format, and broadcast them directly to the widget.

## User Stories
1. As a streamer, I want my live Twitch chat to appear on my OBS widget automatically when the widget is active, so that my viewers can see their messages on stream.
2. As a system operator, I want Twitch connections to only be active when the widget is live, so that we conserve API quotas and server resources.
3. As a frontend developer, I want Twitch emotes and text to be pre-parsed into a unified fragment schema, so that I can render them easily without platform-specific logic.

## Implementation Decisions
- **Protocol (D1):** Use Twitch EventSub WebSockets instead of Twitch IRC, leveraging its modern JSON payloads and official support.
- **Lifecycle (D2):** Use an on-demand connection lifecycle. A Tokio task for the EventSub connection is spawned only when the user's widget establishes an active WebSocket connection to the Rust backend, and torn down when the widget disconnects.
- **Schema (D3):** Define the `ChatMessage` schema's content as an array of `MessageFragment` objects (e.g., text or emote). The Rust backend parses Twitch fragments into this schema to avoid HTML sanitization risks and keep the backend presentation-agnostic.

## Testing Decisions
- **Unit Tests:** Test the EventSub JSON payload parsing into the `ChatMessage` schema, covering both text-only and emote-included messages.
- **Integration Tests:** Mock the Twitch EventSub WebSocket endpoint to verify the on-demand Tokio task spawn and teardown lifecycle upon widget connection/disconnection.

## Out of Scope
- YouTube and Kick chat ingestion (handled in subsequent tickets).
- Chat message sending/replying (read-only for now).
- Complex moderation events (bans, timeouts).

## Further Notes
- Ensure the decrypted OAuth token is safely held only in memory for the duration of the EventSub connection and dropped immediately.
