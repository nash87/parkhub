# Accessibility

## Colorblind Modes

Three simulation modes that remap the color palette for better distinction:

| Mode | Affects | What it does |
|------|---------|-------------|
| Protanopia | Red-blind (~1% of males) | Removes red-green reliance, shifts to blue-yellow distinctions |
| Deuteranopia | Green-blind (~6% of males) | Similar to protanopia but different cone response curve |
| Tritanopia | Blue-yellow blind (rare) | Shifts to red-green distinctions |

These are applied as CSS filter overlays. They don't change the palette — they transform rendered colors. Toggle in **Profile → Accessibility**.

## Font Scaling

Four sizes, applied by changing the root `font-size`. Everything uses `rem` units, so the whole UI scales:

| Size | Base | Effect |
|------|------|--------|
| Small | 14px | Fits more on screen |
| Medium | 16px | Browser default |
| Large | 18px | Easier to read |
| Extra Large | 20px | For users who need it |

Stored in localStorage, applied on page load.

## Reduced Motion

Two triggers:

1. **OS-level:** If the user has `prefers-reduced-motion: reduce` set in their OS, Framer Motion animations are automatically disabled.
2. **Manual toggle:** In Profile → Accessibility, users can force-disable animations regardless of OS setting.

When active: no page transitions, no list item animations, no parallax, no spring physics. Instant state changes only.

## High Contrast

Increases contrast ratios across the UI:

- Borders go from subtle gray to solid dark
- Text gets bolder weight
- Background-foreground contrast meets WCAG AAA (7:1 ratio)
- Focus indicators are thicker and higher-contrast

## Keyboard Navigation

Every interactive element is reachable via `Tab`. Focus order follows visual layout. Key bindings:

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Move focus forward/backward |
| `Enter` or `Space` | Activate buttons, toggle checkboxes |
| `Escape` | Close modals, dropdowns, overlays |
| Arrow keys | Navigate within menus, date pickers, slot grids |

Focus rings are always visible when navigating by keyboard (hidden on mouse click via `:focus-visible`).

## ARIA

- All form inputs have `<label>` associations (via `htmlFor` or wrapping)
- Icon-only buttons have `aria-label` (e.g., the dark mode toggle)
- Navigation landmarks use `<nav aria-label="...">`
- Toast notifications use `aria-live="polite"` regions
- Modals use `role="dialog"` with `aria-modal="true"` and focus trapping
- Loading states announce via `aria-busy`

---

Back to [README](../README.md) · Previous: [Themes](THEMES.md) · Next: [Security](SECURITY.md)
