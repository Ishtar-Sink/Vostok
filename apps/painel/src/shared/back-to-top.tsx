import { useEffect, useState } from 'react';
import styles from './back-to-top.module.css';

function goToTop() {
  const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  window.scrollTo({ top: 0, behavior: reducedMotion ? 'auto' : 'smooth' });
  document.getElementById('conteudo')?.focus({ preventScroll: true });
}

export function BackToTop() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const update = () => setVisible(window.scrollY > window.innerHeight);
    window.addEventListener('scroll', update, { passive: true });
    return () => window.removeEventListener('scroll', update);
  }, []);

  return (
    <button type="button" className={styles.button} hidden={!visible} onClick={goToTop}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
        <path d="M8 13V3M3.5 7.5L8 3l4.5 4.5" />
      </svg>
      Voltar ao topo
    </button>
  );
}
