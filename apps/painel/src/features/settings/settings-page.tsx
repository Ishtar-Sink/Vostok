import { useMemo, useState } from 'react';
import { act, useResource } from '../../shared/api';
import { Button } from '../../ui/button/button';
import { Card } from '../../ui/card/card';
import { Checkbox } from '../../ui/checkbox/checkbox';
import { Dialog } from '../../ui/dialog/dialog';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { SegmentedField } from '../../ui/segmented-field/segmented-field';
import { SelectField } from '../../ui/select-field/select-field';
import { ConfigControl } from './fields';
import { FILES, TOKEN_LABELS, humanize, type Control, type Field, type Option, type Section } from './schema';
import { flatten, parseConfig, valueOf, writeConfig, type ConfigNode, type ConfigValue } from './yaml';
import styles from './settings-page.module.css';

interface ConfigFile {
  name: string;
  path: string;
  content: string;
  hash?: string;
}

interface Source {
  conteudo: string;
  hash: string;
}

type Outcome =
  | { kind: 'idle' }
  | { kind: 'saving' }
  | { kind: 'saved'; message: string; problems: string[] }
  | { kind: 'failed'; message: string }
  | { kind: 'conflict'; file: ConfigFile; mine: string; theirs: Source };

async function sha256Hex(text: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

interface Parsed {
  file: ConfigFile;
  intro: string;
  flat: Map<string, ConfigNode>;
}

type Edits = Record<string, Record<string, ConfigValue>>;

const HEX = /^#[0-9a-fA-F]{3}([0-9a-fA-F]{3})?$/;

export function SettingsPage({ version }: { version: number }) {
  const files = useResource<ConfigFile[]>('/api/configuracoes', version);
  const [current, setCurrent] = useState('main.yml');
  const [edits, setEdits] = useState<Edits>({});
  const [outcome, setOutcome] = useState<Outcome>({ kind: 'idle' });

  const parsed = useMemo(() => {
    const map = new Map<string, Parsed>();
    if (files.kind !== 'ready') return map;
    for (const file of files.data) {
      const { intro, nodes } = parseConfig(file.content);
      map.set(file.name, { file, intro, flat: flatten(nodes) });
    }
    return map;
  }, [files]);

  if (files.kind === 'loading') return <LoadingState label="Carregando as configurações" rows={6} />;
  if (files.kind === 'error') return <ErrorState title="Não foi possível ler as configurações" description={files.message} retryLabel="Tentar de novo" onRetry={files.retry} />;

  if (files.data.length === 0) return <EmptyState title="Nenhuma configuração" description="A pasta vostok/config/ não tem arquivos .yml." />;

  const original = (file: string, key: string) => valueOf(parsed.get(file)?.flat.get(key));
  const read = (file: string, key: string) => edits[file]?.[key] ?? original(file, key);
  const write = (file: string, key: string, value: ConfigValue) =>
    setEdits((all) => ({ ...all, [file]: { ...all[file], [key]: value } }));
  const changedEdits = (file: string) => Object.fromEntries(Object.entries(edits[file] ?? {}).filter(([key, value]) => JSON.stringify(value) !== JSON.stringify(original(file, key))));
  const changed = (file: string) => Object.keys(changedEdits(file)).length;
  const totalChanged = [...parsed.keys()].reduce((sum, name) => sum + changed(name), 0);

  async function save() {
    setOutcome({ kind: 'saving' });
    const problems: string[] = [];
    let saved = 0;
    for (const [name, item] of parsed) {
      const changes = changedEdits(name);
      if (Object.keys(changes).length === 0) continue;
      const mine = writeConfig(item.file.content, changes);
      const hash = item.file.hash ?? (await sha256Hex(item.file.content));
      const result = await act('PUT', `/api/configuracao?caminho=${encodeURIComponent(item.file.path)}&hash=${hash}`, mine);
      if (result.status === 409) {
        setOutcome({ kind: 'conflict', file: item.file, mine, theirs: JSON.parse(result.text) as Source });
        return;
      }
      if (!result.ok) {
        setOutcome({ kind: 'failed', message: result.text || `Erro ${result.status}.` });
        return;
      }
      const reply = JSON.parse(result.text) as { problemas?: string[] };
      problems.push(...(reply.problemas ?? []));
      saved++;
      setEdits((all) => ({ ...all, [name]: {} }));
    }
    files.retry();
    setOutcome({ kind: 'saved', message: saved === 1 ? 'Configuração gravada.' : `${saved} configurações gravadas.`, problems });
  }

  function useDisk(name: string) {
    setEdits((all) => ({ ...all, [name]: {} }));
    setOutcome({ kind: 'idle' });
    files.retry();
  }

  const menu = [
    ...FILES.filter((item) => parsed.has(item.name)),
    ...[...parsed.keys()].filter((name) => !FILES.some((item) => item.name === name)).map((name) => ({ name, label: humanize(name.replace(/\.yml$/, '')), summary: '', sections: [] as Section[] })),
  ];
  const page = menu.find((item) => item.name === current) ?? menu[0];
  const file = parsed.get(page.name);

  return (
    <>
      <PageHeader eyebrow="Configurações" title="Harness desta pasta" subtitle="As opções de vostok/config/, com a explicação de cada uma." />
      <div className={styles.layout}>
        <nav aria-label="Grupos de configuração" className={styles.menu}>
          {menu.map((item) => (
            <button key={item.name} type="button" aria-current={page.name === item.name ? 'page' : undefined} onClick={() => setCurrent(item.name)}>
              <span className={styles.menuLabel}>
                {item.label}
                {changed(item.name) > 0 ? <span className={styles.dot} aria-label="com alterações" /> : null}
              </span>
              {item.summary ? <span className={styles.summary}>{item.summary}</span> : null}
            </button>
          ))}
        </nav>
        {file ? (
          <div className={styles.content} key={page.name}>
            <header className={styles.contentHeader}>
              <h2>{page.label}</h2>
              <p className={styles.path}>{file.file.path}</p>
            </header>
            {page.sections.map((section) => (
              <SectionCard key={section.title} section={section} flat={file.flat} read={(key) => read(page.name, key)} write={(key, value) => write(page.name, key, value)} lookup={(key) => lookup(key, page.name, read)} />
            ))}
            <Advanced flat={file.flat} sections={page.sections} read={(key) => read(page.name, key)} write={(key, value) => write(page.name, key, value)} open={page.sections.length === 0} />
          </div>
        ) : null}
      </div>
      {totalChanged > 0 || outcome.kind === 'saved' || outcome.kind === 'failed' ? (
        <div className={styles.saveBar} role="status">
          <span className={styles.saveText}>
            {outcome.kind === 'saved' ? (
              <>
                <strong>{outcome.message}</strong>
                {outcome.problems.length > 0 ? ' A validação do vostok acusou problemas:' : ' Sem problemas na validação.'}
                {outcome.problems.length > 0 ? <ul className={styles.problems}>{outcome.problems.map((problem) => <li key={problem}>{problem}</li>)}</ul> : null}
              </>
            ) : outcome.kind === 'failed' ? (
              <span role="alert">{outcome.message}</span>
            ) : (
              <>
                <strong>{totalChanged} {totalChanged === 1 ? 'alteração' : 'alterações'}</strong> não salvas.
              </>
            )}
          </span>
          <span className={styles.saveActions}>
            {outcome.kind === 'saved' ? (
              <Button variant="primary" onClick={() => setOutcome({ kind: 'idle' })}>Fechar</Button>
            ) : (
              <>
                <Button variant="ghost" onClick={() => { setEdits({}); setOutcome({ kind: 'idle' }); }}>Descartar</Button>
                <Button variant="primary" disabled={outcome.kind === 'saving' || totalChanged === 0} onClick={() => void save()}>
                  {outcome.kind === 'saving' ? 'Salvando…' : 'Salvar'}
                </Button>
              </>
            )}
          </span>
        </div>
      ) : null}
      {outcome.kind === 'conflict' ? (
        <Dialog
          open
          title={`O arquivo mudou no disco: ${outcome.file.name}`}
          onClose={() => setOutcome({ kind: 'idle' })}
          size="wide"
          actions={
            <>
              <Button variant="ghost" onClick={() => setOutcome({ kind: 'idle' })}>Fechar</Button>
              <Button variant="secondary" onClick={() => useDisk(outcome.file.name)}>Usar a versão do disco</Button>
            </>
          }
        >
          <p className={styles.conflictNote}><strong>Nada foi gravado.</strong> Compare os dois textos: as suas alterações continuam na tela.</p>
          <div className={styles.columns}>
            <label>
              <span>Seu texto</span>
              <textarea readOnly rows={12} value={outcome.mine} />
            </label>
            <label>
              <span>Versão no disco</span>
              <textarea readOnly rows={12} value={outcome.theirs.conteudo} />
            </label>
          </div>
        </Dialog>
      ) : null}
    </>
  );
}

function lookup(key: string, currentFile: string, read: (file: string, key: string) => ConfigValue): ConfigValue {
  const [file, path] = key.includes(':') ? key.split(':') : [currentFile, key];
  return read(file, path);
}

interface SectionProps {
  section: Section;
  flat: Map<string, ConfigNode>;
  read: (key: string) => ConfigValue;
  write: (key: string, value: ConfigValue) => void;
  lookup: (key: string) => ConfigValue;
}

function switcherOptions(section: Section, flat: Map<string, ConfigNode>): Option[] {
  if (!section.switcher) return [];
  if (section.switcher.options) return section.switcher.options;
  const prefix = `${section.switcher.fromKeys}.`;
  return [...flat.keys()].filter((key) => key.startsWith(prefix)).map((key) => key.slice(prefix.length)).map((key) => ({ value: key, label: key }));
}

function SectionCard({ section, flat, read, write, lookup }: SectionProps) {
  const options = switcherOptions(section, flat);
  const [item, setItem] = useState(options[0]?.value ?? '');
  const resolve = (key: string) => key.replaceAll('{item}', item);
  const keys = section.fields.map((field) => resolve(field.key));
  const [enabled, setEnabled] = useState(() => !!section.toggle && valueText(read(section.toggle.key)) !== '');
  const on = !section.toggle || enabled;

  return (
    <Card as="section" className={styles.section} aria-label={section.title}>
      <header className={styles.sectionHeader}>
        <div>
          <h3>{section.title}</h3>
          {section.description ? <p className={styles.description}>{section.description}</p> : null}
        </div>
        {section.switcher ? (
          <div className={styles.switcher}>
            {section.switcher.style === 'select' ? (
              <SelectField label={section.switcher.label} options={options} value={item} onChange={setItem} />
            ) : (
              <SegmentedField label={section.switcher.label} options={options} value={item} onChange={setItem} />
            )}
          </div>
        ) : null}
      </header>
      {section.toggle ? (
        <div className={styles.toggle}>
          <Checkbox
            label={section.toggle.label}
            checked={enabled}
            onChange={(event) => {
              const next = event.currentTarget.checked;
              setEnabled(next);
              if (!next) (section.toggle?.clearAll ? keys : [section.toggle!.key]).forEach((key) => write(key, Array.isArray(read(key)) ? [] : ''));
            }}
          />
          {section.toggle.help ? <p className={styles.description}>{section.toggle.help}</p> : null}
        </div>
      ) : null}
      {section.tokens ? <Tokens flat={flat} theme={item} exclude={section.tokens.exclude} note={section.tokens.note} /> : null}
      {section.fields.length > 0 ? (
        <fieldset className={styles.grid} disabled={!on} key={item}>
          {section.fields.map((field) => (
            <FieldView key={field.key} field={field} resolve={resolve} flat={flat} read={read} write={write} lookup={lookup} />
          ))}
        </fieldset>
      ) : null}
    </Card>
  );
}

function FieldView({ field, resolve, flat, read, write, lookup }: { field: Field; resolve: (key: string) => string } & Omit<SectionProps, 'section'>) {
  const key = resolve(field.key);
  const node = flat.get(key);
  const help = field.help ?? cleanHelp(node?.help || node?.inlineComment || '');
  let control: Control = field.control;
  if (control.type === 'id') control = { ...control, digits: resolve(control.digits) };
  if (control.type === 'checklist' && control.within) {
    const allowed = lookup(resolve(control.within));
    if (Array.isArray(allowed) && allowed.length > 0) control = { ...control, options: control.options.filter((option) => allowed.includes(option.value)) };
  }
  return (
    <div className={styles.cell} data-span={field.wide ? 'full' : undefined}>
      <ConfigControl label={field.label} help={help || undefined} control={control} value={read(key)} onChange={(value) => write(key, value)} lookup={lookup} setOther={write} />
    </div>
  );
}

/** Grade das cores derivadas de um tema, só para conferir. */
function Tokens({ flat, theme, exclude, note }: { flat: Map<string, ConfigNode>; theme: string; exclude: RegExp; note: string }) {
  const prefix = `${theme}.`;
  const entries = [...flat.entries()].filter(([key]) => key.startsWith(prefix)).map(([key, node]) => [key.slice(prefix.length), node.value ?? ''] as const);
  const main = entries.filter(([key]) => !exclude.test(key));
  const others = entries.filter(([key]) => exclude.test(key));
  return (
    <div className={styles.tokens}>
      <p className={styles.note}>{note}</p>
      {main.length === 0 ? <p className={styles.description}>Sem cores derivadas: vale a identidade padrão do harness.</p> : null}
      <ul className={styles.swatches}>
        {main.map(([key, value]) => (
          <li key={key}>
            <span className={styles.swatch} style={{ background: value }} aria-hidden="true" />
            <span className={styles.swatchText}>
              <span>{TOKEN_LABELS[key] ?? humanize(key)}</span>
              <code>{value}</code>
            </span>
          </li>
        ))}
      </ul>
      {others.length > 0 ? (
        <details className={styles.details}>
          <summary>Tokens antigos e sombras ({others.length})</summary>
          <ul className={styles.swatches}>
            {others.map(([key, value]) => (
              <li key={key}>
                {HEX.test(value) ? <span className={styles.swatch} style={{ background: value }} aria-hidden="true" /> : <span className={styles.shadow} style={{ boxShadow: value }} aria-hidden="true" />}
                <span className={styles.swatchText}>
                  <span>{humanize(key)}</span>
                  <code>{value}</code>
                </span>
              </li>
            ))}
          </ul>
        </details>
      ) : null}
    </div>
  );
}

/** Opções sem mapeamento na tela, com o nome derivado da chave e o controle deduzido do valor. */
function Advanced({ flat, sections, read, write, open }: { flat: Map<string, ConfigNode>; sections: Section[]; read: (key: string) => ConfigValue; write: (key: string, value: ConfigValue) => void; open: boolean }) {
  const covered = coveredKeys(sections, flat);
  const rest = [...flat.entries()].filter(([key]) => !covered(key));
  if (rest.length === 0) return null;
  return (
    <Card as="section" className={styles.section} aria-label="Avançado">
      <details className={styles.advanced} open={open}>
        <summary>
          <h3>Avançado</h3>
          <span className={styles.description}>{rest.length} opções internas do harness: nomes de seções, templates e formatos lidos pelos comandos.</span>
        </summary>
        <div className={styles.grid}>
          {rest.map(([key, node]) => {
            const value = read(key);
            return (
              <div key={key} className={styles.cell} data-span={Array.isArray(value) || valueText(value).includes('\n') ? 'full' : undefined}>
                <ConfigControl label={humanize(key)} help={[cleanHelp(node.help || node.inlineComment || ''), key].filter(Boolean).join(' · ')} control={guess(node)} value={value} onChange={(next) => write(key, next)} lookup={read} setOther={write} />
              </div>
            );
          })}
        </div>
      </details>
    </Card>
  );
}

function coveredKeys(sections: Section[], flat: Map<string, ConfigNode>): (key: string) => boolean {
  const keys = new Set<string>();
  const prefixes: string[] = [];
  for (const section of sections) {
    const items = section.switcher ? switcherOptions(section, flat).map((option) => option.value) : [''];
    for (const item of items) {
      if (section.tokens) prefixes.push(`${item}.`);
      for (const field of section.fields) {
        keys.add(field.key.replaceAll('{item}', item));
        if (field.control.type === 'id') keys.add(field.control.digits.replaceAll('{item}', item));
      }
    }
  }
  return (key) => keys.has(key) || prefixes.some((prefix) => key.startsWith(prefix)) || [...keys].some((other) => other.startsWith(`${key}.`));
}

function guess(node: ConfigNode): Control {
  if (node.list) return { type: 'tags' };
  const value = node.value ?? '';
  if (value.includes('\n')) return { type: 'textarea', mono: true };
  if (/^\d+$/.test(value)) return { type: 'number', min: 0 };
  if (HEX.test(value)) return { type: 'color' };
  return { type: 'text', mono: true };
}

function cleanHelp(text: string): string {
  const clean = text.replace(/`/g, '');
  return clean.charAt(0).toUpperCase() + clean.slice(1);
}

function valueText(value: ConfigValue): string {
  return Array.isArray(value) ? value.join(',') : value;
}
