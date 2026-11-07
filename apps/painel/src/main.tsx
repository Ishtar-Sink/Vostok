import { StrictMode, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import './theme/tokens.css';
import './app.css';
import { LiveVersion } from './shared/action-bar';
import { useLiveVersion, useResource, type Brand, type Overview } from './shared/api';
import { applyBrand } from './shared/brand';
import { onLinkClick, useRoute } from './shared/router';
import { useReturnNotices } from './shared/notifications';
import { Sidebar } from './layout/sidebar';
import { Topbar } from './layout/topbar';
import { attentionItems } from './shared/attention';
import { DashboardPage } from './features/dashboard/dashboard-page';
import { AskPage } from './features/ask/ask-page';
import { InboxPage } from './features/inbox/inbox-page';
import { SpecsPage } from './features/specs/specs-page';
import { SpecPage } from './features/spec/spec-page';
import { TasksPage } from './features/board/tasks-page';
import { TaskPage } from './features/task/task-page';
import { DocumentsPage } from './features/documents/documents-page';
import { ActivityPage } from './features/activity/activity-page';
import { SettingsPage } from './features/settings/settings-page';
import { Toast, ToastRegion } from './ui/toast/toast';
import { Button } from './ui/button/button';
import { EmptyState } from './ui/empty-state/empty-state';
import styles from './layout/layout.module.css';

export type Theme = 'light' | 'dark';

const DRAWER_QUERY = '(max-width: 860px)';

function initialTheme(): Theme {
  try {
    const saved = localStorage.getItem('vostok-tema');
    if (saved === 'light' || saved === 'dark') return saved;
  } catch {
    // Sem armazenamento: segue o sistema.
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function useDrawerMode(): boolean {
  const [drawer, setDrawer] = useState(() => window.matchMedia(DRAWER_QUERY).matches);
  useEffect(() => {
    const query = window.matchMedia(DRAWER_QUERY);
    const onChange = () => setDrawer(query.matches);
    query.addEventListener('change', onChange);
    return () => query.removeEventListener('change', onChange);
  }, []);
  return drawer;
}

function ReturnNoticeRegion({ notices, dismiss, askPermission, enableBrowser, declineBrowser, regionProps, followNotice }: ReturnType<typeof useReturnNotices>) {
  const closeNotice = (id: string) => {
    dismiss(id);
    requestAnimationFrame(() => {
      const next = document.querySelector<HTMLElement>('[aria-label^="Avisos"] button[aria-label^="Fechar o aviso"]');
      (next ?? document.getElementById('conteudo'))?.focus();
    });
  };
  return (
    <ToastRegion aria-label="Avisos do agente" {...regionProps}>
      {askPermission ? (
        <Toast tone="info" className={styles.notice}>
          <p className={styles.noticeText}>
            <strong>Avisos do navegador</strong>
            Quer um aviso do navegador quando o agente terminar? Sem ele, os avisos continuam aqui na tela.
          </p>
          <div className={styles.noticeActions}>
            <Button variant="primary" onClick={() => void enableBrowser()}>Ativar avisos do navegador</Button>
            <Button variant="ghost" onClick={declineBrowser}>Agora não</Button>
          </div>
        </Toast>
      ) : null}
      {notices.map((notice) => (
        <Toast key={notice.id} tone={notice.tone} className={styles.notice}>
          <p className={styles.noticeText}>
            <strong>{notice.title}</strong>
            <span className={styles.noticeMessage} data-notice-message title={notice.message}>{notice.message}</span>
          </p>
          <div className={styles.noticeActions}>
            <a href={notice.href} onClick={(event) => followNotice(event, notice)}>{notice.linkLabel}</a>
          </div>
          <button type="button" className={styles.noticeClose} aria-label={`Fechar o aviso de ${notice.id}`} onClick={() => closeNotice(notice.id)}>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" aria-hidden="true">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        </Toast>
      ))}
    </ToastRegion>
  );
}

function App() {
  const route = useRoute();
  const { version, connected } = useLiveVersion();
  const brand = useResource<Brand>('/api/marca', version);
  const overview = useResource<Overview>('/api/visao', version);
  const [theme, setTheme] = useState<Theme>(initialTheme);
  const drawer = useDrawerMode();
  const [open, setOpen] = useState(false);

  const data = overview.kind === 'ready' ? overview.data : null;
  const project = data?.project ?? '';
  const unread = data?.respostas_nao_lidas?.length ?? 0;
  const notices = useReturnNotices(data);
  const brandData = brand.kind === 'ready' ? brand.data : null;

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem('vostok-tema', theme);
    } catch {
      // Preferência só desta sessão.
    }
  }, [theme]);

  useEffect(() => {
    if (brandData) applyBrand(brandData, project);
  }, [brandData, project]);

  useEffect(() => setOpen(false), [route]);

  return (
    <LiveVersion.Provider value={version}>
    <div className={styles.shell}>
      <a href="#conteudo" className={styles.skipLink}>Pular para o conteúdo</a>
      <Sidebar
        name={brandData?.brand.name || project || 'Vostok'}
        icon={brandData?.icon_data ?? ''}
        specs={data?.specs ?? []}
        pending={data ? attentionItems(data).length + unread : 0}
        unreadAnswers={unread}
        loop={data?.loop ?? null}
        requests={data?.requests ?? []}
        route={route}
        theme={theme}
        onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        drawer={drawer}
        open={drawer && open}
        onClose={() => setOpen(false)}
      />
      {drawer && open ? <button type="button" tabIndex={-1} aria-label="Fechar o menu" className={styles.scrim} onClick={() => setOpen(false)} /> : null}
      <div className={styles.column}>
        <Topbar route={route} connected={connected} unread={data?.notificacoes_nao_lidas ?? 0} version={version} drawer={drawer} open={drawer && open} onToggle={() => setOpen(!open)} />
        <main id="conteudo" tabIndex={-1} className={styles.page}>
          <div className={styles.pageInner}>
            {route.page === 'dashboard' && <DashboardPage version={version} name={brandData?.brand.name ?? ''} />}
            {route.page === 'inbox' && <InboxPage version={version} />}
            {route.page === 'ask' && <AskPage version={version} />}
            {route.page === 'specs' && <SpecsPage version={version} />}
            {route.page === 'spec' && <SpecPage id={route.id} view={route.view} version={version} />}
            {route.page === 'tasks' && <TasksPage version={version} />}
            {route.page === 'task' && <TaskPage spec={route.spec} task={route.task} version={version} />}
            {route.page === 'documents' && <DocumentsPage version={version} />}
            {route.page === 'activity' && <ActivityPage version={version} />}
            {route.page === 'settings' && <SettingsPage version={version} />}
            {route.page === 'missing' && <EmptyState title="Página não encontrada" description="Volte pelo menu da barra lateral." />}
          </div>
        </main>
      </div>
      <ReturnNoticeRegion {...notices} />
    </div>
    </LiveVersion.Provider>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
