import { navigate } from './router';
import styles from './back-link.module.css';

/** Voltar para a tela anterior do painel; aberta direto pelo link, volta para o destino padrão. */
export function BackLink({ fallback, label }: { fallback: string; label: string }) {
  return (
    <a
      href={fallback}
      className={styles.back}
      onClick={(event) => {
        if (event.button !== 0 || event.metaKey || event.ctrlKey) return;
        event.preventDefault();
        if (window.history.length > 1) window.history.back();
        else navigate(fallback);
      }}
    >
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
        <path d="m15 18-6-6 6-6" />
      </svg>
      Voltar <span className={styles.to}>· {label}</span>
    </a>
  );
}
