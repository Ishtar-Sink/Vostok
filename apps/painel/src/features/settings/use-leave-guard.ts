import { useEffect, useState } from 'react';
import { navigate } from '../../shared/router';

export interface LeaveGuard {
  /** Endereço que o clique tentou abrir; `null` sem saída em espera. */
  target: string | null;
  stay: () => void;
  leave: () => void;
}

function internalLink(event: MouseEvent): URL | null {
  const anchor = (event.target as Element | null)?.closest?.('a[href]') as HTMLAnchorElement | null;
  if (!anchor || event.defaultPrevented || event.button !== 0) return null;
  if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return null;
  if (anchor.target === '_blank' || anchor.hasAttribute('download')) return null;
  const url = new URL(anchor.href, window.location.href);
  if (url.origin !== window.location.origin || url.pathname === window.location.pathname) return null;
  return url;
}

/** Segura a saída por link interno e pede a confirmação do navegador ao fechar, enquanto `active`. */
export function useLeaveGuard(active: boolean): LeaveGuard {
  const [target, setTarget] = useState<string | null>(null);

  useEffect(() => {
    if (!active) return;
    const onClick = (event: MouseEvent) => {
      const url = internalLink(event);
      if (!url) return;
      event.preventDefault();
      event.stopPropagation();
      setTarget(url.pathname + url.search + url.hash);
    };
    const onUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = '';
    };
    document.addEventListener('click', onClick, true);
    window.addEventListener('beforeunload', onUnload);
    return () => {
      document.removeEventListener('click', onClick, true);
      window.removeEventListener('beforeunload', onUnload);
    };
  }, [active]);

  const stay = () => setTarget(null);
  const leave = () => {
    const destination = target;
    setTarget(null);
    if (destination) navigate(destination);
  };
  return { target, stay, leave };
}
