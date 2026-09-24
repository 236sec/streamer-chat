# UI Registry

Tracks every UI component built.

| Component | Path | Purpose | Patterns Used |
| --------- | ---- | ------- | ------------- |
| Avatar | `frontend/src/components/ui/avatar.tsx` | User profile icon in topbar | shadcn/ui |
| Button | `frontend/src/components/ui/button.tsx` | Interactive clickable elements | shadcn/ui |
| Card | `frontend/src/components/ui/card.tsx` | Main container for grouped content | shadcn/ui |
| DropdownMenu | `frontend/src/components/ui/dropdown-menu.tsx` | User settings menu on topbar | shadcn/ui |
| Input | `frontend/src/components/ui/input.tsx` | Form text fields | shadcn/ui |
| Label | `frontend/src/components/ui/label.tsx` | Form labels for inputs | shadcn/ui |
| Sheet | `frontend/src/components/ui/sheet.tsx` | Mobile-responsive sidebar drawer | shadcn/ui |
| Topbar | `frontend/src/components/layout/topbar.tsx` | Persistent top navigation bar | Shell Pattern |
| Sidebar | `frontend/src/components/layout/sidebar.tsx` | Persistent left navigation menu | Shell Pattern |
| WidgetClient | `frontend/src/components/widget/WidgetClient.tsx` | Connects to WS and renders chat overlay with auto-hide lifecycle and layout presets (card, bubble, clean) for OBS | Custom Hook / Transition Pattern |
| Toast | `frontend/src/components/ui/toast.tsx` | Lightweight toast notification provider and useToast hook | Context / Portal Pattern |
| ObsSetupGuideModal | `frontend/src/components/obs/ObsSetupGuideModal.tsx` | Step-by-step OBS browser source setup guide modal | Modal / Dialog Pattern |
| PinDashboard | `frontend/src/components/widget/PinDashboard.tsx` | Live dashboard feed with pin controls, status, highlight URL, and OBS size guidance | Card / Live Feed Pattern |
| HighlightClient | `frontend/src/components/widget/HighlightClient.tsx` | Public OBS overlay for the active pinned message with entry and exit transitions | Transparent Overlay / Transition Pattern |
| Widget selector | `frontend/src/app/dashboard/widget/page.tsx` | Selects an existing owned widget by UUID to restore OBS and pin controls | Card / Selectable Row Pattern |

The public chat widget page marks its document with `.widget-source` so the page background stays transparent. The configured background remains on the inner `WidgetClient` overlay. The Widget Settings page uses the dashboard's main scroller for its controls, preview, and highlight panel.

Tickets 13 and 14 changed backend architecture only; no UI components were added or changed.

### Widget selector

File: `frontend/src/app/dashboard/widget/page.tsx`
Last updated: 2026-09-24

| Property | Class |
| --- | --- |
| Background | `bg-card` container; `bg-background` unselected row; `bg-primary/10` selected row |
| Border | `border border-border` container and unselected row; `border-primary` selected row |
| Border radius | `rounded-lg` container; `rounded-md` row |
| Text — primary | `text-foreground` UUID |
| Text — secondary | `text-muted-foreground` instruction and pin preview |
| Text size | `text-xl font-semibold` heading; `text-sm` body and row text |
| Spacing | `p-6 space-y-4` container; `p-4` row; `space-y-2` row list |
| Hover state | `hover:bg-secondary` unselected row |
| Accent usage | `border-primary bg-primary/10` selected row |

**Pattern notes:** Show the full UUID in `font-mono` so streamers can match an existing OBS source. Show only validated pin text in the secondary line. Keep source URLs and pin controls inactive until a widget is selected.
