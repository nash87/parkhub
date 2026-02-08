import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface ColorPalette {
  id: string;
  name: string; // i18n key
  light: {
    primary: string;
    secondary: string;
    accent: string;
    bg: string;
    surface: string;
  };
  dark: {
    primary: string;
    secondary: string;
    accent: string;
    bg: string;
    surface: string;
  };
}

export const PALETTES: ColorPalette[] = [
  {
    id: "default-blue",
    name: "palette.defaultBlue",
    light: { primary: "#3B82F6", secondary: "#6366F1", accent: "#0EA5E9", bg: "#F9FAFB", surface: "#FFFFFF" },
    dark: { primary: "#3B82F6", secondary: "#6366F1", accent: "#0EA5E9", bg: "#030712", surface: "#111827" },
  },
  {
    id: "solarized",
    name: "palette.solarized",
    light: { primary: "#268BD2", secondary: "#2AA198", accent: "#B58900", bg: "#FDF6E3", surface: "#EEE8D5" },
    dark: { primary: "#268BD2", secondary: "#2AA198", accent: "#B58900", bg: "#002B36", surface: "#073642" },
  },
  {
    id: "dracula",
    name: "palette.dracula",
    light: { primary: "#BD93F9", secondary: "#FF79C6", accent: "#50FA7B", bg: "#F8F8F2", surface: "#FFFFFF" },
    dark: { primary: "#BD93F9", secondary: "#FF79C6", accent: "#50FA7B", bg: "#282A36", surface: "#44475A" },
  },
  {
    id: "nord",
    name: "palette.nord",
    light: { primary: "#5E81AC", secondary: "#88C0D0", accent: "#81A1C1", bg: "#ECEFF4", surface: "#E5E9F0" },
    dark: { primary: "#88C0D0", secondary: "#81A1C1", accent: "#5E81AC", bg: "#2E3440", surface: "#3B4252" },
  },
  {
    id: "gruvbox",
    name: "palette.gruvbox",
    light: { primary: "#D79921", secondary: "#689D6A", accent: "#D65D0E", bg: "#FBF1C7", surface: "#EBDBB2" },
    dark: { primary: "#D79921", secondary: "#689D6A", accent: "#D65D0E", bg: "#282828", surface: "#3C3836" },
  },
  {
    id: "catppuccin",
    name: "palette.catppuccin",
    light: { primary: "#8839EF", secondary: "#EA76CB", accent: "#179299", bg: "#EFF1F5", surface: "#E6E9EF" },
    dark: { primary: "#CBA6F7", secondary: "#F5C2E7", accent: "#94E2D5", bg: "#1E1E2E", surface: "#313244" },
  },
  {
    id: "tokyo-night",
    name: "palette.tokyoNight",
    light: { primary: "#2E7DE9", secondary: "#9854F1", accent: "#587539", bg: "#D5D6DB", surface: "#E9E9EC" },
    dark: { primary: "#7AA2F7", secondary: "#BB9AF7", accent: "#9ECE6A", bg: "#1A1B26", surface: "#24283B" },
  },
  {
    id: "one-dark",
    name: "palette.oneDark",
    light: { primary: "#4078F2", secondary: "#A626A4", accent: "#50A14F", bg: "#FAFAFA", surface: "#F0F0F0" },
    dark: { primary: "#61AFEF", secondary: "#C678DD", accent: "#98C379", bg: "#282C34", surface: "#2C313C" },
  },
  {
    id: "rose-pine",
    name: "palette.rosePine",
    light: { primary: "#907AA9", secondary: "#D7827E", accent: "#56949F", bg: "#FAF4ED", surface: "#FFFAF3" },
    dark: { primary: "#C4A7E7", secondary: "#EBBCBA", accent: "#9CCFD8", bg: "#191724", surface: "#1F1D2E" },
  },
  {
    id: "everforest",
    name: "palette.everforest",
    light: { primary: "#8DA101", secondary: "#35A77C", accent: "#F57D26", bg: "#FDF6E3", surface: "#F4F0D9" },
    dark: { primary: "#A7C080", secondary: "#83C092", accent: "#E69875", bg: "#2D353B", surface: "#343F44" },
  },
];

interface PaletteStore {
  paletteId: string;
  setPalette: (id: string) => void;
}

export const usePalette = create<PaletteStore>()(
  persist(
    (set) => ({
      paletteId: "default-blue",
      setPalette: (id) => set({ paletteId: id }),
    }),
    { name: "parkhub-palette" }
  )
);

function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r} ${g} ${b}`;
}

export function applyPalette(paletteId: string, isDark: boolean) {
  const palette = PALETTES.find((p) => p.id === paletteId) || PALETTES[0];
  const colors = isDark ? palette.dark : palette.light;
  const root = document.documentElement;
  root.style.setProperty("--color-primary", hexToRgb(colors.primary));
  root.style.setProperty("--color-secondary", hexToRgb(colors.secondary));
  root.style.setProperty("--color-accent", hexToRgb(colors.accent));
  root.style.setProperty("--color-palette-bg", colors.bg);
  root.style.setProperty("--color-palette-surface", colors.surface);
}
