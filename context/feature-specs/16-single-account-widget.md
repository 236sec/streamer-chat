## Problem Statement

An account can own several widget rows. Chat and highlight then use different UUIDs, so a pin made in the dashboard may not reach the OBS highlight source. The current UUID selector asks the streamer to repair this state. The public read policy also lets an anonymous database client list widget UUIDs, defeating the intended unguessable URL access model.

## Solution

Each account has one canonical, permanent, random widget UUID for both chat and highlight. Existing chat UUIDs take precedence when there is reliable evidence of which one the streamer copied. If no such evidence exists, the system chooses deterministically. Legacy widget rows remain private and their old URLs resolve to the canonical stream. The dashboard shows only canonical chat and highlight URLs, without a selector. Public visitors can read only the widget named by a UUID they already know.

## User Stories

1. As a streamer, I want chat and highlight URLs to share one UUID, so pin controls always affect my OBS sources.
2. As a streamer with one widget, I want its UUID to stay the same after this change, so my OBS setup keeps working.
3. As a streamer with duplicate rows, I want the chat UUID I previously copied to remain canonical when the application can validate that evidence, so my chat source URL stays stable.
4. As a streamer with an older highlight or chat URL, I want it to keep showing the current account widget, so migration does not blank an OBS source.
5. As a streamer, I want a current pin to remain visible after migration, so my highlight scene does not lose its message.
6. As a new streamer, I want concurrent dashboard loads to produce one widget, so I never need to select between duplicates.
7. As an OBS viewer, I want a known widget URL to work without signing in, so the browser source can load.
8. As an account owner, I want my widget UUID kept out of public lists, so only someone with my URL can open the overlay.

## Implementation Decisions

- **Canonical identity.** A private account-to-widget identity mapping has `user_id` as its unique key and references one owned canonical widget UUID. The canonical widget row owns appearance settings, pin state, and both overlay paths. Newly created widget UUIDs come from the database's cryptographically random generator. Once mapped, an account's canonical UUID is never rotated by routine saves, refreshes, reconnection, or later browser hints. Legacy widget rows remain stored but are not active identities.
- **Evidence and fallback.** A saved copied-chat UUID is evidence only when it belongs to the signed-in account. The prior browser selection is weaker evidence and must not override a validated copied-chat UUID. No row property currently identifies the URL actually configured in OBS. If the copied-chat hint is missing or invalid, use a stable ordering of owned UUIDs as the fallback. Never guess from pin presence. This fallback cannot guarantee that the chosen canonical UUID equals an unobserved OBS chat URL.
- **First resolution.** The authenticated account-resolution operation runs in one database transaction. It locks or serializes on the account, checks for an existing identity mapping first, and returns that mapping unchanged if present. Otherwise it validates the optional copied-chat hint against rows owned by that account, chooses it if valid, or uses a stable UUID ordering among owned rows. If no row exists, it creates one. It writes the unique mapping and reconciles pin state before returning. Concurrent callers may race with different hints; the first committed mapping wins, and every later caller receives the same UUID. There is no global backfill or deletion requirement.
- **Legacy behavior.** Existing widget rows remain in `widgets`; any owned row other than the mapped canonical row acts as a legacy alias. Both public page routes accept a known legacy UUID and render the mapped canonical widget after identity resolution. The incoming OBS URL remains usable, while the rendered client connects using the canonical UUID. The backend must also canonicalize direct widget WebSocket subscriptions before session acquisition, so legacy IDs cannot start separate ingestion or pin sessions after mapping. A socket already open before the first mapping checks at a bounded five-second interval until the mapping appears, then subscribes to the canonical session, sends its pin snapshot, and releases the old session without an OBS refresh. The checks stop once mapped. New copy controls emit only canonical URLs. An account that has not yet resolved identity continues to serve its existing URLs using the addressed row; its first authenticated dashboard visit establishes the mapping.
- **Data reconciliation.** Keep canonical appearance settings. Preserve its valid pinned message when present. If it has no valid pin, copy one valid pinned message from a legacy row in the same transaction, rewrite that message's `widget_id` to the canonical UUID, and advance the canonical revision beyond the relevant legacy revisions. If more than one legacy row has a valid pin, use a deterministic ordering. Legacy rows and their original pin data remain stored. Pin content is ordinary JSONB and requires no encryption. Platform access and refresh tokens retain their existing encryption and account ownership model.
- **Database access.** Remove public table-wide `SELECT` access. The identity mapping is private. Owner-facing reads return only the mapped canonical row; the dashboard must replace the existing list-of-owned-widgets query with a single canonical resolution result. Row policies should restrict normal owner reads and updates to that row once mapped, while the privileged resolution function can inspect all of the owner's legacy rows. Authenticated table UPDATE grants cover appearance columns only; pin content and revision require the backend command path. Public routes use a narrowly scoped database function or server boundary that accepts one UUID, resolves its owner's canonical row when mapped, and returns only fields needed by the overlay. Database grants and row policies must prevent anonymous clients from enumerating UUIDs through tables, mappings, views, or unscoped function calls.
- **Creation and mutation.** The account-resolution operation is the sole widget creation path. Its transaction and account-level lock make first creation race-safe; the mapping's unique `user_id` key enforces one canonical identity even though legacy widget rows remain. Appearance writes and pin commands require authenticated ownership of the mapped canonical UUID. The internal backend pin endpoint must reject or canonicalize a legacy UUID before mutation. The backend routes live chat and pin events by the canonical UUID.
- **Dashboard.** Remove the UUID chooser, browser-stored selected-widget preference, and instructions to match OBS IDs. The dashboard home and Widget Settings use the same canonical resolution result. Keep copy confirmation tied to the canonical UUID. A saved copied-chat hint may be used once for migration and must not become an ongoing alternate identity.

