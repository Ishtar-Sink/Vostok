import { createRoot } from 'react-dom/client';
import '../../theme/tokens.css';
import '../../app.css';
import { Badge } from '../badge/badge';
import { Button } from '../button/button';
import { CheckList } from '../check-list/check-list';
import { ConfirmPopover } from '../confirm-popover/confirm-popover';
import { SplitDialog } from './split-dialog';

const params = new URLSearchParams(window.location.search);
document.documentElement.dataset.theme = params.get('tema') === 'escuro' ? 'dark' : 'light';
const pane = params.get('painel') === 'evidencia' ? 'main' : 'side';

const LOG = [
  'running 1 test',
  'test reviews_ct22_returns_the_four_states ... ok',
  '',
  'warning: unused variable: `total`',
  'test result: ok. 1 passed; 0 failed; 0 ignored',
  'Duration  1.2s',
].join('\n');

function Demo() {
  return (
    <SplitDialog
      open
      title="Testes do quadro passam"
      eyebrow="EV-01"
      status={<Badge tone="warning">A conferir</Badge>}
      pane={pane}
      onClose={() => undefined}
      side={
        <>
          <section>
            <h3>Trecho esperado</h3>
            <CheckList
              label="Trecho esperado"
              itemTitle="Clique para destacar no log e copiar"
              active="0"
              onSelect={() => undefined}
              items={[
                { id: '0', label: 'test reviews_ct22_returns_the_four_states ... ok', result: 'apareceu · 1 linha', tone: 'good' },
                { id: '1', label: 'test result: ok. 2 passed', result: 'não apareceu', tone: 'bad' },
              ]}
            />
          </section>
          <section>
            <h3>Sinais de falha</h3>
            <CheckList
              label="Sinais de falha"
              onSelect={() => undefined}
              items={[
                { id: '0', label: 'FAILED', result: 'não apareceu', tone: 'good' },
                { id: '1', label: 'warning', result: 'apareceu · 1 linha', tone: 'bad' },
              ]}
            />
          </section>
        </>
      }
      main={
        <pre style={{ margin: 0, padding: 'var(--s4)', fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace', fontSize: 'var(--font-size-xs)', overflow: 'auto' }}>
          {LOG}
        </pre>
      }
      footer={
        <>
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--s2)' }}>
            <Button variant="secondary" aria-label="Anterior">‹</Button>
            <span style={{ color: 'var(--text-muted)', fontSize: 'var(--font-size-sm)' }}>1 de 3</span>
            <Button variant="secondary" aria-label="Próxima">›</Button>
          </div>
          <div style={{ display: 'flex', gap: 'var(--s2)' }}>
            <Button variant="secondary">Recusar</Button>
            <Button variant="primary">Aceitar</Button>
          </div>
          <ConfirmPopover
            open
            title="Recusar a evidência"
            description="O motivo é obrigatório: é o que o agente vai corrigir."
            fieldLabel="Motivo da recusa"
            placeholder="O que está errado ou falta"
            required
            confirmLabel="Recusar"
            tone="danger"
            onConfirm={() => undefined}
            onCancel={() => undefined}
          />
        </>
      }
    />
  );
}

createRoot(document.getElementById('root') as HTMLElement).render(<Demo />);
