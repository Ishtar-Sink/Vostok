import type { dark, light } from './tokens';

export {
  light,
  dark,
  radii,
  space,
  spacing,
  fontSizes,
  motion,
  gradients,
  fonts,
  breakpoints,
} from './tokens';
export { artPalettes, paletteFor } from './palette';
export { contrastRatio } from './contrast';

export type ThemeName = 'light' | 'dark';
export type ThemeTokens = typeof light | typeof dark;
