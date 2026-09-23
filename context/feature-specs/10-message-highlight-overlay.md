# Feature Spec — 10: Message Highlight Overlay & Dashboard Pin Control

## Problem Statement

During live streams, viewers frequently ask compelling questions or post notable comments that the streamer wants to highlight directly on stream for audience engagement:

1. Chat messages move rapidly, making it difficult for the audience to notice a specific comment in the moving stream feed.
2. Streamers have no built-in way to select a single message and feature it prominently in a dedicated OBS overlay.
3. Streamers must either read the question out loud repeatedly or use clumsy external tools to display viewer comments.

## Solution

Provide a Message Highlight system comprising a live interactive feed in the dashboard and a dedicated OBS Highlight Overlay route:

1. **Dashboard Interactive Feed**:
   - Streamer views a live stream of unified chat messages inside the dashboard.
   - Each message card has a "Pin to Screen" action button.
   - Active pinned message is highlighted in the dashboard with an "Unpin" button and clear status indicator.
2. **Dedicated Highlight Widget Route (`/widget/[id]/highlight`)**:
   - A standalone OBS browser source overlay designed to sit in a prominent stream screen area (e.g. lower-third or corner).
   - Renders the active pinned message in an elevated card with author name, platform badge, avatar/color, and content.
   - Smooth entrance (slide-up / pop-in) and exit (fade-out) animations when a message is pinned or unpinned.
3. **Real-time Sync**:
   - Communicates pin/unpin events in real-time with low latency via the WebSocket connection so OBS updates instantaneously.

## User Stories

1. As a streamer, I want to see incoming chat messages in a live dashboard feed with an action to pin any message, so that I can feature viewer comments during my stream.
2. As a streamer, I want to add `/widget/[id]/highlight` to OBS as a separate browser source, so that featured comments appear in a designated screen location separate from the fast chat list.
3. As a streamer, I want pinning a message to replace the currently pinned message immediately, so that switching featured questions is seamless.
4. As a streamer, I want an "Unpin" button in the dashboard, so that I can clear the featured message from the screen once we finish discussing it.
5. As an OBS viewer, I want the highlighted message to animate smoothly on screen, so that the stream production feels polished.

## Implementation Decisions

- **D1: Route Architecture**:
  - Add public route `frontend/src/app/widget/[id]/highlight/page.tsx`.
  - Reuse the secure widget UUID for authentication-free browser source inclusion in OBS.
  - Transparent background by default, with centered or bottom-anchored animated callout styling.

- **D2: WebSocket Event Schema**:
  - Extend WebSocket server protocol with two event types:
    - `{ type: "pin_message", message: ChatMessage }`
    - `{ type: "unpin_message" }`
  - When the streamer triggers pin/unpin in the dashboard, the event is broadcast to all active `/widget/[id]` and `/widget/[id]/highlight` subscribers.

- **D3: Highlight Card Visual Design**:
  - Elevated card with high-contrast border, platform badge, author name, and prominent message text.
  - Enter animation: `animate-in fade-in slide-in-from-bottom-4 duration-300`.
  - Exit animation: `animate-out fade-out duration-200`.

## Out of Scope

- Queueing multiple pinned messages for automatic cycling (single active pin per widget).
- Viewer voting or automated pin algorithms.
- Custom streamer response audio clips.
