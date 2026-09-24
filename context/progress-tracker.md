# Progress Tracker

## Open Questions
- None.

## Frontier Tickets
- None.

## In Progress
- None.

## Completed
- 01: Supabase Auth & Dashboard Scaffold
- 02: Rust Backend WebSocket Server Scaffold
- 03: Next.js Widget Receiving Dummy Messages
- 04: Real Twitch/YouTube/Kick Integration
- 05: Rust Backend Twitch Chat Ingestion
- 05-fix: Twitch EventSub bugfix — dynamic user ID resolution, OAuth scopes, reconnect/disconnect UI
- 06: Rust Backend YouTube Chat Ingestion
- 06-fix: YouTube 404 retry / fallback + WebSocket channel and worker reconnect lifecycle
- 07: Rust Backend Kick Chat Ingestion
- 07-fix: Worker reconnect lifecycle / state redundancy bug fix
- 08-fix: Kick Pusher reconnect loop fix (CancellationToken & sleep_or_cancel) + token expiry error propagation (WebSocket platform_error, widget alert banner & accounts page token status)
- 08: Frontend UX and Error Handling (Toast notifications, Supabase widget persistence, dynamic origin URL & copy feedback, 3-step onboarding checklist, OBS setup guide modal, Kick validation, OAuth error banner, token tooltips, disconnect confirm)
- 09: Chat Overlay Enhancements (Auto-hide duration timer with smooth fade-out CSS transitions, Card/Bubble/Clean layout presets, dashboard settings controls, and live preview synchronization)
- 10: Message Highlight Overlay & Dashboard Pin Control (authenticated dashboard pin controls, persisted pin state, widget-scoped broadcasts, reconnect snapshot, and separate OBS highlight source)
- Widget display bugfix: restored Widget Settings scrolling after the highlight panel was added and transparent document background for the public chat source.
- 12: OBS highlight pin display investigation — the OBS source used an older widget ID with no pin. The current dashboard highlight URL displayed the persisted Kick pin after the source URL was corrected. No application code changed; the required verification commands passed.

## Verification Limits
- Ticket 10 database integration test is explicitly ignored until a disposable `TEST_DATABASE_URL` is available. Docker's daemon and local PostgreSQL binaries were unavailable in this environment. From `backend/`, run `cargo test pin_broadcasts_to_one_widget_and_reconnects_from_database -- --ignored` with `TEST_DATABASE_URL` set. The required lint, build, format, Clippy, and Rust test commands pass.

## Blocked
- None.
