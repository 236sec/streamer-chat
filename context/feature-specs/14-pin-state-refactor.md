# Feature Spec — C3: Backend Pin State Refactor

## Problem Statement

The current backend pin behavior works, but the web transport performs the pin state transition itself. It validates message content, coordinates the per-widget lock, writes the database revision, constructs and serializes events, publishes them, and loads reconnect snapshots. This makes the transition difficult to test without the disposable PostgreSQL test that is currently ignored, and it leaves the application boundary unclear.

## Solution

Move the complete pin and unpin transition, plus reconnect snapshot preparation, into the application pin module. Keep the HTTP and WebSocket handlers responsible for parsing transport input, checking the internal command secret, mapping application outcomes to existing responses, and sending or closing sockets. Preserve the existing persisted state, revisioned event payloads, widget-scoped delivery, and snapshot-first reconnect behavior.

## User Stories

1. As a streamer, I want a successful pin to remain visible when the highlight source reconnects, so that a temporary source disconnect does not clear the selected message.
2. As a streamer, I want replacing or clearing a pin to update only my widget, so that other streamers' sources stay unchanged.
3. As a streamer, I want invalid pin commands and persistence failures to remain visible as command failures, so that the dashboard does not claim an unconfirmed state.
4. As an OBS viewer, I want the current pin snapshot before later live changes, so that the highlight source applies revisions in a predictable order.
5. As a maintainer, I want pin transitions testable through the application boundary, so that validation, persistence, revision, and publication can be verified without a running database.

## Implementation Decisions

- **D1 — Terms:** A *pin transition* is validation followed by a database write that increments the widget's revision, event construction, and widget-scoped publication. A *snapshot* is the current persisted revision and optional message sent as `pin_state` to a newly connected widget subscriber. *Snapshot-first* means a subscriber is registered before the snapshot is loaded and the snapshot frame is sent before queued live frames are forwarded.
- **D2 — Application ownership:** The application pin module owns command validation, pin/unpin event selection, revisioned event serialization, persistence coordination, publication, and snapshot construction. The web module retains HTTP secret enforcement, request decoding and UUID parsing, response status mapping, WebSocket upgrades, frame I/O, and close behavior. Do not move authentication into a public socket path or accept mutations over WebSocket.
- **D3 — Narrow storage boundary:** Keep SQL and JSONB row mapping in the infrastructure database module. Define only the pin read and write dependency needed by the application workflow, with the current PostgreSQL functions as its production implementation. This permits an in-memory boundary fake in application tests. Do not add a general repository framework or alter the schema.
- **D4 — Channel boundary and ordering:** Use the existing widget session publisher and its per-widget lock. A transition holds the widget lock through write, event construction, and publication, so concurrent commands publish in persisted revision order. Snapshot preparation uses the same lock. On WebSocket connect, acquire the widget subscription before preparing the snapshot and send that snapshot before forwarding queued live events. A queued event with the same revision as the snapshot remains safe because clients already ignore revisions that are not newer.
- **D5 — Existing contract:** Preserve `pin_message` with the full normalized message, `unpin_message` with `message: null`, and `pin_state` with the current message or null. Each carries the persisted integer revision. Preserve one active pin per widget, replacement semantics, widget-scoped delivery, and the current normalized message validation bounds. Do not change frontend parsers or event names.
- **D6 — Failures:** Invalid UUID or command remains a bad request, missing database remains unavailable, missing widget remains not found, and storage or serialization failure remains a server failure. Failed validation or persistence must not publish. A missing widget or failed snapshot load still closes the widget socket; an absent database or non-UUID mock widget keeps its current no-snapshot behavior. Do not expose message content or secrets in error logs.
- **D7 — Scope:** This is a backend refactor of the existing behavior. No frontend, database schema, platform ingestion, or widget session lifecycle changes are part of C3.

## Testing Decisions

- Follow backend TDD. First write a failing application-level test that submits a valid pin through the new application entry point using an in-memory storage boundary and an actual widget-scoped broadcast receiver. Assert the persisted revision, returned `pin_message` payload, and received payload. This test must fail before the application workflow exists.
- At that same application seam, cover invalid command without write or publish; pin replacement and unpin with increasing revisions; independent widget state and no cross-widget publication; missing widget and storage failure without publication; and snapshot with the current revision and optional message. Use a fake only for the database boundary, not for validation, event construction, or the real widget publisher.
- Test the subscription/snapshot race at the application and session seam: subscribe before snapshot preparation, arrange a transition around snapshot loading, and verify the snapshot is emitted first and live events follow in nondecreasing revision order. Avoid timing-only sleeps as the proof of ordering.
- Keep the existing HTTP/WebSocket tests for secret rejection, invalid payload status, socket close on snapshot load failure, and reconnect behavior. Extend them only where the refactor changes a meaningful boundary. The ignored disposable-PostgreSQL test remains the strongest end-to-end database contract check, but it cannot be the only evidence for this refactor while `TEST_DATABASE_URL` is unavailable.
- Run the required gates in order: frontend lint, frontend build, backend format, backend Clippy with warnings denied, and backend tests. Respect the recorded environment-specific frontend build substitute only if the Engineer has approved it for this run.

## Out of Scope

- Frontend highlight or dashboard changes, DB migration, stored chat history, multiple pins, and new event types.
- Redis or multi-process fanout, public mutation access, and changes to Twitch, YouTube, Kick, or widget worker lifecycle.
- General-purpose storage or event abstractions beyond the pin workflow's immediate dependencies.

## Further Notes

### Implementation Plan

1. Write and run the red application test for a valid pin transition through the intended entry point.
2. Introduce the minimal pin storage dependency and production PostgreSQL adapter while preserving the current SQL read and write behavior.
3. Implement the application transition and snapshot operations. Keep the per-widget lock across revision write and publish, and across snapshot load. Reuse the existing widget session broadcaster.
4. Reduce the HTTP pin handler to secret check, transport parsing, application call, and status/JSON response. Reduce the WebSocket pin setup to subscription, application snapshot call, first-frame send, and existing close handling.
5. Add the application behavior and ordering tests, run targeted backend tests, then run the full required verification sequence.

The current ignored database test uses a disposable `TEST_DATABASE_URL`; do not claim that the PostgreSQL adapter has been integration-tested unless that test actually runs. If the implementation changes the stated architecture or standards, update the relevant context before continuing under the project workflow.
