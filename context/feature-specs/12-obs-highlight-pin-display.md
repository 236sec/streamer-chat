# Feature Spec — 12: OBS Highlight Pin Display Bugfix

## Problem Statement

The streamer pins a live message in the dashboard, but the separate OBS highlight browser source stays blank. The dashboard can show a confirmed pin while the public source has no visible card, so the streamer cannot rely on the highlight during a stream.

## Solution

Make the currently pinned message appear in the separate OBS highlight source when it opens, and update that source when the streamer pins, replaces, or clears a message. Identify the failed boundary in the existing route → WebSocket → snapshot/event → client-render path before changing it. Keep the normal chat source and existing pin controls working.

## User Stories

1. As a streamer, I want the message confirmed as pinned in the dashboard to appear in my OBS highlight source, so that viewers see the selected message.
2. As a streamer, I want an already pinned message to appear when OBS opens or refreshes the source, so that the display does not depend on a new chat event.
3. As a streamer, I want replacing or clearing the pin to update the open source, so that OBS matches the dashboard.
4. As a streamer, I want the dashboard's copied URL to open the separate highlight source, so that I can configure OBS reliably.

## Implementation Decisions

- **D1 — Terms:** The *highlight source* is the public, separate browser source for one active pinned message. The *normal widget* is the public scrolling chat source. *Confirmed pin* means the authenticated mutation succeeded and returned the persisted revision.
- **D2 — Fault isolation:** Reproduce with the copied highlight URL in a normal browser and OBS. Check the public route result, the actual WebSocket URL and connection state in each browser, the initial `pin_state` frame and later pin events, schema parsing, and the rendered card. Record the first failing boundary and fix that cause. Do not infer success from the dashboard's mutation response alone.
- **D3 — State contract:** Preserve the persisted single-message pin and monotonic revision contract. A newly connected highlight source receives the current pin even if no new chat message arrives. Live pin, replacement, and unpin events update the source only when their revision is newer. Normal chat remains unaffected.
- **D4 — Connection configuration:** If diagnosis confirms a bad or unreachable browser WebSocket endpoint, correct the public runtime configuration so the dashboard and OBS reach the same backend. Do not place a private internal backend URL or pin command secret in browser code.
- **D5 — Error visibility:** If the route loads but cannot connect or read the snapshot, provide a concise diagnostic in the dashboard or development log that identifies that boundary. The public OBS canvas stays transparent when there is no pin. Avoid exposing secrets or chat payloads in logs.
- **D6 — Scope:** Repair the existing highlight path and its setup information only where the reproduction proves a defect. No new pin model, storage migration, or visual redesign is needed.

## Testing Decisions

- Use the existing backend WebSocket integration seam to verify a late subscriber receives the persisted pin and an open subscriber receives pin, replacement, and unpin events in revision order. Run the database-backed test with a disposable PostgreSQL URL when available; it is currently ignored without that service.
- At the frontend boundary, verify that the actual snapshot and event frames validate against the client schema and that the highlight renders the message after initial load and after reconnect. Cover the failure boundary identified during reproduction rather than adding tests that mirror implementation details.
- In a browser and OBS, open the copied highlight URL, pin a live message, refresh the source, replace the pin, and unpin it. Confirm that the normal chat widget still displays chat. Record the tested URL host and WebSocket host without credentials.
- Run the project's required gates in order: frontend lint and build, then backend format, Clippy, and tests.

## Out of Scope

- New highlight layouts or appearance controls.
- Changes to Twitch, YouTube, or Kick ingestion.
- Multi-instance WebSocket fan-out or replacing the current in-process broadcast channel with Redis.
- A persistent chat history or multiple simultaneous pins.

## Further Notes

- Existing code already persists the pin, broadcasts revisioned events, and sends a snapshot on widget WebSocket connection. The frontend client parses only these events before displaying a card. A blank source can arise at the public route, connection, snapshot, parsing, or rendering boundary; the report alone does not distinguish them.
- The frontend environment module substitutes a localhost WebSocket URL during production builds. Investigate the URL embedded in the client bundle if OBS cannot connect, but do not assume this is the cause before observing the browser connection.
- The dashboard's copied highlight URL ends with `/highlight`. The normal widget URL is a different source and intentionally ignores pin events.
- The preceding widget display regression fix has unrelated uncommitted edits. Preserve them while implementing this ticket.
- Reproduction on 2026-09-24 isolated the failure to a stale OBS URL. The supplied source used an older widget ID; its backend snapshot was `pin_state` revision 0 with no message. A fresh dashboard pin returned HTTP 200 and persisted on a different, current widget ID; a new WebSocket subscriber received revision 4 with the pinned Kick message. Both highlight routes returned HTTP 200. The streamer copied the current highlight URL from the dashboard. No application code change is indicated by this reproduction.
- The streamer confirmed the current highlight URL displays the pinned Kick message. Frontend lint/build and backend format/Clippy/tests pass. The existing database-backed pin integration test remains ignored without a disposable `TEST_DATABASE_URL`.
