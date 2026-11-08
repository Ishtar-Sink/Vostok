// Origem: Ishtar-Sink/.github docs/identidade-ecossistema.md (seção Tokens, revisão 2026-10).
// Fonte única; nenhum app redeclara cor, raio ou espaço.

export const light = {
  bg: '#F3F0FA',
  bgElevated: '#FAF8FE',
  surface: '#FFFFFF',
  surfaceMuted: '#EAE5F6',
  surfaceHover: '#E1DAF2',
  border: '#D9D1EC',
  borderStrong: '#BFB3DD',

  elevation1: '#FFFFFF',
  elevation2: '#FFFFFF',
  elevation3: '#FFFFFF',

  primary: '#6D28D9',
  primaryHover: '#5B21B6',
  primaryBright: '#A855F7',
  primarySoft: '#EBE0FD',
  primaryOn: '#FFFFFF',

  accent: '#0E7490',
  accentFill: '#06B6D4',
  accentBright: '#22D3EE',
  accentSoft: '#D8F1F6',

  text: '#1A1230',
  textMuted: '#4F4469',
  textFaint: '#8B84A3',

  success: '#047857',
  successSoft: '#D9F3E7',
  warning: '#92400E',
  warningSoft: '#FCEFCF',
  danger: '#BE123C',
  dangerSoft: '#FDE2E9',
  info: '#0369A1',
  infoSoft: '#DAEEFB',

  shadowSm: '0 1px 2px rgba(42, 24, 96, 0.08)',
  shadowMd: '0 4px 12px rgba(42, 24, 96, 0.10), 0 1px 3px rgba(42, 24, 96, 0.06)',
  shadowLg: '0 18px 44px rgba(42, 24, 96, 0.20), 0 4px 12px rgba(42, 24, 96, 0.08)',
} as const;

export const dark = {
  bg: '#0B0816',
  bgElevated: '#110B21',
  surface: '#181028',
  surfaceMuted: '#130D22',
  surfaceHover: '#241A3D',
  border: '#2F2250',
  borderStrong: '#45337A',

  elevation1: '#181028',
  elevation2: '#201638',
  elevation3: '#2A1D4A',

  primary: '#A78BFA',
  primaryHover: '#C4B5FD',
  primaryBright: '#C084FC',
  primarySoft: '#2B1D52',
  primaryOn: '#12082B',

  accent: '#22D3EE',
  accentFill: '#06B6D4',
  accentBright: '#67E8F9',
  accentSoft: '#0F3342',

  text: '#F5F1FF',
  textMuted: '#B1A5CF',
  textFaint: '#7A6C9E',

  success: '#34D399',
  successSoft: '#0E2E27',
  warning: '#FBBF24',
  warningSoft: '#33260B',
  danger: '#FB7185',
  dangerSoft: '#3A1222',
  info: '#38BDF8',
  infoSoft: '#0C2C42',

  shadowSm: '0 1px 2px rgba(0, 0, 0, 0.45)',
  shadowMd: '0 6px 16px rgba(0, 0, 0, 0.50), 0 0 0 1px rgba(167, 139, 250, 0.08)',
  shadowLg: '0 22px 52px rgba(0, 0, 0, 0.65), 0 0 0 1px rgba(167, 139, 250, 0.14)',
} as const;

export const radii = { sm: 8, md: 12, lg: 18, xl: 26, pill: 999 } as const;

/** Grade de 4px. */
export const space = (n: number): number => n * 4;

export const fontSizes = {
  xs: 12,
  sm: 13,
  md: 14,
  base: 16,
  lg: 18,
  xl: 21,
  xxl: 30,
  xxxl: 36,
} as const;

/** Escala de espaçamento do protótipo (`--s1` a `--s10`), em múltiplos de 4 px. */
export const spacing = { unit: 4, steps: [1, 2, 3, 4, 5, 6, 8, 10] } as const;

export const motion = {
  fast: 120,
  base: 200,
  slow: 320,
  ease: {
    standard: 'cubic-bezier(0.4, 0, 0.2, 1)',
    enter: 'cubic-bezier(0, 0, 0.2, 1)',
    exit: 'cubic-bezier(0.4, 0, 1, 1)',
    linear: 'linear',
  },
} as const;

export const gradients = {
  brand: ['#7C3AED', '#06B6D4'],
  deep: ['#6D28D9', '#7C3AED'],
  calm: ['#F3EBFF', '#E0F7FB'],
} as const;

export const fonts = {
  display: "'Outfit', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif",
  body: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif",
} as const;

export const breakpoints = { md: 720, sm: 480, min: 360 } as const;