## Testing Decisions

- Prove the duplicate-account resolution path with database tests covering valid copied-chat hint, invalid or foreign hint, no hint, repeated calls, concurrent calls with different hints, and mapping uniqueness. Verify that all legacy rows remain stored and no second canonical row is created. Start Rust changes with a failing test per backend TDD rules.
- Verify single-row accounts retain their UUID and settings. Verify legacy chat and highlight URLs resolve to the canonical widget and that direct legacy WebSocket connections use the canonical session.
- Cover pin reconciliation for a canonical pin, a migrated pin, no pin, and multiple legacy pins. Assert the migrated message's widget ID and revision permit the existing pin dashboard and highlight snapshot to display and unpin it.
- Use an anonymous database client to prove known UUID lookup succeeds and full table, mapping, view, and unscoped function enumeration fail. Check the owner-facing read returns only the canonical widget, normal owner listing cannot surface legacy rows, canonical mutations work, and legacy mutations fail or resolve safely. Another user cannot resolve private account data through owner APIs.
- At the page level, verify both dashboard pages show one UUID with no chooser, and copy links share it across refresh and sign-in changes. Verify old OBS page paths still load with canonical live behavior.
- Run the required frontend lint and build, backend format, Clippy, and tests in pipeline order. The existing disposable-database integration test remains a separate environment limit unless a test database is supplied.

## Out of Scope

- Encrypting pinned message content.
- Rotating public widget UUIDs, deleting legacy widget rows, or adding authentication to OBS browser sources.
- Changing Twitch, YouTube, or Kick ingestion behavior or token encryption.
- Asking the streamer to choose a UUID or paste an OBS URL.

## Further Notes

The copied-chat hint exists only in browser storage, and only the dashboard home copy action currently writes it. The Widget Settings copy action and OBS configuration do not. Therefore the system cannot always identify the actual chat UUID. Alias support preserves old source behavior in that case, but the canonical UUID may differ from the URL that was already in OBS. This is a product limit to disclose at the Engineer gate.

## Implementation Plan — One Widget Identity Per Account

### Language

- **Canonical UUID:** The one active widget row ID used for both chat and highlight and emitted by new copy controls.
- **Legacy alias:** An existing noncanonical widget row whose UUID remains usable in public URLs and resolves through its owner's private identity mapping.
- **Copied-chat hint:** A browser-stored UUID from the dashboard home copy action, validated against owned rows before first canonical resolution.

### Build sequence

1. Add a private unique account-to-widget identity mapping, canonical-only owner reads, scoped public lookup, and transactional authenticated account resolution. Keep every existing widget row.
2. Update both public overlay routes and backend WebSocket subscription to resolve known canonical and legacy IDs to the canonical stream when a mapping exists.
3. Update dashboard home and Widget Settings to call account resolution, supply any valid copied-chat hint, and use one canonical widget. Remove the selector and its persisted preference.
4. Reconcile pin data atomically on first resolution. Restrict appearance and pin mutations to the mapped canonical widget and verify that legacy rows remain stored but do not appear as separate dashboard widgets.
5. Run the behavior tests and required verification commands, then submit for isolated Review Agent and Engineer gates.
