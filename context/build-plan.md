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

**Status:** ready-for-agent

- [ ] Connect to Kick's chat service.
- [ ] Parse incoming Kick message payloads into the `ChatMessage` schema.
- [ ] Stream parsed messages to the live widget connection.

---

## Verification Commands

| Step | Command | Purpose |
| ---- | ------- | ------- |
| Format (FE) | `npm run lint` | Next.js formatting & linting |
| Build (FE) | `npm run build` | Next.js production build |
| Format (BE) | `cargo fmt -- --check` | Rust formatting |
| Lint (BE) | `cargo clippy -- -D warnings` | Rust linting |
| Test (BE) | `cargo test` | Rust TDD tests |
