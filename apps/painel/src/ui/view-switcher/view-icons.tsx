import type { ComponentPropsWithRef, ReactNode } from 'react';

export interface ViewIconProps extends Omit<ComponentPropsWithRef<'svg'>, 'children'> {
  /** Lado do ícone em pixels. Padrão: 18. */
  size?: number;
}

/** Base comum: SVG decorativo que herda a cor do texto; `aria-hidden` não pode ser sobrescrito. */
function ViewIcon({ size = 18, children, ...rest }: ViewIconProps & { children: ReactNode }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      {...rest}
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}

/** Visualização em lista. */
export function ListIcon(props: ViewIconProps) {
  return (
    <ViewIcon {...props}>
      <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
    </ViewIcon>
  );
}

/** Visualização em blocos. */
export function BlocksIcon(props: ViewIconProps) {
  return (
    <ViewIcon {...props}>
      <rect x="3" y="3" width="18" height="7" rx="1.5" />
      <rect x="3" y="14" width="18" height="7" rx="1.5" />
    </ViewIcon>
  );
}
