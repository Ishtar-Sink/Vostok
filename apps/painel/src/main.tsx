import { StrictMode, useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import './theme/tokens.css';
import './app.css';
import { useLiveVersion, useResource, type Brand, type Overview } from './shared/api';
import { applyBrand } from './shared/brand';
import { onLinkClick, useRoute } from './shared/router';
import { Sidebar } from './layout/sidebar';
import { Topbar } from './layout/topbar';
import { DashboardPage } from './features/dashboard/dashboard-page';
import { SpecsPage } from './features/specs/specs-page';
import { SpecPage } from './features/spec/spec-page';
import { TasksPage } from './features/board/tasks-page';
import { TaskPage } from './features/task/task-page';
import { DocumentsPage } from './features/documents/documents-page';
import { SettingsPage } from './features/settings/settings-page';
import { EmptyState } from './ui/empty-state/empty-state';
import { BackToTop } from './shared/back-to-top';
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
    <div className={styles.shell}>
      <a href="#conteudo" className={styles.skipLink}>Pular para o conteúdo</a>
      <Sidebar
        name={brandData?.brand.name || project || 'Vostok'}
        icon={brandData?.icon_data ?? ''}
        specs={data?.specs ?? []}
        route={route}
        theme={theme}
        onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        drawer={drawer}
        open={drawer && open}
        onClose={() => setOpen(false)}
      />
      {drawer && open ? <button type="button" tabIndex={-1} aria-label="Fechar o menu" className={styles.scrim} onClick={() => setOpen(false)} /> : null}
      <div className={styles.column}>
        <Topbar route={route} connected={connected} drawer={drawer} open={drawer && open} onToggle={() => setOpen(!open)} />
        <main id="conteudo" tabIndex={-1} className={styles.page}>
          <div className={styles.pageInner}>
            {route.page === 'dashboard' && <DashboardPage version={version} name={brandData?.brand.name ?? ''} />}
            {route.page === 'specs' && <SpecsPage version={version} />}
            {route.page === 'spec' && <SpecPage id={route.id} view={route.view} version={version} />}
            {route.page === 'tasks' && <TasksPage version={version} />}
            {route.page === 'task' && <TaskPage spec={route.spec} task={route.task} version={version} />}
            {route.page === 'documents' && <DocumentsPage version={version} />}
            {route.page === 'settings' && <SettingsPage />}
            {route.page === 'missing' && <EmptyState title="Página não encontrada" description="Volte pelo menu da barra lateral." />}
          </div>
        </main>
        <BackToTop />
      </div>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
