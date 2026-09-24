# Feature Spec — 15: Restore Pin Control After Widget Settings Refresh

## Problem Statement

After the streamer pins a message and refreshes Widget Settings, OBS still displays the highlight, but the dashboard says “No message pinned” and offers no unpin control. The dashboard reports “Live chat connected.” The streamer confirmed that the dashboard's highlight URL after refresh has a different widget UUID from the active OBS highlight URL.

## Solution

Let the authenticated streamer select the existing widget used by OBS when their account has multiple widget records. Use that selection consistently in Widget Settings and the dashboard home, including the highlight URL, live feed, and pin controls. Preserve all existing records, pins, and public URLs. Stop a failed or ambiguous widget lookup from inserting another record.

## User Stories

1. As a streamer, I want to identify my existing OBS widget by its UUID and current pin, so that I can restore dashboard control of that source.
2. As a streamer, I want my widget choice to survive refresh in this browser, so that I can unpin without selecting it again.
3. As a streamer, I want Widget Settings and dashboard home to use the same selected widget, so that their copied URLs agree.
4. As a streamer, I want a settings lookup error to be visible, so that it does not silently create a new widget.

## Implementation Decisions

- **D1 — Terms:** A *widget record* is one row with a public UUID. The *selected widget* is the owned record whose settings, URLs, chat feed, and pin controls the dashboard currently manages. The *OBS widget* is the UUID in the active OBS highlight source URL. A *current pin* is that record's persisted `pinned_message` and `pin_revision`.
- **D2 — Confirmed failure boundary:** The dashboard and OBS highlight URLs have different UUIDs after refresh. The dashboard socket's connected status refers only to its newly selected UUID. The schema permits multiple records per user. Widget Settings uses a singular query by user ID and interprets `PGRST116` as zero records, although it also means multiple records. Its recovery branch inserts another row and changes the selected UUID. Correct this lookup and selection path. Do not change backend pin transitions or the event schema without separate evidence of a defect.
- **D3 — Existing widget selection:** Query the authenticated user's owned widget records as a list. With one record, select it. With multiple records, show a small chooser in Widget Settings listing each full UUID and a safe text preview of any current pin. State that the streamer should match the UUID in their OBS highlight URL. Require explicit selection if no valid prior choice exists. Do not infer the OBS widget from creation order or revision. Keep pin mutation controls and copyable source URLs inactive until a widget is selected.
- **D4 — Remember and validate:** Store the selected UUID under a key scoped to the authenticated user in browser storage. On each load, accept it only if it still appears in the freshly fetched owned records. An invalid or missing saved choice with multiple records returns to the chooser. A choice made in Widget Settings is reflected in dashboard home, including its onboarding and copy URL. Selection is local to the browser, so a new browser must choose again when multiple records exist.
- **D4a — Open tabs and account changes:** Dashboard home revalidates selection when another tab changes it or the window regains focus. Both dashboard views clear prior-account widget state on sign-out, account change, or uncertain authentication. Successful same-account revalidation preserves unsaved Widget Settings edits and the mounted pin feed; a same-account widget-list failure reports an error without discarding that draft.
- **D5 — Creation and error handling:** Insert a widget only when a successful list query proves the user owns zero records. Never insert after a query error or ambiguous result. Report load and insert failures. No uniqueness migration or row cleanup is included. Concurrent first-time loads in different tabs can still both observe zero rows without a database constraint; this ticket does not claim to eliminate that separate race.
- **D6 — Preservation and authorization:** Keep every existing widget row, pin, revision, and public URL unchanged. Selection only changes which owned widget the dashboard controls. The existing pin mutation route continues to verify ownership server-side. Treat the browser's saved UUID as a preference, never as proof of ownership. Validate unknown pin preview data before rendering it.
- **D7 — UI conventions:** Use existing dashboard cards, buttons, error states, CSS tokens, `cn()` class merging, and the documented radius scale. Add no hardcoded colors. Read the installed Next.js 16.3.5 guides required by the frontend instructions before coding route or page changes.

## Testing Decisions

- Exercise the settings loader at its highest practical frontend seam with zero, one, multiple, and failed widget-list responses. Verify that only the successful zero-row response permits insertion and that multiple rows never insert.
- Exercise selection with a saved owned UUID, a stale or other user's saved UUID, and no saved UUID. Verify that Widget Settings and dashboard home resolve to the same owned record. The frontend currently has no test runner; use focused automated tests only if a suitable seam can be established without broad tooling changes, and document browser verification otherwise.
- Inspect cross-tab selection and account-switch handling: stale user data must clear, while returning focus to the same account and widget must preserve unsaved settings and the live pin panel. Confirmed pin events should update chooser previews.
- In a browser, select the UUID from the active OBS highlight URL, refresh Widget Settings, verify that the dashboard URL and OBS URL still match, that the persisted pin appears, and that unpin clears the OBS source. Verify that other existing widget URLs remain reachable.
- Run the required verification gates in order: frontend lint/build, then backend format, Clippy, and tests. Use the previously approved environment-specific build substitute only if the documented Turbopack restriction recurs.

## Implementation Plan

1. Replace the singular user-widget lookup with an owned-record list response. Distinguish zero rows, one row, multiple rows, and query error explicitly.
2. Add one shared selection rule for Widget Settings and dashboard home: read the user-scoped saved UUID, validate it against the current owned list, and require a choice when several rows remain.
3. Add the Widget Settings chooser with full UUIDs, validated pin previews, and a clear selected state. On selection, update settings, normal/highlight URLs, preview, live feed, and pin panel from that record. Persist the choice for that user.
4. Ensure the dashboard home uses the same selected UUID for its widget status and copied normal source URL. Suppress copy actions when multiple rows exist without a valid selection.
5. Verify the reported refresh → restored pin → unpin sequence against the unchanged OBS highlight URL, then run the required coding gates.

## Out of Scope

- Deleting, merging, or rewriting existing widget rows, pinned data, revisions, or public URLs.
- Database uniqueness enforcement or a cross-tab creation lock.
- Changes to platform ingestion, backend pin logic, or highlight appearance.

## Further Notes

- The user's URL comparison confirms ID drift, but the actual row count has not been inspected. The list query will establish whether the code-confirmed multiple-row path is the cause of this instance. If the list contains only one row, investigate when and where the OBS URL became stale before attributing it to `PGRST116`.
- The public route remains keyed by UUID, so existing OBS sources continue working while dashboard selection is restored.
- No authenticated Supabase/OBS browser session was available for the live refresh-to-unpin and account-switch checks. Frontend lint, the approved Webpack build substitute, backend format, Clippy, and tests passed; the existing disposable-database pin integration test remains ignored.
