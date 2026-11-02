import type { Brand } from './api';

/** Tokens do Atria que o `documents.yml` traz com o mesmo nome (com `_` no lugar de `-`); o
 *  `vostok documents brand` os deriva da cor da marca, então basta aplicá-los. */
const ATRIA_TOKENS = [
  'bg', 'bg_elevated', 'surface', 'surface_muted', 'surface_hover', 'border', 'border_strong',
  'primary', 'primary_hover', 'primary_soft', 'primary_on', 'accent', 'accent_soft',
  'text', 'text_muted', 'text_faint', 'success', 'success_soft', 'warning', 'warning_soft',
  'danger', 'danger_soft', 'info', 'info_soft', 'brand_from', 'brand_to',
];

function declarations(theme: Record<string, string>): string {
  return ATRIA_TOKENS.filter((key) => theme[key]).map((key) => `--${key.replaceAll('_', '-')}: ${theme[key]};`).join('\n  ');
}

/** Aplica a marca da pasta: as cores dos dois temas e o título da aba. */
export function applyBrand(brand: Brand, project: string) {
  const css = `:root, [data-theme='light'] {\n  ${declarations(brand.light)}\n}\n[data-theme='dark'] {\n  ${declarations(brand.dark)}\n}`;
  let style = document.getElementById('brand-tokens');
  if (!style) {
    style = document.createElement('style');
    style.id = 'brand-tokens';
    document.head.appendChild(style);
  }
  style.textContent = css;
  document.title = `${brand.brand.name || project} · Vostok`;
}
