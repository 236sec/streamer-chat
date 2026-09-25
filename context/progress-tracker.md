# Progress Tracker

## Open Questions
- None. The streamer chose automatic selection when no saved chat UUID is available; legacy OBS URLs remain usable even if the canonical UUID differs.

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
- 13: Backend Widget Session Refactor — one session owns channel, worker run, and final-connection cleanup. Shared fanout, stale-worker safety, and YouTube discovery-to-polling behavior have regression tests.
- 14: Backend Pin State Refactor — the application pin module owns validation, revisioned persistence/publication, and snapshot preparation; HTTP and WebSocket transport keep authorization and I/O. Application tests cover state changes and ordering without a database.
- 15: Restore Pin Control After Widget Settings Refresh — Widget Settings lists owned widgets, lets the streamer select the OBS UUID when several exist, and keeps that owned choice across refresh. Dashboard home uses the same selection; pin previews reconcile from confirmed events. Ambiguous lookups no longer create rows. Auth revalidation clears prior-account state while preserving same-account drafts.
- 16: One Permanent Widget UUID Per Account — canonical account mapping, legacy URL support, pin migration, one dashboard identity, and scoped public lookup. The live database migration is installed and scoped access smoke checks pass. Independent review found no remaining major code issues.
- 17: Viewer Count OBS Widget — separate public OBS source with Twitch, YouTube, and Kick counts, backend-configured refresh interval, Widget Settings URL/copy/preview, and canonical/legacy UUID handling. Independent review found no remaining Critical or Important defects.

## Verification Limits
- Ticket 17 passes frontend lint, the previously approved Webpack production build with temporary local Google Fonts responses, backend format, Clippy, and Rust tests. Exact Turbopack build could not fetch Google Fonts in this network-restricted environment. No frontend test runner is installed, and database-backed viewer endpoint and live OBS checks were unavailable. Seven database-dependent Rust tests remain ignored without a disposable `TEST_DATABASE_URL`.
- Ticket 16 passes frontend lint, the approved Webpack build substitute, backend format, Clippy, and Rust tests. The migration is installed on the linked Supabase database; live smoke checks confirmed scoped public lookup, denied anonymous widget listing, one owner-visible widget, and restricted pin-column updates. Six new database integration tests for canonical aliases, open-socket handoff, resolution, pins, concurrency, and RLS remain ignored without a disposable `TEST_DATABASE_URL`. No live OBS end-to-end test was available. A pre-mapping socket can take up to five seconds to switch to the canonical session.
- Ticket 10 database integration test is explicitly ignored until a disposable `TEST_DATABASE_URL` is available. Docker's daemon and local PostgreSQL binaries were unavailable in this environment. From `backend/`, run `cargo test pin_broadcasts_to_one_widget_and_reconnects_from_database -- --ignored` with `TEST_DATABASE_URL` set. The required lint, build, format, Clippy, and Rust test commands pass.
- Ticket 13 frontend lint and backend format, Clippy, and tests pass. The user approved a passing Webpack production build with temporary local Google Fonts responses as the build gate because exact `npm run build` fails when Turbopack tries to bind a local port in this environment. The same existing disposable-database test remains ignored.
- Ticket 14 passes frontend lint, the same approved Webpack build substitute, backend format, Clippy, and tests. The existing disposable-database pin integration test remains ignored.
- Ticket 15 passes frontend lint, the approved Webpack build substitute, backend format, Clippy, and tests. The authenticated dashboard/OBS refresh-to-unpin sequence could not be run in this environment. The same disposable-database pin integration test remains ignored.

## Blocked
- None.
