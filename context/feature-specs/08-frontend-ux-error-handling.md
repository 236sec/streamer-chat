# Feature Spec — 08: Frontend UX and Error Handling

## Problem Statement

Streamers setting up and operating their chat overlay encounter multiple friction points and confusing states in the web interface:

1. Modifying widget appearance in the dashboard only alters the in-browser preview without persisting to the database. When pasted into OBS, the widget loads default unstyled configurations.
2. The widget URL uses a hardcoded localhost address rather than the live host domain, breaking OBS browser sources deployed outside local environments.
3. Copying the widget URL gives no visual or auditory confirmation, leaving users uncertain if the clipboard copy succeeded.
4. Streamers lack clear, in-app instructions for configuring OBS browser sources with recommended dimensions and settings.
5. The dashboard home page displays static navigation links without indicating what steps remain to get chat streaming into OBS.
6. Account management operations fail silently or provide cryptic warnings. Kick connection failures log to the developer console without notifying the user, OAuth redirect errors are dropped, and refresh token warnings provide no explanation of token expiry risks or remedy steps.
7. Account disconnection has no confirmation gate, allowing accidental disconnection of active stream sources.

## Solution

Deliver a cohesive frontend UX and error-handling suite that guides streamers through onboarding, persists their settings, and communicates system status with clarity:

1. Persist widget appearance settings to Supabase with an explicit save mechanism and status feedback.
2. Provide a lightweight toast notification system for transient confirmations and error alerts across the dashboard.
3. Introduce an onboarding progress checklist on the dashboard home page tracking account connection, widget customization, and OBS integration.
4. Add an OBS setup guide with step-by-step instructions and recommended dimensions accessible from the dashboard and widget configuration pages.
5. Improve the connected accounts interface with inline error banners, clear token health explanations, Kick username validation, and disconnection confirmation dialogs.
6. Provide clear visual distinction between simulated preview data and live stream overlays.

## User Stories

1. As a streamer, I want to save my widget theme, font size, and background color to the database, so that OBS renders my custom styling instead of default fallback values.
2. As a streamer, I want my widget URL to reflect my active host domain, so that the copied link functions in OBS on both local and production deployments.
3. As a streamer, I want clear visual confirmation when I copy my widget URL, so that I know the URL is in my clipboard before switching to OBS.
4. As a streamer, I want step-by-step instructions inside the dashboard for adding the browser source to OBS, so that I do not need external documentation to configure source dimensions and visibility rules.
5. As a streamer, I want an onboarding checklist on the dashboard home page, so that I can see at a glance which setup steps remain before my stream goes live.
6. As a streamer, I want to see a helpful error message when a Kick connection fails, so that I know whether the username was invalid or the service was unreachable.
7. As a streamer, I want OAuth authorization errors displayed on the accounts page, so that I know why Twitch or YouTube authorization was not completed.
8. As a streamer, I want an understandable explanation of missing refresh tokens, so that I understand why reconnecting is necessary to prevent stream disconnects.
9. As a streamer, I want a confirmation step before disconnecting a platform account, so that I do not accidentally disconnect a live chat source during a stream.
10. As a streamer, I want to clearly distinguish between simulated test chat in the dashboard preview and real incoming messages, so that I do not mistake preview messages for live viewers.

## Implementation Decisions

- **D1: Toast Notification System**:
  - Implement a centralized toast notification provider and custom hook.
  - Position toasts in the viewport corner adhering to design tokens for background, borders, and text.
  - Support semantic toast variants: success, error, info.
  - Automatically dismiss toasts after 4000 milliseconds with optional manual dismissal.
  - Avoid heavy external dependencies by implementing a lightweight React context and hook compatible with existing shadcn tokens and Tailwind styling.

- **D2: Widget Configuration Persistence**:
  - Extend widget retrieval in the widget settings page to load existing `theme`, `font_size`, and `background_color` attributes alongside the widget ID from the `widgets` table.
  - Add an explicit "Save Changes" button with pending and disabled states during mutation.
  - Execute a Supabase update mutation on user submission targeting the user's widget record.
  - Trigger a success toast upon successful update, or an error toast with actionable text if the update query fails.

