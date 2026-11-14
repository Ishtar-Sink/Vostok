import { createRoot } from 'react-dom/client';
import '../../theme/tokens.css';
import '../../app.css';
import { SettingsPage } from './settings-page';

const params = new URLSearchParams(window.location.search);
document.documentElement.dataset.theme = params.get('tema') === 'escuro' ? 'dark' : 'light';

createRoot(document.getElementById('root')!).render(
  <main style={{ padding: 'var(--s6) var(--s4)' }}>
    <div style={{ maxWidth: '1100px', margin: '0 auto' }}>
      <SettingsPage />
    </div>
  </main>,
);
