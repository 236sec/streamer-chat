# Build Plan

Tracer-bullet tickets. Each ticket is a vertical slice — a narrow but complete path through every layer (schema, API, UI, tests), demoable on its own. Detailed implementation steps for each ticket live in `context/feature-specs/`.

Work the **frontier**: any ticket whose blockers are all done.

---

## Ticket Format

Every ticket uses this structure:

```
## <NN>: <Ticket title>

**What to build:** The end-to-end behaviour this ticket makes work.

**Blocked by:** The numbers/titles of the tickets that gate this one.

**Status:** ready-for-agent | in-progress | done | blocked

- [ ] Acceptance criterion 1
```

---

## 01: Supabase Auth & Dashboard Scaffold

**What to build:** Streamer can log in via Supabase and see an empty dark-mode dashboard with a sidebar.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Next.js app setup with Tailwind & shadcn
- [ ] Supabase Auth integration (login/logout)
- [ ] Basic dashboard layout (sidebar, topbar) matches UI tokens

---

## 02: Rust Backend WebSocket Server Scaffold

**What to build:** Basic Rust backend running with TDD, accepting WebSocket connections and broadcasting ping/pong messages.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Rust project setup with clean architecture
- [ ] WebSocket server setup (e.g. Tokio/Axum)
- [ ] Basic connection test (ping/pong) built via TDD

---

## 03: Next.js Widget Receiving Dummy Messages

**What to build:** A public widget route in Next.js that connects to the Rust backend and displays dummy chat messages on a transparent background.

**Blocked by:** 01: Supabase Auth & Dashboard Scaffold, 02: Rust Backend WebSocket Server Scaffold.

**Status:** blocked

- [ ] Widget configuration DB schema
- [ ] Public widget route `/widget/[id]`
- [ ] WebSocket client in Next.js connecting to Rust server
- [ ] UI component for rendering chat messages

---

## 04: Real Twitch/YouTube/Kick Integration

**What to build:** Connect the Rust backend to real external platform chat feeds and route them to the correct widget.

**Blocked by:** 03: Next.js Widget Receiving Dummy Messages.

**Status:** blocked

- [ ] Dashboard UI to enter/oauth platform credentials (encrypted)
- [ ] Rust backend ingesting real Twitch IRC/WebSocket events
- [ ] Rust backend ingesting YouTube Live Chat events
- [ ] Rust backend ingesting Kick chat events
- [ ] Messages correctly routed to the user's specific widget ID

## 05: Rust Backend Twitch Chat Ingestion

**What to build:** Implement the actual async Twitch IRC/WebSocket client in the Rust backend to consume live chat messages, parse them, and stream them to the connected Next.js widget.

**Blocked by:** 04: Real Twitch/YouTube/Kick Integration

**Status:** done

- [x] Connect to Twitch EventSub or IRC using the decrypted OAuth token.
- [x] Parse incoming Twitch message payloads into the `ChatMessage` schema.
- [x] Stream parsed messages to the live widget connection over WebSocket.
- [x] Dynamically resolve Twitch user ID via Helix API (bugfix: was hardcoded).
- [x] Request correct OAuth scopes (`user:read:chat`) during Twitch connection (bugfix: was missing).
- [x] Allow reconnect/disconnect for already-linked platform identities (bugfix: button was permanently disabled).

---

## 06: Rust Backend YouTube Chat Ingestion

**What to build:** Implement the async YouTube Live Chat API polling loop in the Rust backend.

**Blocked by:** 04: Real Twitch/YouTube/Kick Integration

**Status:** done

- [x] Call YouTube Live Chat API using the decrypted Google OAuth token.
- [x] Parse incoming YouTube chat payloads into the `ChatMessage` schema.
- [x] Stream parsed messages to the live widget connection.
- [x] Handle YouTube live chat startup 404 with retry backoff and video `activeLiveChatId` fallback.
- [x] Maintain worker lifecycle and reconnect reliability across widget page refreshes.

---

## 07: Rust Backend Kick Chat Ingestion

**What to build:** Implement the async Kick WebSocket or API client to consume Kick chat messages.

**Blocked by:** 04: Real Twitch/YouTube/Kick Integration

**Status:** done

- [x] Connect to Kick's chat service.
- [x] Parse incoming Kick message payloads into the `ChatMessage` schema.
- [x] Stream parsed messages to the live widget connection.

---

## 08: Frontend UX and Error Handling

**What to build:** Polish frontend UX/UI with persistent widget configuration, clipboard feedback, interactive onboarding checklist, OBS integration guide, toast notifications, and robust account error handling.

**Blocked by:** 07: Rust Backend Kick Chat Ingestion

**Status:** done

- [x] Toast notification system for transient alerts and copy feedback
- [x] Persistent widget settings stored to Supabase with save status
- [x] Dynamic origin widget URL and copy confirmation state
- [x] Onboarding setup checklist on Dashboard home page
- [x] OBS setup instructions guide modal/card
- [x] Connected accounts error banners, Kick validation, and token health explanations
- [x] Disconnection confirmation dialog

---

## 09: Chat Overlay Enhancements

**What to build:** Message auto-hide lifespan timer and visual layout presets (Card, Bubble, Clean) configurable in the dashboard and rendered in OBS.

