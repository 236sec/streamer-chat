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
