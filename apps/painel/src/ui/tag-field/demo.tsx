import { createRoot } from 'react-dom/client';
import { useState } from 'react';
import '../../theme/tokens.css';
import '../../app.css';
import { Checkbox } from '../checkbox/checkbox';
import { NumberField } from '../number-field/number-field';
import { Switch } from '../switch/switch';
import { TagField } from './tag-field';

const params = new URLSearchParams(window.location.search);
document.documentElement.dataset.theme = params.get('tema') === 'escuro' ? 'dark' : 'light';
const withErrors = params.get('estado') === 'erro';

const KINDS = ['automática', 'automática, da verificação', 'diff', 'manual', 'print'];
const TEXT_SUGGESTIONS = ['out', 'txt', 'log', 'md', 'json', 'ndjson', 'csv', 'yaml'];
const ALERT_SUGGESTIONS = ['FAILED', 'error', 'warning', 'panicked', 'fatal', 'Traceback'];

const extension = (text: string) => {
  const value = text.trim().replace(/^\.+/, '').toLowerCase();
  return /^[a-z0-9]+$/.test(value) ? value : null;
};

function Demo() {
  const [kinds, setKinds] = useState<string[]>(
    withErrors ? [] : KINDS.filter((kind) => kind !== 'diff'),
  );
  const [timeout, setTimeoutValue] = useState<number | null>(withErrors ? -1 : 120);
  const [titleMax, setTitleMax] = useState<number | null>(withErrors ? 500 : 60);
  const [persist, setPersist] = useState(!withErrors);
  const [textExtensions, setTextExtensions] = useState(
    withErrors ? ['txt', 'log', 'md', 'png'] : ['txt', 'log', 'md'],
  );
  const [alerts, setAlerts] = useState(['FAILED', 'error']);

  function toggle(kind: string, checked: boolean) {
    setKinds(KINDS.filter((item) => (item === kind ? checked : kinds.includes(item))));
  }

  return (
    <main
      style={{
        boxSizing: 'border-box',
        maxWidth: '760px',
        margin: '0 auto',
        padding: 'var(--s6) var(--s4)',
        display: 'grid',
        gap: 'var(--s6)',
      }}
    >
      <h1 style={{ margin: 0, fontFamily: 'var(--font-display)', fontSize: 'var(--font-size-xl)' }}>
        Componentes de formulário
      </h1>
      <fieldset
        style={{
          margin: 0,
          padding: 'var(--s4)',
          display: 'grid',
          gap: 'var(--s3)',
          border: '1px solid var(--border)',
          borderRadius: 'var(--radius-md)',
        }}
      >
        <legend>Tipos que pedem conferência</legend>
        {KINDS.map((kind) => (
          <Checkbox
            key={kind}
            label={kind.charAt(0).toUpperCase() + kind.slice(1)}
            help={kind === 'diff' ? 'Pede conferência das evidências de diff.' : undefined}
            checked={kinds.includes(kind)}
            onChange={(checked) => toggle(kind, checked)}
          />
        ))}
        <Checkbox label="Exemplo desabilitado" checked disabled onChange={() => undefined} />
      </fieldset>
      <NumberField
        label="Tempo-limite no baseline"
        unit="s"
        min={0}
        max={3600}
        help="0 sem limite."
        value={timeout}
        error={withErrors ? 'Use um valor de 0 a 3600 segundos.' : undefined}
        onChange={setTimeoutValue}
      />
      <NumberField
        label="Tamanho máximo do título"
        unit="caracteres"
        min={20}
        max={200}
        value={titleMax}
        error={withErrors ? 'Use um valor de 20 a 200 caracteres.' : undefined}
        onChange={setTitleMax}
      />
      <Switch
        label="Copiar as evidências para o projeto"
        help="Desligado, a pasta de destino fica vazia."
        checked={persist}
        onChange={setPersist}
      />
      <TagField
        label="Extensões de texto"
        help="Sem ponto, em minúsculas. Escolha uma sugestão ou digite e tecle Enter."
        placeholder="Nova extensão"
        values={textExtensions}
        suggestions={TEXT_SUGGESTIONS}
        normalize={extension}
        error={withErrors ? 'A extensão png já está nas de imagem.' : undefined}
        onChange={setTextExtensions}
      />
      <TagField
        label="Trechos de alerta"
        values={alerts}
        suggestions={ALERT_SUGGESTIONS}
        placeholder="Novo trecho"
        onChange={setAlerts}
      />
    </main>
  );
}

createRoot(document.getElementById('root') as HTMLElement).render(<Demo />);
