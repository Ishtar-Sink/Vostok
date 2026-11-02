import { onLinkClick, type Route } from '../shared/router';
import styles from './layout.module.css';

interface Crumb {
  label: string;
  href?: string;
}

const VIEW_LABELS = { documento: 'Documento', quadro: 'Quadro', plano: 'Plano' };

function crumbs(route: Route): Crumb[] {
  switch (route.page) {
    case 'dashboard':
      return [{ label: 'Painel' }];
    case 'inbox':
      return [{ label: 'Pendências' }];
    case 'specs':
      return [{ label: 'Specs' }];
    case 'spec':
      return [{ label: 'Specs', href: '/specs' }, { label: route.id, href: route.view === 'documento' ? undefined : `/specs/${route.id}` }, ...(route.view === 'documento' ? [] : [{ label: VIEW_LABELS[route.view] }])];
    case 'tasks':
      return [{ label: 'Tarefas' }];
    case 'task':
      return [{ label: 'Tarefas', href: '/tarefas' }, { label: route.spec, href: `/specs/${route.spec}/quadro` }, { label: route.task }];
    case 'documents':
      return [{ label: 'Documentos' }];
    case 'settings':
      return [{ label: 'Configurações' }];
    default:
      return [{ label: 'Painel', href: '/' }, { label: 'Não encontrada' }];
  }
}

function MenuIcon() {
  return (
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
      <path d="M4 6h16M4 12h16M4 18h16" />
    </svg>
  );
}

export function Topbar({ route, connected, drawer, open, onToggle }: { route: Route; connected: boolean | null; drawer: boolean; open: boolean; onToggle: () => void }) {
  const items = crumbs(route);
  return (
    <header className={styles.topbar}>
      {drawer ? (
        <button type="button" className={styles.menuButton} aria-label={open ? 'Fechar o menu' : 'Abrir o menu'} aria-expanded={open} aria-controls="sidebar" onClick={onToggle}>
          <MenuIcon />
        </button>
      ) : null}
      <nav aria-label="Trilha" className={styles.breadcrumbs}>
        <ol className={styles.crumbList}>
          {items.map((item, index) => (
            <li key={`${item.label}-${index}`} className={styles.crumb}>
              {item.href ? <a href={item.href} onClick={onLinkClick}>{item.label}</a> : <span aria-current="page">{item.label}</span>}
            </li>
          ))}
        </ol>
      </nav>
      <span className={styles.live} data-on={connected === true} role="status">
        <i aria-hidden="true" />
        {connected ? 'Ao vivo' : connected === null ? 'Conectando' : 'Reconectando'}
      </span>
    </header>
  );
}
