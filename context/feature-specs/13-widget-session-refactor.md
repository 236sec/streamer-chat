# Feature Spec — C1: Backend Widget Session Refactor

## Implementation Plan — Backend Widget Session Refactor

### What we are building

Consolidate the Rust backend's per-widget broadcast channel, platform worker lifecycle, and connection cleanup into one widget session owner. Preserve the existing WebSocket, pin, and platform ingestion behavior. This is a backend orchestration refactor with regression tests, not a change to the Twitch EventSub, YouTube polling, or Kick Pusher clients.

### Language we agreed on

- **Widget session:** The backend state for one widget ID while at least one widget WebSocket is connected. It owns the widget-scoped broadcast sender and cancellation state for its ingestion workers.
- **Widget connection:** One WebSocket consumer of a widget session. Multiple connections to the same widget share the same session and feed.
- **Worker run:** One on-demand group of independent Twitch, YouTube, and Kick ingestion tasks for a widget. A later connection may start a new run if the previous run has ended while the session is still present.
- **Discovery:** YouTube `liveBroadcasts.list` lookup to obtain an active `liveChatId`. It stops during healthy `liveChatMessages.list` polling and may resume after the chat ends or repeated failures.

### Decisions made

- **D1 — Single session owner:** Replace the separate channel and worker registries with one per-widget session registry. The session owns the channel and cancellation state together so creation, reuse, and removal cannot disagree about which worker belongs to a channel.
- **D2 — Shared connection lifecycle:** The first connection starts a worker run, concurrent connections reuse it, and the final disconnect removes the session and cancels its workers. A reconnect after final cleanup gets a fresh session and worker run. A new connection to a retained session restarts a worker run that has terminated.
- **D3 — Platform boundaries:** Keep the three existing platform clients and their public behavior. Spawn their tasks independently after token lookup. Do not introduce a platform adapter trait for this refactor.
- **D4 — Pin continuity:** Pin commands still publish to the widget's channel. A connecting client still receives the persisted pin snapshot before regular feed events. Preserve the current close behavior when pin state cannot be loaded.
- **D5 — YouTube polling contract:** Once discovery finds a `liveChatId`, no `liveBroadcasts.list` calls occur across successful chat polls. After terminal chat failure, the existing retry and rediscovery path remains available. YouTube remains a polling client, not EventSub.

### Assumptions

- Session orchestration is the only production code scope. No database schema, frontend contract, authentication, platform protocol, normalization, or pin payload changes are required.
- Current YouTube terminal 404 behavior means one optional video-details fallback, five 2-second retries, then discovery after its configured interval. Existing token refresh and page-token behavior remain as implemented.
- The existing per-widget broadcast capacity and mock message cadence remain unchanged.

### How to build it

1. Add failing backend tests at the WebSocket and session boundary for shared sessions, independent widget routing, final-disconnect cancellation and cleanup, reconnect, and restart after an ended worker run. Use the existing mock widget path where it exercises the behavior without a database. Add focused HTTP request-count tests for the YouTube discovery-to-polling and terminal-404-to-rediscovery transitions.
2. Introduce one session owner per widget ID with the broadcast sender, connection lifetime, and worker-run cancellation/status. Define an identity or generation check so a finishing old worker or old connection cannot remove or mark a newer session/run as stopped.
3. Make WebSocket connection acquisition and release use the session owner. Subscribe the connection before starting a worker run so a new worker sees an active receiver. Ensure every exit path, including failed pin snapshot and send/receive task exit, releases its connection. Keep global `/ws` behavior separate.
4. Move on-demand worker startup coordination into the session. Keep token fetch and decryption behavior and launch Twitch, YouTube, and Kick independently as today. Retain the mock worker path and the ability to restart a completed run when a later connection arrives.
5. Route pin broadcasts through the session's existing channel and preserve the snapshot-first WebSocket sequence.
6. Run the required verification commands in pipeline order, then send the result to the Review Agent. Update progress only after final Engineer approval.

## Problem Statement

The widget backend keeps each widget's broadcast channel and worker status in separate maps. A WebSocket connection creates or reuses both, while a different code path tears them down. This makes concurrent connects, disconnects, and worker completion difficult to reason about and risks a stale worker or cleanup operation affecting a newer widget connection.

## Solution

Give each active widget one session that owns its chat broadcast channel and ingestion worker lifecycle. Connecting widgets share that session, and its final connection releases it. Users continue to receive the same Twitch, YouTube, and Kick chat messages and pin events through the existing widget WebSocket route.

## User Stories

