# Bugfix Spec — Widget Display Regressions

## Problem Statement

The Widget Settings page no longer lays out its appearance controls, live preview, and message highlight section reliably. The public normal chat widget also shows an opaque page background when its configured background is transparent, covering the OBS scene behind it.

## Solution

Restore a stable, scrollable settings page with the existing controls and live preview visible at desktop and narrow widths. Make the public normal widget document transparent when the widget background setting is transparent, while preserving an explicitly configured background color and the dashboard's dark surfaces.

## User Stories

1. As a streamer, I want the Widget Settings controls and preview to remain readable and usable after the Message Highlight section appears, so that I can configure the overlay.
2. As a streamer, I want the live preview to fit within the settings page without clipping or covering other controls, so that I can judge its appearance.
3. As a streamer, I want a transparent chat widget URL to leave my OBS scene visible around chat messages, so that the overlay works as intended.
4. As a streamer, I want my selected widget background color to remain visible when it is not transparent, so that existing appearance settings still work.
5. As a streamer, I want the separate highlight source to retain its transparent background, so that its existing behavior does not regress.

## Implementation Decisions

- **D1 — Scope and terms:** “Widget Settings” is the authenticated configuration page, including its simulated preview and Message Highlight panel. “Normal widget” is the public chat source, separate from the public highlight source. This repair preserves current configuration and pin behavior.
- **D2 — Settings layout:** Remove the height dependency that lets the added highlight panel compress the settings grid. Give the settings and preview panels natural minimum space and let the dashboard's existing main scroller carry the longer page. Keep the preview's simulated OBS dimensions and its own overflow behavior.
- **D3 — Overlay transparency:** Scope document-level transparency to public widget routes. Both the root page background and inner widget background must allow transparency for the default transparent setting. Do not make the dashboard document transparent. Preserve an explicit, nontransparent widget background on the inner overlay.
- **D4 — Styling:** Use existing design tokens, Tailwind classes, and the global stylesheet for any document-level rule. Do not introduce raw colors or change the design system.
- **D5 — Data:** No schema, API, WebSocket, or backend change is needed. The widget table already defaults background color to `transparent`.
- **D6 — Framework guidance:** Before changing a Next.js page or route, read the relevant installed Next.js 16.3.5 guide required by the frontend agent instructions.

## Testing Decisions

- Visually inspect Widget Settings at desktop and narrow viewport sizes, both before and after its Message Highlight panel loads. Verify controls, preview, and highlight section remain readable and scrollable without overlap.
- Open a public normal widget with its default transparent setting over a contrasting test surface and verify transparent pixels outside chat messages. Repeat with an explicit background color and verify that color is preserved. Verify the separate highlight source remains transparent and the dashboard retains its dark background.
- Run the full required verification sequence: frontend lint, frontend build, backend format check, backend Clippy, and backend tests. Frontend visual behavior is the relevant test seam; no new test framework or backend test is needed for this CSS/layout repair.

## Out of Scope

- New widget controls, themes, or layout presets.
- Changes to chat ingestion, pinning, persistence, or widget URLs.
- Redesign of the settings page or highlight panel.

## Further Notes

- The latest Message Highlight change appended a new panel after a `flex-1 min-h-0` settings grid inside a `h-full` flex column. That combination gives the grid permission to collapse when the additional panel takes height, despite the preview having a 500 px minimum height.
- The normal widget renders a transparent inner element, but the shared global body rule paints the page background. A later transparency rule covers only the highlight source, leaving the normal source opaque. The original public widget spec explicitly requires a transparent OBS page.
