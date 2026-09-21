## Problem Statement
Streamers need a unique, customizable URL they can drop into OBS as a browser source to display a unified chat. Without this, there is no way for the real-time chat aggregated by the backend to be visible on their live stream.

## Solution
A public Next.js route `/widget/[id]` that fetches the streamer's widget configuration (e.g., theme, font size, background transparency) from Supabase. It connects via WebSocket to the Rust backend to receive real-time chat messages. For testing without real platform integrations, the backend will stream dummy messages when a mocking flag is enabled.

## User Stories
1. As a streamer, I want a unique, non-guessable widget URL so that my chat source is secure and only used by me.
2. As a streamer, I want my widget configuration (theme, font size, transparent background) to be saved in the database so that it automatically applies when OBS loads the browser source.
3. As a streamer, I want the widget to connect to the backend automatically and display incoming messages in real-time.
4. As a developer, I want the Rust backend to send dummy messages when a mocking flag is set, so that I can test the UI and WebSocket client without needing real Twitch/YouTube/Kick integrations.

## Implementation Decisions
- **Database Schema**: A `widgets` table in Supabase.
  - Columns: `id` (UUID, primary key), `user_id` (UUID, foreign key to auth.users), `theme` (string), `font_size` (string), `background_color` (string, supporting transparent values).
  - RLS: Public read access for the widget route using the `id`, and restricted updates to the authenticated owner.
- **Next.js Route**: `/app/widget/[id]/page.tsx` as a public route. It will fetch the widget configuration and apply the styling (e.g., transparent background for OBS).
- **WebSocket Client**: A custom React `useEffect` hook in the widget page to manage the native `WebSocket` connection, handle automatic reconnects, and maintain a rolling list of recent messages.
- **Mocking Flag**: The Rust backend will include a CLI flag or environment variable (e.g., `MOCK_CHAT=true`). When active, it will periodically broadcast dummy chat messages to connected WebSocket clients.

## Testing Decisions
- **Frontend**: Manual visual testing of the widget route simulating OBS (transparent background). Component tests for the chat message UI to ensure configuration styles apply correctly.
- **Backend (Rust)**: TDD for the mock message generator and WebSocket broadcasting logic to ensure the mock flag behaves correctly.

## Out of Scope
- Actual OAuth integration with Twitch/YouTube/Kick (handled in Ticket 04).
- The Next.js dashboard UI to edit the widget settings (this ticket only establishes the schema and the consumer widget).

## Further Notes
- Ensure the widget page explicitly supports and defaults to `bg-transparent` so OBS can seamlessly overlay it on the stream.
