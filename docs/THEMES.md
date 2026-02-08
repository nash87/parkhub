# Color Themes

ParkHub ships with 10 color palettes. Each has a light and dark variant. Users pick their palette in Profile → Appearance. The choice is stored in localStorage (`parkhub-palette`).

## How Themes Work

Palettes are defined in `parkhub-web/src/stores/palette.ts` as a `PALETTES` array. Each palette has 5 colors for light and 5 for dark mode:

- **primary** — buttons, links, active elements
- **secondary** — secondary actions, accents
- **accent** — highlights, badges, notifications
- **bg** — page background
- **surface** — card/panel backgrounds

When a palette is selected, `applyPalette()` converts hex colors to RGB triplets and sets them as CSS custom properties on `:root`:

```css
--color-primary: 59 130 246;     /* RGB values, not hex */
--color-secondary: 99 102 241;
--color-accent: 14 165 233;
--color-palette-bg: #F9FAFB;     /* these two stay as hex */
--color-palette-surface: #FFFFFF;
```

TailwindCSS classes reference these properties, so the entire UI updates instantly.

## All Palettes

### Default Blue

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#3B82F6` | `#6366F1` | `#0EA5E9` | `#F9FAFB` | `#FFFFFF` |
| Dark | `#3B82F6` | `#6366F1` | `#0EA5E9` | `#030712` | `#111827` |

### Solarized

Based on [Ethan Schoonover's Solarized](https://ethanschoonover.com/solarized/).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#268BD2` | `#2AA198` | `#B58900` | `#FDF6E3` | `#EEE8D5` |
| Dark | `#268BD2` | `#2AA198` | `#B58900` | `#002B36` | `#073642` |

### Dracula

Based on [Dracula Theme](https://draculatheme.com/).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#BD93F9` | `#FF79C6` | `#50FA7B` | `#F8F8F2` | `#FFFFFF` |
| Dark | `#BD93F9` | `#FF79C6` | `#50FA7B` | `#282A36` | `#44475A` |

### Nord

Based on [Arctic Ice Studio's Nord](https://www.nordtheme.com/).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#5E81AC` | `#88C0D0` | `#81A1C1` | `#ECEFF4` | `#E5E9F0` |
| Dark | `#88C0D0` | `#81A1C1` | `#5E81AC` | `#2E3440` | `#3B4252` |

### Gruvbox

Based on [morhetz's Gruvbox](https://github.com/morhetz/gruvbox).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#D79921` | `#689D6A` | `#D65D0E` | `#FBF1C7` | `#EBDBB2` |
| Dark | `#D79921` | `#689D6A` | `#D65D0E` | `#282828` | `#3C3836` |

### Catppuccin

Based on [Catppuccin](https://catppuccin.com/) (Latte for light, Mocha for dark).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#8839EF` | `#EA76CB` | `#179299` | `#EFF1F5` | `#E6E9EF` |
| Dark | `#CBA6F7` | `#F5C2E7` | `#94E2D5` | `#1E1E2E` | `#313244` |

### Tokyo Night

Based on [folke's Tokyo Night](https://github.com/folke/tokyonight.nvim).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#2E7DE9` | `#9854F1` | `#587539` | `#D5D6DB` | `#E9E9EC` |
| Dark | `#7AA2F7` | `#BB9AF7` | `#9ECE6A` | `#1A1B26` | `#24283B` |

### One Dark

Based on Atom's One Dark.

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#4078F2` | `#A626A4` | `#50A14F` | `#FAFAFA` | `#F0F0F0` |
| Dark | `#61AFEF` | `#C678DD` | `#98C379` | `#282C34` | `#2C313C` |

### Rose Pine

Based on [Rose Pine](https://rosepinetheme.com/).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#907AA9` | `#D7827E` | `#56949F` | `#FAF4ED` | `#FFFAF3` |
| Dark | `#C4A7E7` | `#EBBCBA` | `#9CCFD8` | `#191724` | `#1F1D2E` |

### Everforest

Based on [sainnhe's Everforest](https://github.com/sainnhe/everforest).

|  | Primary | Secondary | Accent | Background | Surface |
|--|---------|-----------|--------|------------|---------|
| Light | `#8DA101` | `#35A77C` | `#F57D26` | `#FDF6E3` | `#F4F0D9` |
| Dark | `#A7C080` | `#83C092` | `#E69875` | `#2D353B` | `#343F44` |

## Adding a Custom Theme

Add a new entry to the `PALETTES` array in `parkhub-web/src/stores/palette.ts`:

```typescript
{
  id: "my-theme",
  name: "palette.myTheme",
  light: { primary: "#...", secondary: "#...", accent: "#...", bg: "#...", surface: "#..." },
  dark:  { primary: "#...", secondary: "#...", accent: "#...", bg: "#...", surface: "#..." },
}
```

Then add the translation key:

```typescript
// en.ts
"palette.myTheme": "My Theme",

// de.ts
"palette.myTheme": "Mein Theme",
```

Rebuild the frontend and the new theme appears in the selector.

---

Back to [README](../README.md) · Previous: [Development](DEVELOPMENT.md) · Next: [Accessibility](ACCESSIBILITY.md)
