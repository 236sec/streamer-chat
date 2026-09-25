## Problem Statement

Streamers can show combined chat and a pinned message in OBS, but cannot show how many people are currently watching on Twitch, YouTube, and Kick from this application.

## Solution

Add a separate, public OBS browser source for viewer counts. It always shows labeled Twitch, YouTube, and Kick values. Each value comes from that platform's current live stream count when available, and displays `0` when the stream is offline, the account is disconnected, the count is omitted, or a request fails. The streamer can copy the source URL from Widget Settings. The backend supplies a refresh interval from its configuration, defaulting to 60 seconds. The source reads counts immediately when loaded or reconnected, then at that interval.

## User Stories

1. As a streamer, I want a separate viewer count OBS source, so I can place it independently of chat and highlights.
2. As a streamer, I want labeled counts for Twitch, YouTube, and Kick, so viewers can see each platform's audience separately.
3. As a streamer, I want all three labels to stay visible when a platform is offline or disconnected, so the source layout remains stable.
4. As a streamer, I want missing or failed counts to display as `0`, so the source never shows a blank or an old number.
5. As a streamer, I want counts to appear when the source connects and update automatically, so I do not need to refresh OBS manually.
6. As a streamer, I want a copyable source URL and preview in Widget Settings, so I can set up the OBS source confidently.
7. As an OBS viewer, I want the source to load through its public widget UUID without signing in, so it works as a browser source.
8. As a streamer using an older widget UUID, I want that UUID to resolve to my canonical account source, so an existing alias works if used for the new route.

## Implementation Decisions

- **Source and identity.** Add a viewer count page beneath the public widget UUID. Resolve a known UUID through the existing scoped public widget lookup and render from the canonical widget identity. Do not introduce a second UUID or alter chat and highlight URLs.
- **Display contract.** Return and render exactly three nonnegative integer values keyed by `twitch`, `youtube`, and `kick`, in that order. The public JSON response has a `counts` object with those three keys plus a separate `refresh_seconds` number. Default every count to `0` for disconnected, offline, absent, malformed, expired-token, or failed requests. A displayed zero therefore means that no positive count is available, not necessarily that the platform measured zero viewers. Never retain a stale positive value after a failed refresh. Do not show a combined total.
- **Count sources.** Fetch counts on the Rust backend, never from the public browser. Twitch uses the connected user's Helix identity and Get Streams `viewer_count`; an absent stream means zero. YouTube uses the connected account's active broadcast ID and Videos `liveStreamingDetails.concurrentViewers`; a missing field means zero. Kick uses the existing username-only connection and the channel lookup already used by chat, reading the live stream's viewer count when present. Do not add Kick OAuth in this ticket. Reuse encrypted tokens, the existing YouTube refresh path, and platform base URL configuration. Secrets and raw API responses must not be exposed in the public response or logs.
- **Public data path.** Add a narrow public HTTP count-read endpoint keyed by the known widget UUID. Validate its UUID, resolve the canonical identity before token lookup, and return only the count object and safe refresh metadata. The Next.js route proxies to the Rust backend so the browser uses a same-origin read. A count read works without a chat or highlight socket and must not start chat ingestion workers. Bound external request time and coalesce simultaneous reads for one canonical widget; a short per-widget cache prevents duplicate external calls from concurrent OBS sources.
- **Refresh configuration.** Read `VIEWER_COUNT_REFRESH_SECONDS` from the Rust backend environment, including the existing `backend/.env` loading path through dotenvy. Default to 60 seconds. Accept whole-number seconds from 30 through 300; use the 60-second default for absent, malformed, or out-of-range values. Include the validated `refresh_seconds` in each successful public count response. The OBS client requests counts on mount and restored connectivity, then schedules the next read from response metadata. On a failed read, it shows zeros and retries after the last validated interval, or 60 seconds before any successful response. It clears timers and aborts in-flight reads on unmount. Do not read, print, or expose other values from `backend/.env`.
- **Dashboard.** Add a count preview, count source URL, copy feedback, and short OBS source guidance to Widget Settings. Use the same canonical UUID as chat and highlight. The preview represents the actual three-row source layout and current count behavior. Styling uses existing Tailwind tokens and transparent OBS page background.
- **Storage and scope.** Do not add an interval column or expose the interval through public widget lookup. No viewer history or per-platform audience records are persisted. Do not change chat, pin, highlight, or account connection behavior.

## Testing Decisions

- Start Rust work with failing tests for the viewer-count use case. At the application seam, test all three success paths, disconnected accounts, offline results, absent YouTube count, malformed payload, platform failure, and mixed success/failure. Use fake platform boundaries and a fake widget identity/token boundary; do not mock the use case itself.
- Test HTTP boundary UUID validation, known canonical and legacy aliases, unknown UUID, response shape and `refresh_seconds` without secret leakage. Test that count reads do not acquire a chat widget session or spawn chat workers, and that simultaneous reads coalesce behind one platform fetch.
- At the frontend page/component seam, test all labels, initial zero state, returned numbers, failed reads resetting to zero, immediate first read, response-driven timer, restored-connectivity read, and timer/request cleanup. Test dashboard source URL and copy behavior.
- Test backend configuration at absent, valid boundary, malformed, and out-of-range values, including the 60-second fallback. Test that a cached response retains the backend interval and that cache expiry permits a fresh platform read.
- Verify platform adapters against representative official Twitch and YouTube responses and a captured Kick channel response at an external-HTTP mock boundary. Exercise token refresh and request timeout/error handling.
- Run frontend lint/build, then backend format/Clippy/tests in the required order. Use the previously approved Webpack build substitute only if the same Turbopack port-binding environment failure recurs and the Engineer approves its use for this ticket.

## Out of Scope

- A combined audience total, viewer history, analytics, graphs, or unique-viewer deduplication across platforms.
- Kick OAuth migration, new platform connection UI, or changes to existing chat ingestion.
- Viewer-controlled refresh settings or additional visual customization beyond existing design tokens.
- Treating a displayed zero as proof that a platform observed zero live viewers.

## Further Notes

Twitch Get Streams documents a live stream's `viewer_count` and omits offline users. YouTube documents `liveStreamingDetails.concurrentViewers`, which may be absent when the owner hides the count or no current viewers are present. Kick's existing integration stores an encrypted username and uses a website channel endpoint for chat discovery; its viewer-count response shape and durability require verification during the red/green adapter tests. The Kick Dev API has an authenticated channel route, but changing to it would require a broader connection flow than this ticket.
