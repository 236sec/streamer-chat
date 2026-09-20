# UI Rules

## Component Library

shadcn/ui on top of Tailwind CSS v4. Components live in `frontend/components/ui/`. Use the CLI to add new components rather than writing from scratch.

## Layout Patterns

- **Shell**: Full-viewport layout with a fixed-width left sidebar and scrollable main content area.
- **Sidebar Navigation**: Persistent left sidebar for navigation (Dashboard, Widget Settings, Connected Accounts) with Lucide icons.
- **Top Bar**: Persistent top bar for user profile dropdown and logout actions.
- **Cards**: Used in the main content area for logical groupings (e.g., Connected Accounts, Widget Preview).
- **Modals**: For confirming destructive actions or complex input that doesn't fit on a card.
- **Data Tables**: Standard shadcn tables for displaying connection logs or chat history if needed.
- **Widget Preview**: Clean, transparent-like overlay display simulating OBS integration.

## Icons

Lucide React — stroke-based icons. Sizes: h-4 w-4 for inline, h-5 w-5 for buttons and nav, h-6 w-6 for section headers. All icons inherit currentColor.

## Responsive Behavior

Desktop-first responsive design. The sidebar collapses into a hamburger menu (Sheet component) on mobile devices. Widget route is completely fluid to fit OBS dimensions.

## Component Patterns

- **Forms**: Validation on blur using React Hook Form + Zod, loading states on submit buttons.
- **Empty States**: Centered illustration, title, description, CTA button for connecting the first account.
- **Loading States**: Skeleton loaders matching card shapes, avoiding full-screen spinners.
- **Error States**: Toast notifications for transient errors, inline red text for form field errors.
