import { useEffect, useRef, type KeyboardEvent, type ReactNode } from 'react';
import type { OpenSpec } from '../shared/api';
import { onLinkClick, type Route } from '../shared/router';
import type { Theme } from '../main';
import styles from './layout.module.css';

const FOCUSABLE = 'a[href], button:not([disabled])';

export interface SidebarProps {
  name: string;
  icon: string;
  specs: OpenSpec[];
  pending: number;
  route: Route;
  theme: Theme;
  onToggleTheme: () => void;
  drawer: boolean;
  open: boolean;
  onClose: () => void;
}

function Icon({ children }: { children: ReactNode }) {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      {children}
    </svg>
  );
}

const ICONS = {
  dashboard: <Icon><rect x="3" y="3" width="7" height="9" rx="1.5" /><rect x="14" y="3" width="7" height="5" rx="1.5" /><rect x="14" y="12" width="7" height="9" rx="1.5" /><rect x="3" y="16" width="7" height="5" rx="1.5" /></Icon>,
  inbox: <Icon><path d="M22 12h-6l-2 3h-4l-2-3H2" /><path d="M5.5 5.1 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.5-6.9A2 2 0 0 0 16.8 4H7.2a2 2 0 0 0-1.7 1.1z" /></Icon>,
  specs: <Icon><path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" /><path d="M14 3v5h5M9 13h6M9 17h4" /></Icon>,
  tasks: <Icon><rect x="3" y="4" width="5" height="16" rx="1.5" /><rect x="10" y="4" width="5" height="10" rx="1.5" /><rect x="17" y="4" width="4" height="13" rx="1.5" /></Icon>,
  documents: <Icon><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" /></Icon>,
  theme: <Icon><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z" /></Icon>,
  settings: <Icon><circle cx="12" cy="12" r="3" /><path d="M19 12a7 7 0 0 0-.1-1.3l2-1.5-2-3.4-2.3.9a7 7 0 0 0-2.2-1.3L14 3h-4l-.4 2.4a7 7 0 0 0-2.2 1.3l-2.3-.9-2 3.4 2 1.5A7 7 0 0 0 5 12c0 .4 0 .9.1 1.3l-2 1.5 2 3.4 2.3-.9a7 7 0 0 0 2.2 1.3L10 21h4l.4-2.4a7 7 0 0 0 2.2-1.3l2.3.9 2-3.4-2-1.5c.1-.4.1-.9.1-1.3z" /></Icon>,
};

function section(route: Route): string {
  if (route.page === 'spec') return 'specs';
  if (route.page === 'task') return 'tasks';
  return route.page;
}

export function Sidebar({ name, icon, specs, pending, route, theme, onToggleTheme, drawer, open, onClose }: SidebarProps) {
  const ref = useRef<HTMLElement>(null);
  const current = section(route);
  const currentSpec = route.page === 'spec' ? route.id : route.page === 'task' ? route.spec : null;
  const active = specs.filter((spec) => spec.tasks.some((task) => task.status !== 'done') || spec.plan || spec.meta.status === 'refined');

  useEffect(() => {
    if (drawer && open) ref.current?.querySelector<HTMLElement>(FOCUSABLE)?.focus();
  }, [drawer, open]);

  function onKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (drawer && open && event.key === 'Escape') onClose();
  }

  const items = [
    { id: 'dashboard', href: '/', label: 'Painel', icon: ICONS.dashboard },
    { id: 'inbox', href: '/pendencias', label: 'Pendências', icon: ICONS.inbox, count: pending },
    { id: 'specs', href: '/specs', label: 'Specs', icon: ICONS.specs },
    { id: 'tasks', href: '/tarefas', label: 'Tarefas', icon: ICONS.tasks },
    { id: 'documents', href: '/documentos', label: 'Documentos', icon: ICONS.documents },
  ];

  return (
    // O onKeyDown só fecha a gaveta com Esc; o aside não vira controle.
    <aside
      id="sidebar"
      ref={ref}
      className={[styles.sidebar, drawer ? styles.drawer : '', open ? styles.drawerOpen : ''].filter(Boolean).join(' ')}
      inert={drawer && !open}
      onKeyDown={onKeyDown}
    >
      <a href="/" onClick={onLinkClick} className={styles.brand}>
        {icon ? <img src={icon} alt="" className={styles.mark} /> : <i className={styles.mark} aria-hidden="true" />}
        <span className={styles.brandText}>
          <span className={styles.brandName}>{name}</span>
          <span className={styles.brandProduct}>Vostok</span>
        </span>
      </a>
      <nav aria-label="Principal" className={styles.nav}>
        <ul className={styles.navList}>
          {items.map((item) => (
            <li key={item.id}>
              <a href={item.href} onClick={onLinkClick} className={styles.navLink} aria-current={current === item.id ? 'page' : undefined}>
                {item.icon}
                <span className={styles.navLabel}>{item.label}</span>
                {item.count ? <span className={styles.navCount}>{item.count}</span> : null}
              </a>
            </li>
          ))}
        </ul>
        {active.length > 0 ? (
          <>
            <p className={styles.navGroup}>Em andamento</p>
            <ul className={styles.navList}>
              {active.map((spec) => {
                const done = spec.tasks.filter((task) => task.status === 'done').length;
                return (
                  <li key={spec.meta.id}>
                    <a
                      href={`/specs/${spec.meta.id}${spec.tasks.length > 0 ? '/quadro' : ''}`}
                      onClick={onLinkClick}
                      className={styles.specLink}
                      aria-current={spec.meta.id === currentSpec ? 'true' : undefined}
                      title={spec.meta.title}
                    >
                      <span className={styles.navId}>{spec.meta.id}</span>
                      <span className={styles.navTitle}>{spec.meta.title}</span>
                      {spec.tasks.length > 0 ? (
                        <span className={styles.miniTrack} aria-label={`${done} de ${spec.tasks.length} tarefas concluídas`}>
                          <span style={{ width: `${(done / spec.tasks.length) * 100}%` }} />
                        </span>
                      ) : null}
                    </a>
                  </li>
                );
              })}
            </ul>
          </>
        ) : null}
      </nav>
      <div className={styles.sidebarFooter}>
        <a href="/configuracoes" onClick={onLinkClick} className={styles.navLink} aria-current={current === 'settings' ? 'page' : undefined}>
          {ICONS.settings}
          <span>Configurações</span>
        </a>
        <button type="button" className={styles.navLink} onClick={onToggleTheme}>
          {ICONS.theme}
          <span>{theme === 'dark' ? 'Tema claro' : 'Tema escuro'}</span>
        </button>
      </div>
    </aside>
  );
}
