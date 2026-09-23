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
| WidgetClient | `frontend/src/components/widget/WidgetClient.tsx` | Connects to WS and renders dummy chat messages for OBS | Custom Hook |
| Toast | `frontend/src/components/ui/toast.tsx` | Lightweight toast notification provider and useToast hook | Context / Portal Pattern |
| ObsSetupGuideModal | `frontend/src/components/obs/ObsSetupGuideModal.tsx` | Step-by-step OBS browser source setup guide modal | Modal / Dialog Pattern |