- **D3: Dynamic Widget URL and Clipboard Feedback**:
  - Construct the widget URL using `window.location.origin` dynamically in the client rather than hardcoding `http://localhost:3000`.
  - When copying, trigger `navigator.clipboard.writeText`, toggle the button icon to a checkmark for 2000 milliseconds, and dispatch a success toast confirming copy completion.

- **D4: OBS Setup Guide**:
  - Build an OBS Setup Guide modal and card component.
  - Outline explicit sequential steps:
    1. Add Source -> Browser in OBS Studio.
    2. Enter Source Name (e.g. "StreamSync Chat").
    3. Paste Widget URL into URL input.
    4. Set Width to 400 (or custom) and Height to 600 (or custom).
    5. Check "Shutdown source when not visible" and "Refresh browser when scene becomes active".
  - Make the guide accessible via a prominent button on both the dashboard overview and widget settings pages.

- **D5: Dashboard Onboarding Checklist**:
  - Transform the dashboard home page from static navigation cards to an interactive 3-step setup tracker:
    - Step 1: Connect Accounts (displays connected platform count and badge indicators; links to `/dashboard/accounts`).
    - Step 2: Customize Overlay (shows current saved theme and font size; links to `/dashboard/widget`).
    - Step 3: Add to OBS (provides 1-click URL copy and opens OBS Setup Guide).
  - Calculate overall completion percentage based on whether at least one account is connected and whether the widget has been generated.

- **D6: Accounts Management Error Handling and Feedback**:
  - Clean and validate Kick usernames before submission (strip leading `@`, verify non-empty, reject whitespace).
  - Display inline field errors on Kick modal submission failures based on backend JSON response payloads (`{ error: string }`).
  - Read query parameters on `/dashboard/accounts` for `error` and `error_description` sent by Supabase or external OAuth providers, rendering a prominent alert banner at the top of the page.
  - Replace ambiguous refresh token warnings with a structured warning badge and explanatory tooltip detailing that offline refresh tokens allow continuous streaming without hourly re-authentication.
  - Add a confirmation dialog before executing platform account deletion to prevent accidental disconnects.
  - Fire success toasts upon account connection and disconnection.

- **D7: Preview versus Live State Demarcation**:
  - Add an overlay pill badge labeled "Preview Mode — Simulated Chat" in the widget settings live preview frame.
  - Ensure widget errors broadcast over WebSocket (e.g. platform token expiration) render with accessible warning banners matching design tokens.

## Testing Decisions

- **T1: Widget Persistence and State Hydration**:
  - Verify that the widget settings page populates theme, font size, and background color from Supabase query data.
  - Verify that clicking "Save Changes" invokes the Supabase update mutation with current form state and triggers the success toast.
  - Verify that the public widget route `/widget/[id]` displays the persisted attributes.

- **T2: Copy Feedback and Dynamic URL**:
  - Verify that copying writes the dynamic origin URL to clipboard and changes the copy button visual state.

- **T3: Account Validation and Error Handling**:
  - Verify that Kick submission with empty or invalid usernames produces an inline error and does not dispatch API requests.
  - Verify that an API failure during Kick submission renders the error message returned by the server and shows an error toast.
  - Verify that navigating to `/dashboard/accounts?error=access_denied&error_description=User+cancelled` renders a dismissible alert banner.
  - Verify that clicking Disconnect presents a confirmation dialog before token removal.

- **T4: Verification Pipeline Compatibility**:
  - Next.js build (`npm run build`) and lint (`npm run lint`) must pass with strict TypeScript checks and zero lint errors.

## Out of Scope

- Adding real-time bi-directional chat messaging or chat input forms from the dashboard.
- Custom CSS upload or arbitrary code execution in the widget.
- Backend schema changes (the existing `widgets` and `platform_tokens` schema already contain all required fields).
- External moderation tooling or user ban management.

## Further Notes

- All colors and borders must strictly utilize CSS variables defined in `ui-tokens.md` (`--background`, `--card`, `--border`, `--primary`, `--destructive`, `--muted-foreground`).
- Component structures must follow patterns established in `ui-registry.md` and `ui-rules.md`.
