## Problem Statement
Streamers currently only see dummy placeholder messages in their OBS widget. They need the widget to display real-time chat messages from their actual Twitch, YouTube, and Kick channels so they can interact with their audience effectively.

## Solution
Connect the system to the real chat feeds of Twitch, YouTube, and Kick. When a streamer opens their widget via its unique URL in OBS, the Rust backend will look up their connected platform accounts, authenticate with those platforms using securely stored tokens, and stream a unified, normalized feed of chat messages directly to the widget in real-time.

## User Stories
1. As a streamer, I want to securely connect my Twitch, YouTube, and Kick accounts in the dashboard, so that the app has permission to read my chats.
2. As a streamer, I want my OBS widget to only connect to my chat platforms when it is actively running in OBS, so that my stream's chat is instantly available but doesn't waste resources when I am offline.
3. As a streamer, I want chat messages from all connected platforms to appear in a unified, consistent format in the widget, so that I can easily read chat without worrying about which platform it came from.
4. As an application, I want to ensure third-party OAuth tokens are encrypted at rest and decrypted only in-memory by the backend, so that sensitive user data is protected.

## Implementation Decisions
- **Token Storage and Access**: Next.js will handle the dashboard UI for users to connect platforms via OAuth. The third-party OAuth tokens will be encrypted and stored in the Supabase PostgreSQL database.
- **Direct Database Connection in Rust**: The Rust backend will establish its own database connection pool using `sqlx` (configured via a `DATABASE_URL` environment variable). This allows the backend to retrieve and decrypt a user's platform tokens using only the public Widget ID.
- **Connection Lifecycle (On-Demand)**: The Rust backend will NOT keep persistent connections to external platforms indefinitely. It will spawn asynchronous `tokio` ingestion tasks for Twitch, YouTube, and Kick only when a widget WebSocket actively connects. When the widget WebSocket disconnects, the backend will gracefully terminate these platform connections.
- **Message Normalization**: The Rust backend will parse the disparate event formats from Twitch (IRC/WS), YouTube (API/WS), and Kick (WS) into a single, unified JSON schema. This schema will include standard fields like `platform`, `username`, `avatar_url`, `badges`, and `message`. This normalized payload will be sent to the Next.js widget, keeping the frontend simple.

## Testing Decisions
- **Unit Tests**: Test the message normalization logic in Rust to ensure Twitch, YouTube, and Kick events correctly map to the unified schema.
- **Integration Tests (Rust)**: Use `sqlx` against a test database to verify token retrieval and decryption logic works using a test widget ID.
- **Mocking**: Mock the external platform WebSocket connections to simulate incoming chat events. Verify that the Rust backend correctly initiates and tears down `tokio` tasks based on widget connect/disconnect events.

## Out of Scope
- Implementing chat reply or moderation features directly from the widget or dashboard.
- Advanced custom emote parsing (beyond standard text/platform defaults).
- Handling authentication for platforms other than Twitch, YouTube, and Kick.
- Sending data back to external platforms (read-only architecture).

## Further Notes
- The master decryption key for the OAuth tokens must be securely passed to the Rust service via an environment variable.
