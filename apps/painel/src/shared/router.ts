import { useEffect, useState } from 'react';

export type Route =
  | { page: 'dashboard' }
  | { page: 'specs' }
  | { page: 'spec'; id: string; view: 'documento' | 'plano' }
  | { page: 'tasks' }
  | { page: 'task'; spec: string; task: string }
  | { page: 'documents' }
  | { page: 'settings' }
  | { page: 'missing' };

const SPEC_VIEWS = ['documento', 'plano'] as const;

export function parseRoute(pathname: string): Route {
  const parts = pathname.split('/').filter(Boolean).map(decodeURIComponent);
  const [first, second, third] = parts;
  if (parts.length === 0) return { page: 'dashboard' };
  if (first === 'documentos' && parts.length === 1) return { page: 'documents' };
  if (first === 'configuracoes' && parts.length === 1) return { page: 'settings' };
  if (first === 'specs' && parts.length === 1) return { page: 'specs' };
  if (first === 'specs' && third === 'quadro' && parts.length === 3) return { page: 'tasks' };
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

/** Endereço do quadro único para o endereço antigo do quadro de uma spec; os demais não mudam. */
export function redirectOf(pathname: string): string | null {
  const [first, id, third, ...rest] = pathname.split('/').filter(Boolean).map(decodeURIComponent);
  return first === 'specs' && third === 'quadro' && id && rest.length === 0 ? `/tarefas?spec=${encodeURIComponent(id)}` : null;
}

/** Troca o endereço antigo pelo novo sem criar entrada no histórico, para o Voltar não prender. */
export function applyRedirect() {
  const target = redirectOf(window.location.pathname);
  if (target) window.history.replaceState(window.history.state, '', target);
}

export function readSearchParam(name: string): string | null {
  return new URLSearchParams(window.location.search).get(name);
}

export function writeSearchParam(name: string, value: string | null) {
  const params = new URLSearchParams(window.location.search);
  if (value === null) params.delete(name);
  else params.set(name, value);
  const search = params.toString();
  window.history.replaceState(window.history.state, '', `${window.location.pathname}${search ? `?${search}` : ''}${window.location.hash}`);
}

export function useSearchParam(name: string): [string | null, (value: string | null) => void] {
  const [value, setValue] = useState(() => readSearchParam(name));
  useEffect(() => {
    const update = () => setValue(readSearchParam(name));
    window.addEventListener('popstate', update);
    return () => window.removeEventListener('popstate', update);
  }, [name]);
  const set = (next: string | null) => {
    writeSearchParam(name, next);
    setValue(next);
  };
  return [value, set];
}

const RESTORE_INTERVAL_MS = 50;
const RESTORE_TRIES = 40;
let restoreTimer = 0;

/** Rola até a posição e tenta de novo enquanto a página ainda não tem altura para chegar nela. */
export function restoreScroll(top: number) {
  window.clearTimeout(restoreTimer);
  let tries = 0;
  const attempt = () => {
    window.scrollTo(0, top);
    tries += 1;
    if (Math.abs(window.scrollY - top) > 1 && tries < RESTORE_TRIES) {
      restoreTimer = window.setTimeout(attempt, RESTORE_INTERVAL_MS);
    }
  };
  attempt();
}

export function installScrollRestoration(): () => void {
  if ('scrollRestoration' in window.history) window.history.scrollRestoration = 'manual';
  const onPopState = (event: PopStateEvent) => {
    const top = (event.state as { scroll?: unknown } | null)?.scroll;
    if (typeof top === 'number') restoreScroll(top);
  };
  window.addEventListener('popstate', onPopState);
  return () => window.removeEventListener('popstate', onPopState);
}

export function navigate(href: string) {
  window.clearTimeout(restoreTimer);
  const current = typeof window.history.state === 'object' ? window.history.state : null;
  window.history.replaceState({ ...current, scroll: window.scrollY }, '');
  window.history.pushState(null, '', href);
  window.dispatchEvent(new PopStateEvent('popstate'));
  if (!href.includes('#')) window.scrollTo(0, 0);
}

export function useRoute(): Route {
  const [route, setRoute] = useState(() => {
    applyRedirect();
    return parseRoute(window.location.pathname);
  });
  useEffect(() => {
    const update = () => {
      applyRedirect();
      setRoute(parseRoute(window.location.pathname));
    };
    window.addEventListener('popstate', update);
    const uninstall = installScrollRestoration();
    return () => {
      window.removeEventListener('popstate', update);
      uninstall();
    };
  }, []);
  return route;
}

/** Link interno sem recarregar a página; com Ctrl ou o botão do meio, o navegador abre normalmente. */
export function onLinkClick(event: React.MouseEvent<HTMLAnchorElement>) {
  if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey) return;
  event.preventDefault();
  navigate(event.currentTarget.getAttribute('href') ?? '/');
}
