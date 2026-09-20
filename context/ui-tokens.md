# UI Tokens

## Theme

Dark mode by default, tailored for streamers who typically prefer dark UI. The aesthetic is sleek and modern, utilizing slate gray backgrounds with vibrant purple accents. No hex values in this section.

## Colors

All components must use these CSS custom property tokens. No hardcoded hex or oklch values in component code.

### Base Tokens

| Role              | CSS Variable              | Value (oklch or hex)       | Usage                          |
| ----------------- | ------------------------- | -------------------------- | ------------------------------ |
| Page background   | `--background`            | `oklch(20% 0.01 250)`      | Root page background           |
| Text primary      | `--foreground`            | `oklch(95% 0.01 250)`      | Primary content text           |
| Card background   | `--card`                  | `oklch(25% 0.01 250)`      | Card and panel surfaces        |
| Card text         | `--card-foreground`       | `oklch(95% 0.01 250)`      | Text on cards                  |
| Popover bg        | `--popover`               | `oklch(25% 0.01 250)`      | Dropdowns, tooltips            |
| Popover text      | `--popover-foreground`    | `oklch(95% 0.01 250)`      | Text in popovers               |
| Primary accent    | `--primary`               | `oklch(65% 0.2 290)`       | Primary buttons, active states |
| Primary text      | `--primary-foreground`    | `oklch(98% 0 0)`           | Text on primary backgrounds    |
| Secondary surface | `--secondary`             | `oklch(30% 0.01 250)`      | Secondary buttons, muted surfaces |
| Secondary text    | `--secondary-foreground`  | `oklch(85% 0.01 250)`      | Text on secondary backgrounds  |
| Muted surface     | `--muted`                 | `oklch(35% 0.01 250)`      | Muted backgrounds              |
| Muted text        | `--muted-foreground`      | `oklch(70% 0.01 250)`      | Secondary text, captions       |
| Accent surface    | `--accent`                | `oklch(60% 0.2 290)`       | Hover states                   |
| Accent text       | `--accent-foreground`     | `oklch(98% 0 0)`           | Text on accent backgrounds     |
| Borders           | `--border`                | `oklch(35% 0.01 250)`      | Card borders, dividers         |
| Input border      | `--input`                 | `oklch(40% 0.01 250)`      | Form input borders             |
| Focus ring        | `--ring`                  | `oklch(65% 0.2 290)`       | Focus indicators               |
| Destructive       | `--destructive`           | `oklch(60% 0.2 20)`        | Delete buttons, error states   |

## Typography

| Role       | Font           | CSS Variable      | Notes                          |
| ---------- | -------------- | ----------------- | ------------------------------ |
| Headings   | Inter          | `--font-heading`  | Section titles, page headers   |
| UI / body  | Inter          | `--font-sans`     | Primary sans-serif for all UI  |
| Fallback   | sans-serif     | `--font-sans`     | System fallback                |
| Code/mono  | JetBrains Mono | `--font-mono`     | Code blocks, data values       |

## Border Radius

| Context              | Token / Class       | Value      |
| -------------------- | ------------------- | ---------- |
| Inline / small UI    | `--radius-sm`       | `0.125rem` |
| Buttons, inputs      | `--radius-md`       | `0.25rem`  |
| Cards, panels        | `--radius-lg`       | `0.5rem`   |
| Modals, dialogs      | `--radius-xl`       | `0.75rem`  |
| Large containers     | `--radius-2xl`      | `1rem`     |