1. As a streamer, I want multiple OBS or dashboard views of one widget to receive the same live chat feed, so that opening another view does not duplicate platform ingestion.
2. As a streamer, I want chat ingestion to recover after reloading a widget, so that a previous worker or cancellation cannot block the new connection.
3. As a streamer, I want one remaining widget view to keep receiving chat when another view closes, so that the backend only stops ingestion after the last view disconnects.
4. As a streamer, I want Twitch, YouTube, and Kick messages to keep arriving with their current format and timing, so that this internal refactor does not alter my overlay.
5. As a streamer, I want my current pin to appear when a widget reconnects and later pin changes to reach the correct widget, so that highlight behavior stays reliable.
6. As a system operator, I want a widget's channel and workers to share one lifecycle, so that stale cleanup does not cancel a newly connected session.

## Implementation Decisions

- **Session ownership:** The application layer owns one registry of active widget sessions keyed by widget ID. A session owns the broadcast sender, worker-run cancellation token and status, and active connection lifetime. Its public operations cover acquisition, worker start or reuse, widget-scoped publishing, and release. Keep one synchronization boundary for session identity changes and avoid holding it across database or network awaits.
- **Connection lifetime:** Acquire the session, subscribe to its channel, then start or reuse a worker run. Release the connection on every exit, including pin-snapshot failure. Only the final release removes the matching session and cancels its workers. An older release or worker completion must not mutate a replacement session.
- **Worker coordination:** Keep one active worker run per session. A connection starts a run only if none is running. Token lookup and decryption use existing application behavior. Each available platform starts in an independent task. A failed token lookup or terminated platform tasks leave the session able to start a new run on a later connection.
- **Platform contracts:** Twitch continues using EventSub WebSocket, Kick continues using Pusher WebSocket, and YouTube continues using `liveBroadcasts.list` discovery followed by `liveChatMessages.list` polling. Preserve cancellation, refresh, fallback, retry, message normalization, and platform error events. No new adapter interface or platform policy is needed.
- **YouTube discovery rule:** Stop broadcast discovery after obtaining `liveChatId` while chat polling is healthy. The existing chat-ended or terminal-failure paths can return to discovery. Do not infer that discovery is permanently forbidden for the worker's lifetime.
- **WebSocket and pin contract:** Preserve `/ws` and `/ws/widget/:id` behavior, mock query behavior, widget-scoped fanout, ping/pong, pin command authorization and payloads, snapshot-first delivery, and close-on-snapshot-load-failure behavior. Pin publication uses the session channel for that widget ID.
- **Data model:** No storage or API schema migration.

## Testing Decisions

- Follow backend TDD: begin with a failing lifecycle test that exposes the session ownership gap, prove the red phase, then implement. Characterization tests for behavior that already works may pass before the refactor and must remain green afterward. Preserve existing platform tests and their mock HTTP/WebSocket servers.
- Exercise the highest existing seam with real widget WebSocket clients using the mock feed: two simultaneous connections for one widget share one session and both receive messages; closing one keeps the other active; closing the last eventually removes the session and stops its worker; a reconnect creates a fresh working session. Inspect session identity or worker start count only where message delivery alone cannot establish single-run behavior.
- Cover races deterministically: a delayed old connection release or worker completion cannot remove or stop a newer session or run. Avoid timing-only assertions when a synchronization signal or state observation can establish the transition.
- Verify separate widget IDs remain isolated and the global `/ws` ping/pong path remains independent.
- Preserve pin tests for widget-scoped broadcasts and reconnect snapshots. Add a focused regression for connection release when snapshot loading fails, using the existing failing-database seam. Keep the disposable database integration test ignored until `TEST_DATABASE_URL` is available.
- Extend the existing YouTube HTTP mock tests to count requests: after one successful discovery response, observe multiple successful chat polls with no additional `liveBroadcasts.list` request; after terminal 404 retry exhaustion, observe a subsequent discovery request. Assert the existing video-details fallback and retry ordering where relevant. Keep the response polling interval and `nextPageToken` checks, token refresh, and receiver/cancellation teardown coverage.
- Run verification in the required order: frontend lint, frontend build, backend format check, backend Clippy with warnings denied, then backend tests. Report the existing ignored database test separately.

## Out of Scope

- Changes to platform transport or API strategy, especially replacing YouTube polling with EventSub.
- New platform client abstraction, protocol adapter trait, or generalized plugin system.
- Changes to frontend UI, widget rendering, pin feature behavior, schemas, OAuth flows, or secrets storage.
- Broader platform client reliability work beyond behavior-preserving session orchestration.

## Further Notes

- This spec is C1 only. Any later widget or platform work is a separate ticket and review gate.
- The Engineer must approve this plan and spec before implementation. The Review Agent will independently verify the completed build against these decisions.
