# Code Standards

## General

- Keep modules small and single-purpose.
- Fix root causes, do not layer workarounds.
- Apply SOLID principles strictly throughout the codebase.
- Enforce clean architecture patterns to ensure maintainability.

## TypeScript (Frontend)

- Strict mode is required. No exceptions.
- Avoid `any` — use explicit interfaces, `unknown` with type guards, or narrowly scoped generics.
- Validate unknown external input at system boundaries using Zod.
- Prefer `const` over `let`; never use `var`.
- Use async/await exclusively — no raw Promise chains.
- Frontend does not require TDD.

## Rust (Backend)

- Follow TDD (Test-Driven Development) strictly.
- Enforce clean architecture layers (domain, use cases, interfaces, infrastructure).
- Never log sensitive data; user secrets must be encrypted at rest.
- Ensure efficient memory and concurrent connection management for WebSockets.

## Next.js

- Default to server components. Add `'use client'` only when browser interactivity requires it.
- Keep route handlers focused on a single responsibility.
- Use the App Router exclusively.

## Styling

- Use CSS custom property tokens defined in `ui-tokens.md`. No hardcoded hex or oklch values in component code.
- Follow the border radius scale from `ui-tokens.md`.
- Use `cn()` utility for all className merging.
- Apply `.dark` class by default for the dark theme.

## API Routes

- Validate and parse request input before any logic runs.
- Enforce auth and ownership before any mutation.
- Next.js acts as a BFF (Backend for Frontend) or proxy; heavy websocket logic stays in Rust.

## Data and Storage

- Core data lives in Supabase (PostgreSQL).
- Real-time event brokering uses Redis.
- User secrets must be encrypted.

## File Organization

- `frontend/` — Next.js frontend code
- `backend/` — Rust backend code
- `context/` — AI development context files
