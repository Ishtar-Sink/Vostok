import { useEffect, useState } from 'react';

export type Route =
  | { page: 'dashboard' }
  | { page: 'specs' }
  | { page: 'spec'; id: string; view: 'documento' | 'quadro' | 'plano' }
  | { page: 'tasks' }
  | { page: 'task'; spec: string; task: string }
  | { page: 'documents' }
  | { page: 'missing' };

const SPEC_VIEWS = ['documento', 'quadro', 'plano'] as const;

export function parseRoute(pathname: string): Route {
  const parts = pathname.split('/').filter(Boolean).map(decodeURIComponent);
  const [first, second, third] = parts;
  if (parts.length === 0) return { page: 'dashboard' };
  if (first === 'documentos' && parts.length === 1) return { page: 'documents' };
  if (first === 'specs' && parts.length === 1) return { page: 'specs' };
  if (first === 'specs' && parts.length <= 3) {
    const view = (SPEC_VIEWS as readonly string[]).includes(third ?? '') ? (third as (typeof SPEC_VIEWS)[number]) : 'documento';
    return { page: 'spec', id: second, view };
  }
  // Endereço antigo do plano, de antes de ele virar uma aba da spec.
  if (first === 'planos' && parts.length === 2) return { page: 'spec', id: second, view: 'plano' };
  if (first === 'tarefas' && parts.length === 1) return { page: 'tasks' };
  if (first === 'tarefas' && parts.length === 3) return { page: 'task', spec: second, task: third };
  return { page: 'missing' };
}

export function navigate(href: string) {
  window.history.pushState(null, '', href);
  window.dispatchEvent(new PopStateEvent('popstate'));
  if (!href.includes('#')) window.scrollTo(0, 0);
}

export function useRoute(): Route {
  const [route, setRoute] = useState(() => parseRoute(window.location.pathname));
  useEffect(() => {
    const update = () => setRoute(parseRoute(window.location.pathname));
    window.addEventListener('popstate', update);
    return () => window.removeEventListener('popstate', update);
  }, []);
  return route;
}

/** Link interno sem recarregar a página; com Ctrl ou o botão do meio, o navegador abre normalmente. */
export function onLinkClick(event: React.MouseEvent<HTMLAnchorElement>) {
  if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return;
  event.preventDefault();
  navigate(event.currentTarget.getAttribute('href') ?? '/');
}
