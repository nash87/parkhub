# Accessibility

ParkHub is designed to be usable by everyone. This document covers the built-in accessibility features.

## Colorblind Modes

ParkHub includes three colorblind simulation modes that adjust the UI palette for better contrast:

| Mode | Condition | Description |
|------|-----------|-------------|
| Protanopia | Red-blind | Reduces reliance on red-green distinctions |
| Deuteranopia | Green-blind | Alternative color mappings for green-deficient vision |
| Tritanopia | Blue-yellow blind | Adjusted palette for blue-yellow deficiency |

Enable via **Profile → Accessibility → Colorblind Mode**.

## Font Scaling

Users can adjust the base font size from the profile settings:

- **Small** (14px)
- **Medium** (16px, default)
- **Large** (18px)
- **Extra Large** (20px)

The entire UI scales proportionally using `rem` units.

## Reduced Motion

For users who prefer minimal animation:

- Respects the `prefers-reduced-motion` media query automatically
- Can be toggled manually in **Profile → Accessibility**
- Disables Framer Motion animations, transitions, and parallax effects

## High Contrast

High contrast mode increases color contrast ratios across the UI:

- Sharper borders and outlines
- Bolder text rendering
- Increased contrast between foreground and background
- Meets WCAG AAA standards when enabled

## ARIA Labels

All interactive elements include proper ARIA attributes:

- Form inputs have associated `<label>` elements
- Buttons include `aria-label` for icon-only controls
- Navigation uses `<nav>` landmarks with `aria-label`
- Status messages use `aria-live` regions
- Modals include `role="dialog"` with `aria-modal`

## Keyboard Navigation

The entire application is navigable via keyboard:

| Key | Action |
|-----|--------|
| `Tab` | Move to next interactive element |
| `Shift+Tab` | Move to previous element |
| `Enter` / `Space` | Activate buttons and links |
| `Escape` | Close modals and dropdowns |
| Arrow keys | Navigate within menus and date pickers |

Focus indicators are visible on all interactive elements.

---

Back to [README](../README.md) · Previous: [Themes](THEMES.md) · Next: [Security](SECURITY.md)