**Blocked by:** 08: Frontend UX and Error Handling

**Status:** done

- [x] Add `auto_hide_seconds` and `layout_style` to widget settings state / DB schema
- [x] Implement message expiry timer and smooth fade-out animation in `WidgetClient`
- [x] Add Card, Bubble, and Clean layout styles adhering to UI tokens
- [x] Add auto-hide duration slider/select and layout style selector in widget settings dashboard page
- [x] Live preview in dashboard reflects auto-hide timing and layout preset changes

---

## 10: Message Highlight Overlay & Dashboard Pin Control

**What to build:** Interactive chat feed in dashboard with a "Pin to Screen" action, and a dedicated OBS browser source route at `/widget/[id]/highlight` displaying the featured message.

**Blocked by:** 09: Chat Overlay Enhancements

**Status:** done

- [x] Dedicated public route `/widget/[id]/highlight` in Next.js
- [x] WebSocket event handling for `pin_message` and `unpin_message` broadcasts
- [x] Highlight overlay component with animated entry/exit and high-contrast card styling
- [x] Interactive live chat feed in dashboard allowing streamer to pin and unpin messages
- [x] Highlight widget URL copy button and OBS dimensions guide in dashboard

---

## 13: Backend Widget Session Refactor

**What to build:** Give each active widget one backend session that owns its broadcast channel, platform worker lifecycle, and connection cleanup while preserving existing chat and pin behavior.

**Blocked by:** 10: Message Highlight Overlay & Dashboard Pin Control.

**Status:** done

- [x] Concurrent widget connections share one ingestion run and receive the same widget-scoped feed.
- [x] Final disconnect cancels the matching run; reconnect starts a fresh working session without stale cleanup interference.
- [x] Twitch EventSub, Kick Pusher, and YouTube discovery and chat polling behavior remain unchanged.
- [x] YouTube makes no further broadcast discovery calls during successful chat polling and may rediscover after terminal chat failure.
- [x] Pin snapshot, broadcast, and socket close behavior remain unchanged.
- [x] Backend lifecycle and polling regression tests pass with the approved Webpack build substitute.

---

## 14: Backend Pin State Refactor

**What to build:** Move pin and unpin transitions and reconnect snapshot preparation into the backend application pin module while preserving the existing authorization, revisioned event, and widget-scoped delivery behavior.

**Blocked by:** 13: Backend Widget Session Refactor.

**Status:** done

- [x] Application pin workflow owns validation, persistence coordination, event construction, publication, and snapshot preparation.
- [x] HTTP and WebSocket transport retain authorization, status mapping, socket I/O, and snapshot-first delivery.
- [x] Pin, replacement, unpin, failures, widget isolation, and reconnect ordering retain their current behavior.
- [x] Application-level tests cover transitions using a fake database boundary and the real widget publisher.
- [x] Verification passed with the approved environment-only Webpack build substitute.

---

## 15: Restore Pin Control After Widget Settings Refresh

**What to build:** Keep Widget Settings on the owned widget chosen for OBS across refresh, and let a streamer recover control of an existing widget when their account has multiple rows.

**Blocked by:** 14: Backend Pin State Refactor.

**Status:** done

- [x] Ambiguous widget lookups never create a new row.
- [x] Multiple owned widgets can be selected by UUID with a safe pin preview; the choice survives refresh in the same browser.
- [x] Widget Settings and dashboard home use the same validated selection for URLs and controls.
- [x] Selecting the OBS widget reconnects the pin panel to its persisted pin and unpin action without changing its public URL. Live OBS verification remains outstanding.
- [x] Frontend and backend verification gates pass with the approved environment-only Webpack build substitute.

---

## 16: One Permanent Widget UUID Per Account

**What to build:** Give each account one permanent canonical widget UUID shared by chat and highlight, keep legacy rows and OBS URLs working, and make public UUID lookup non-enumerable.

**Blocked by:** 15: Restore Pin Control After Widget Settings Refresh.

**Status:** done

- [x] Existing single-row accounts retain their UUID; duplicate owners use a validated copied-chat hint when available, then a documented deterministic fallback.
- [x] A private, unique account identity mapping makes legacy chat and highlight URLs use the canonical live stream and pin state after first resolution; legacy rows remain stored.
- [x] Dashboard home and Widget Settings show one canonical identity without a UUID chooser or URL paste step.
- [x] Concurrent account loads cannot create duplicate canonical identities or new duplicate rows; no global backfill or legacy-row deletion is required.
- [x] Public routes resolve a known UUID, while anonymous database clients cannot enumerate widget IDs; owner dashboard reads do not surface legacy rows.
- [x] Existing pin content remains readable and usable; platform tokens remain encrypted.
- [x] Verification and review gates pass for the approved feature spec. Disposable-database and live OBS end-to-end checks remain outstanding.

---

## Verification Commands

| Step | Command | Purpose |
| ---- | ------- | ------- |
| Format (FE) | `npm run lint` | Next.js formatting & linting |
| Build (FE) | `npm run build` | Next.js production build |
| Format (BE) | `cargo fmt -- --check` | Rust formatting |
| Lint (BE) | `cargo clippy -- -D warnings` | Rust linting |
| Test (BE) | `cargo test` | Rust TDD tests |
