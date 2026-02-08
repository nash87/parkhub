# Color Themes

ParkHub ships with 10 carefully curated color themes, each with light and dark variants.

## Available Themes

### Default Blue

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#3B82F6` | `#6366F1` | `#0EA5E9` | `#F9FAFB` | `#FFFFFF` |
| Dark | `#3B82F6` | `#6366F1` | `#0EA5E9` | `#030712` | `#111827` |

### Solarized

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#268BD2` | `#2AA198` | `#B58900` | `#FDF6E3` | `#EEE8D5` |
| Dark | `#268BD2` | `#2AA198` | `#B58900` | `#002B36` | `#073642` |

### Dracula

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#BD93F9` | `#FF79C6` | `#50FA7B` | `#F8F8F2` | `#FFFFFF` |
| Dark | `#BD93F9` | `#FF79C6` | `#50FA7B` | `#282A36` | `#44475A` |

### Nord

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#5E81AC` | `#88C0D0` | `#81A1C1` | `#ECEFF4` | `#E5E9F0` |
| Dark | `#88C0D0` | `#81A1C1` | `#5E81AC` | `#2E3440` | `#3B4252` |

### Gruvbox

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#D79921` | `#689D6A` | `#D65D0E` | `#FBF1C7` | `#EBDBB2` |
| Dark | `#D79921` | `#689D6A` | `#D65D0E` | `#282828` | `#3C3836` |

### Catppuccin

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#8839EF` | `#EA76CB` | `#179299` | `#EFF1F5` | `#E6E9EF` |
| Dark | `#CBA6F7` | `#F5C2E7` | `#94E2D5` | `#1E1E2E` | `#313244` |

### Tokyo Night

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#2E7DE9` | `#9854F1` | `#587539` | `#D5D6DB` | `#E9E9EC` |
| Dark | `#7AA2F7` | `#BB9AF7` | `#9ECE6A` | `#1A1B26` | `#24283B` |

### One Dark

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#4078F2` | `#A626A4` | `#50A14F` | `#FAFAFA` | `#F0F0F0` |
| Dark | `#61AFEF` | `#C678DD` | `#98C379` | `#282C34` | `#2C313C` |

### Rose Pine

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#907AA9` | `#D7827E` | `#56949F` | `#FAF4ED` | `#FFFAF3` |
| Dark | `#C4A7E7` | `#EBBCBA` | `#9CCFD8` | `#191724` | `#1F1D2E` |

### Everforest

| Mode | Primary | Secondary | Accent | Background | Surface |
|------|---------|-----------|--------|------------|---------|
| Light | `#8DA101` | `#35A77C` | `#F57D26` | `#FDF6E3` | `#F4F0D9` |
| Dark | `#A7C080` | `#83C092` | `#E69875` | `#2D353B` | `#343F44` |

## Creating Custom Themes

Add a new palette to `parkhub-web/src/stores/palette.ts`:

```typescript
{
  id: "my-theme",
  name: "palette.myTheme",
  light: {
    primary: "#...",
    secondary: "#...",
    accent: "#...",
    bg: "#...",
    surface: "#..."
  },
  dark: {
    primary: "#...",
    secondary: "#...",
    accent: "#...",
    bg: "#...",
    surface: "#..."
  },
}
```

Add translation keys in `parkhub-web/src/i18n/en.ts` and `de.ts`.

## CSS Custom Properties

Themes are applied via CSS custom properties on `:root`:

```css
--color-primary: <r> <g> <b>;
--color-secondary: <r> <g> <b>;
--color-accent: <r> <g> <b>;
--color-palette-bg: <hex>;
--color-palette-surface: <hex>;
```

Use in TailwindCSS:

```html
<div class="bg-primary text-white">...</div>
```

---

Back to [README](../README.md) · Previous: [Development](DEVELOPMENT.md) · Next: [Accessibility](ACCESSIBILITY.md)
