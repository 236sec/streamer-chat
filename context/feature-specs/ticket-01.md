## Problem Statement
The streamer needs a secure way to log into the application to access their dashboard, and a foundational user interface that provides navigation and layout for future widget configuration and integrations. 

## Solution
We will scaffold a Next.js application (App Router) integrated with Tailwind CSS and shadcn/ui. We will implement authentication using Supabase (`@supabase/ssr` and `@supabase/auth-ui-react`). A protected `/dashboard` route will be created, featuring a persistent shell layout with a sidebar and topbar styled according to our dark-mode UI tokens. 

## User Stories
1. As a streamer, I want to access a login page so that I can securely authenticate using my existing accounts (via Supabase Auth).
2. As a streamer, I want unauthenticated access to be redirected to the login page so that my dashboard remains private.
3. As a streamer, I want to see a unified dashboard layout with a sidebar and top bar so that I can easily navigate the application.
4. As a streamer, I want to log out of my account so that my session ends securely.

## Implementation Decisions
- **D1: Supabase Auth Protection & Environment Validation**
  - We will use Supabase's middleware approach (`middleware.ts`) to intercept requests and redirect unauthenticated users from `/dashboard` to `/login`.
  - We will implement `@supabase/ssr` for server, client, and middleware clients.
  - We will validate environment variables (`NEXT_PUBLIC_SUPABASE_URL` and `NEXT_PUBLIC_SUPABASE_ANON_KEY`) using Zod on application startup.
- **D2: Authentication UI**
  - We will use `@supabase/auth-ui-react` and `@supabase/auth-ui-shared` for the login page to easily implement the authentication UI, rather than building custom forms with shadcn/ui.
- **D3: Next.js Layout & UI**
  - We will configure Tailwind CSS v4 to map the design tokens from `ui-tokens.md` (e.g., `--background`, `--card`, `--primary`).
  - The application will be strictly dark mode by default (`.dark` class applied).
  - The dashboard will implement the "Shell" pattern with a collapsible left sidebar (using Lucide icons) and a top bar for user actions.

## Testing Decisions
- Frontend does not require TDD as per `code-standards.md`.
- Manual verification will be done to ensure Zod environment validation works.
- Manual verification will be done to ensure `middleware.ts` correctly blocks access to `/dashboard` when logged out.
- Manual verification will ensure the UI matches the color tokens.

## Out of Scope
- Actual OAuth provider configuration in the Supabase project dashboard (this requires manual developer setup).
- Database schema and widget configuration functionality (handled in Ticket 03).
- Real-time chat WebSocket ingestion (handled in Ticket 02).

## Further Notes
- The Next.js app should use Server Components by default, dropping to `'use client'` only for interactive layout pieces like the mobile hamburger menu or the auth UI.
