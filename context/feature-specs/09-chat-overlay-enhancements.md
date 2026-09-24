# Feature Spec — 09: Chat Overlay Enhancements

## Problem Statement

Streamers using the chat overlay in OBS face display limitations during active broadcasts:

1. Messages persist indefinitely until pushed off screen by newer messages. On slower chats, stale messages linger on stream for hours, cluttering the broadcast.
2. The current chat message display only supports a single generic card visual style. Streamers with different broadcast themes (e.g. minimalist games vs. lively community streams) cannot adapt the layout structure to their stream aesthetic.
3. The widget settings page only allows configuring font size, theme (dark/light), and background color, with no controls for message lifespan or message layout styles.

## Solution

Enhance the chat overlay and dashboard settings with message lifecycle controls and visual layout presets:

1. Add an `auto_hide_seconds` setting (0 = permanent, 5s, 10s, 15s, 30s, 60s) with smooth CSS fade-out animation when messages expire.
2. Provide three layout presets:
   - `card`: Current card style with background and border.
   - `bubble`: Rounded pill badge appearance with compact padding.
   - `clean`: Transparent background with text shadow for unobtrusive game overlays.
3. Update Supabase schema and widget settings dashboard to configure and preview auto-hide and layout presets in real time.
4. Update `WidgetClient.tsx` to handle auto-hide timers and layout preset classes in both live OBS sources and simulated dashboard preview.

## User Stories

1. As a streamer, I want messages to automatically fade out after a configured number of seconds, so that my stream screen stays clean when chat is inactive.
2. As a streamer, I want to disable message auto-hiding (set to 0 / never), so that my stream keeps the latest messages visible if I prefer permanent history.
3. As a streamer, I want to choose between Card, Bubble, and Clean layout presets in the dashboard, so that the chat display matches my stream's visual identity.
4. As a streamer, I want the dashboard Live Preview to reflect the auto-hide timing and layout preset immediately as I test different configurations.
5. As a streamer, I want my auto-hide and layout preset settings to persist to the database when I click "Save Changes", so that my OBS browser source displays them reliably across restarts.

## Implementation Decisions

- **D1: Message Expiry Tracking**:
  - Assign each incoming message a creation timestamp upon arrival in `WidgetClient.tsx`.
  - When `auto_hide_seconds > 0`, calculate expiry time and trigger a transition to `opacity-0 scale-95` before removing or hiding the message.
  - Implement smooth CSS transition (`transition-opacity transition-transform duration-500`) to prevent jarring visual pop-out.

- **D2: Layout Style Tokens and Classes**:
  - Define three distinct layout classes adhering to Tailwind and design tokens in `context/ui-tokens.md`:
    - `card`: `bg-card text-card-foreground border border-border/50 rounded shadow-sm px-4 py-2`
    - `bubble`: `bg-secondary text-secondary-foreground rounded-2xl shadow px-3 py-1.5`
    - `clean`: `bg-transparent text-foreground drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)] px-2 py-1`
  - Ensure platform badges (Twitch, YouTube, Kick) and author colors render consistently across all layout presets.

- **D3: Schema & Persistence**:
  - Extend `widgets` table in Supabase (or fallback gracefully to local storage / state defaults) with:
    - `auto_hide_seconds`: INTEGER DEFAULT 0
    - `layout_style`: TEXT DEFAULT 'card'
  - Update `frontend/src/app/dashboard/widget/page.tsx` with a slider/select for auto-hide duration and a segmented selector for layout presets.

## Out of Scope

- Sound alerts on message arrival or mentions (explicitly excluded).
- Custom CSS upload / arbitrary stylesheet injection.
- Message pinning to a secondary overlay (deferred to Ticket 10).
