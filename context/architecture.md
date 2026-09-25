# Architecture Context

## Stack

| Layer             | Technology                  | Role   |
| ----------------- | --------------------------- | ------ |
| Frontend          | Next.js + TypeScript        | Dashboard and OBS widget rendering |
| UI                | Tailwind CSS + shadcn/ui    | Styling and components |
| Icons             | Lucide React                | Iconography |
| Backend           | Rust                        | Real-time WebSocket aggregation |
| Auth / DB         | Supabase (PostgreSQL)       | Authentication, user profiles, and config |
| Cache             | Redis                       | Message routing / pub-sub between Rust and Next.js / OBS |

## System Boundaries

- `frontend/` — Next.js frontend, responsible for user auth, dashboard UI, widget configuration UI, and rendering the OBS widget overlay.
- `backend/` — Rust backend, responsible for maintaining persistent WebSocket connections to external platforms (Twitch, Kick, YouTube), parsing incoming messages, and broadcasting them.

The separate public viewer-count source reads a narrow HTTP endpoint through the Next.js BFF. The Rust application service owns canonical-widget count orchestration and coalescing; infrastructure adapters own token lookup/decryption and platform HTTP calls. The backend environment variable `VIEWER_COUNT_REFRESH_SECONDS` controls the source's read interval, defaulting to 60 seconds. Count reads do not start chat ingestion workers, and no viewer history is stored.

Within the Rust backend, one widget session owns each active widget's broadcast channel, ingestion worker run, and final-connection cleanup. The application pin module owns validated pin transitions and persisted snapshot preparation. The web transport owns authorization, status mapping, and socket I/O; the PostgreSQL adapter owns pin storage queries. Widget subscribers register before snapshot loading, and the snapshot is sent before queued live events.

An OBS socket opened before its account's first identity mapping checks for that mapping every five seconds. Once mapping exists, it acquires the canonical session, sends the canonical pin snapshot, and releases the old session and worker without requiring an OBS refresh. The checks stop after mapping. Existing widget worker cancellation and YouTube polling lifecycle remain owned by the session.

## Storage Model

- **Supabase PostgreSQL**: User accounts, platform integration tokens (encrypted), widget configurations, and a private unique account-to-canonical-widget mapping. Legacy widget rows remain as aliases. An authenticated transactional resolver creates or chooses the permanent canonical UUID, validates an optional copied-chat hint, and reconciles pin state. A scoped public lookup resolves one known UUID to overlay fields without granting table-wide public reads.
- **Redis**: Real-time message brokering, short-lived session states, rate limiting.

## Auth and Access Model

- Authentication is managed via Supabase Auth (JWT).
- User secrets (tokens for Twitch/YouTube/Kick) must be encrypted at rest and never logged in plaintext.
- The OBS widget route (e.g. `/widget/:id`) is public and requires no active session, relying solely on a secure, non-guessable UUID for access.
- Each account has one mapped canonical widget UUID. The dashboard resolves it through an authenticated database function and uses it for chat, highlight, appearance, and pin controls. Public legacy URLs resolve to the canonical row after mapping. Backend widget subscriptions canonicalize before acquiring a session or switch after first mapping, and pin mutations require the canonical UUID. Authenticated table updates are limited to canonical appearance columns; pin state is written through the authorized backend command path.

## Invariants

1. The Next.js frontend must never communicate directly with the Twitch/YouTube/Kick APIs for chat streams; all chat events must be ingested and normalized by the Rust backend.
2. The Rust backend must never store or log plaintext user secrets or tokens.
3. The OBS widget route must be completely public using a secure, non-guessable UUID for access.
4. All UI components in the Next.js app must use Tailwind CSS for styling; no custom CSS files should be created outside of the global stylesheet.
5. The codebase must follow clean architecture and SOLID principles.
6. The Rust backend must follow TDD (Test-Driven Development).
