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

## Storage Model

- **Supabase PostgreSQL**: User accounts, platform integration tokens (encrypted), widget configurations.
- **Redis**: Real-time message brokering, short-lived session states, rate limiting.

## Auth and Access Model

- Authentication is managed via Supabase Auth (JWT).
- User secrets (tokens for Twitch/YouTube/Kick) must be encrypted at rest and never logged in plaintext.
- The OBS widget route (e.g. `/widget/:id`) is public and requires no active session, relying solely on a secure, non-guessable UUID for access.

## Invariants

1. The Next.js frontend must never communicate directly with the Twitch/YouTube/Kick APIs for chat streams; all chat events must be ingested and normalized by the Rust backend.
2. The Rust backend must never store or log plaintext user secrets or tokens.
3. The OBS widget route must be completely public using a secure, non-guessable UUID for access.
4. All UI components in the Next.js app must use Tailwind CSS for styling; no custom CSS files should be created outside of the global stylesheet.
5. The codebase must follow clean architecture and SOLID principles.
6. The Rust backend must follow TDD (Test-Driven Development).
