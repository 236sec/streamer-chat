# Feature Spec — 10: Message Highlight Overlay & Dashboard Pin Control

## Problem Statement

The streamer can see chat in the OBS feed but cannot select one live message for a separate, prominent on-screen callout. A pinned message must remain visible when the highlight browser source opens or reconnects during a stream.

## Solution

Show the widget's incoming chat in an interactive dashboard feed. The streamer can pin one message at a time, replace that pin with another message, and clear it. A separate public highlight browser source displays the current pin with a transparent background and animated transitions. The dashboard offers its URL and OBS source size guidance.

## User Stories

1. As a streamer, I want to see my widget's live chat in the dashboard, so that I can choose a viewer message while streaming.
2. As a streamer, I want to pin a message from that feed, so that it appears in my separate OBS highlight source.
3. As a streamer, I want a new pin to replace the current pin, so that I can switch the featured comment without clearing it first.
4. As a streamer, I want to see which message is pinned and unpin it, so that I can remove it from the stream.
5. As a streamer, I want to copy the highlight URL and see recommended OBS dimensions, so that I can set up the separate browser source.
6. As an OBS viewer, I want the current pin to appear when the source loads or reconnects, so that a temporary disconnect does not lose the featured comment.
7. As an OBS viewer, I want pin, replacement, and unpin changes to animate smoothly, so that the callout does not jump abruptly.
8. As a streamer, I want pin controls restricted to my authenticated dashboard, so that someone with the public widget URL cannot change what appears on stream.

## Implementation Decisions

- **D1 — Terms and scope:** A *pin* is the single active featured `ChatMessage` for one widget. The *dashboard feed* is a bounded, live view of that widget's chat, not a stored chat history. The *highlight source* is a separate public browser source and does not alter the normal chat overlay. A new pin replaces the old one.
- **D2 — Data model:** Persist the active message as nullable structured widget data in PostgreSQL, with a monotonically increasing revision for ordering. This supports the current pin on first load, page reload, and WebSocket reconnect. Preserve the normalized message's author, platform, content, optional avatar/color, and fragments needed for rendering. A cleared pin is null. No message history table is required.
- **D3 — Read and write boundaries:** The public widget route and widget WebSocket remain read-only. The dashboard invokes a Next.js mutation route that validates input, verifies the Supabase user, and proves ownership of the widget before forwarding a pin or unpin command. The Rust backend accepts commands only through a server-to-server endpoint protected by a secret unavailable to browser code. Never accept pin commands from the public WebSocket. A failed command must surface in the dashboard without claiming success.
- **D4 — Event contract:** On successful persistence, the Rust backend broadcasts `pin_message` with the full normalized message or `unpin_message` to that widget's existing broadcast channel. A newly connected subscriber receives a `pin_state` snapshot containing the current message or null, then live events. All three event types include the persisted revision and are scoped to one widget; clients ignore older revisions if a snapshot and event race. The dashboard and highlight client must distinguish these events from chat messages and platform errors. The normal chat overlay must ignore highlight events.
- **D5 — Dashboard:** Put the live feed and pin control on the dashboard, using the existing widget ID and WebSocket. Keep a bounded set of recent messages, show connection/empty/error states, and visually identify the active pin. The pin action sends the selected normalized message; the unpin action clears it. Disable repeat actions while a mutation is pending and reconcile the display from confirmed server state.
- **D6 — Highlight source and appearance:** The highlight route uses the existing public widget UUID lookup and transparent page treatment. Display one high-contrast card with platform, author, message, and optional avatar where available. Use design tokens and the existing radius scale. Animate entry, replacement, and exit while honoring reduced-motion preference. Do not apply the normal chat auto-hide timer to the pinned message.
- **D7 — Setup:** The dashboard shows a separate highlight URL, copy feedback, and concise OBS browser-source guidance with a starting size of 800 × 250 px; the source remains fluid for user adjustment.
- **D8 — Framework conventions:** Before coding Next.js route or page changes, read the relevant installed Next.js 16.3.5 guides required by the frontend's local agent instructions. Follow the existing Supabase server-client, App Router, and WebSocket patterns where applicable.

## Testing Decisions

- Rust tests start in the red phase and cover authorized command acceptance, invalid or unauthorized command rejection, widget-scoped pin and unpin broadcasts, and a late subscriber receiving current state. Use the existing WebSocket integration test seam. Exercise persistence behavior with a database test seam where available; avoid mocking domain logic.
- Frontend verification covers the mutation route's validation and ownership check, dashboard pin/unpin failure feedback, normal chat filtering of highlight events, highlight initial state and transitions, and URL copy behavior. Use the highest existing practical seam; the current frontend has no test runner, so add focused tests only where they verify these interactions without duplicating implementation.
- Run the required verification commands in order: frontend lint and build, backend format, Clippy, and tests. Manually exercise dashboard pin, replace, unpin, separate OBS source, and reconnect in a browser where an integration environment is available.

## Out of Scope

- Multiple pins, queues, timed rotation, pin expiration, and a persistent chat history.
- Viewer voting, moderation, chat replies, and custom highlight appearance controls.
- Changes to platform ingestion or the existing normal chat layout presets.

## Further Notes

- The current backend uses per-widget in-process broadcast channels despite Redis being listed in the architecture as the intended broker. This ticket should work with the existing runtime and keep all highlight state durable in PostgreSQL; multi-instance fan-out is a separate architecture concern.
- Database changes require the normal migration path. The implementation must not depend on manual Supabase SQL editor steps.
