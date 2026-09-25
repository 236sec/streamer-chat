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
| Widget Settings | `frontend/src/app/dashboard/widget/page.tsx` | Configures the account's one canonical widget and its OBS chat and highlight sources | Card / Form / Live Preview Pattern |
| ViewerCountClient | `frontend/src/components/widget/ViewerCountClient.tsx` | Shows platform viewer counts in the OBS source and Widget Settings preview | Transparent Overlay / Three-Row Count Pattern |

The public chat widget page marks its document with `.widget-source` so the page background stays transparent. The configured background remains on the inner `WidgetClient` overlay. The Widget Settings page uses the dashboard's main scroller for its controls, preview, and highlight panel.

Tickets 13 and 14 changed backend architecture only; no UI components were added or changed.

### Widget Settings

File: `frontend/src/app/dashboard/widget/page.tsx`
Last updated: 2026-09-24

| Property | Class |
| --- | --- |
| Background | `bg-card` appearance and preview panels; `bg-background/30` preview canvas |
| Border | `border border-border` panels; `border-dashed` preview canvas |
| Border radius | `rounded-lg` panels; `rounded-md` controls and preview canvas |
| Text — primary | `text-foreground` heading and form labels |
| Text — secondary | `text-muted-foreground` descriptions and guidance |
| Text size | `text-xl font-semibold` panel headings; `text-sm` body |
| Spacing | `p-6` panels; `gap-6` panel grid |
| Accent usage | `bg-primary/10 text-primary` selected layout preset |

**Pattern notes:** Resolve one canonical widget before enabling source URLs and pin controls. Show only its chat URL in the copy field. The PinDashboard uses the same UUID for its highlight URL and pin events. There is no widget selector.

### Viewer Count Source

File: `frontend/src/components/widget/ViewerCountClient.tsx`
Last updated: 2026-09-25

| Property | Class |
| --- | --- |
| Background | `bg-transparent` source; `bg-card/80` count rows |
| Border | `border border-border` rows |
| Border radius | `rounded-md` rows |
| Text | `text-foreground` source; `text-sm font-medium` labels; `font-mono text-lg tabular-nums` values |
| Spacing | `gap-2` between rows; `px-4 py-3` within rows |

**Pattern notes:** Keep Twitch, YouTube, and Kick rows visible in that order. Show zero for missing or failed counts. The public source uses `.widget-source` for a transparent page; the dashboard preview embeds the same component.
