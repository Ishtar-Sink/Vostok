import { Fragment, useEffect, useMemo, useRef, useState } from 'react';
import { fileUrl, formatDate, listOf, readTextPreview, useResource, type EvidenceReview, type FileRef, type ManualEvidence, type ReviewStatus, type SpecDetail, type TaskDetail } from '../../shared/api';
import { BackLink } from '../../shared/back-link';
import { buildReferences } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Markdown, slug } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Badge, type BadgeTone } from '../../ui/badge/badge';
import { CopyButton } from '../../ui/copy-button/copy-button';
import { ImageViewer } from '../../ui/image-viewer/image-viewer';
import { VideoPlayer } from '../../ui/video-player/video-player';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { Tabs } from '../../ui/tabs/tabs';
import page from '../../shared/page.module.css';
import styles from './task-page.module.css';

/** Ordem de leitura: primeiro o que interessa a quem decide, depois o que é roteiro do agente. */
const HUMAN = [
  { file: 'executivo', title: 'Resumo executivo' },
  { file: 'tecnico', title: 'Detalhes técnicos' },
  { file: 'criterios', title: 'Critérios de conclusão' },
];
const AGENT = [
  { file: 'execucao', title: 'Execução' },
  { file: 'verificacao', title: 'Verificação' },
  { file: 'registro', title: 'Registro de execução' },
];

const TASK_TABS = [
  { id: 'resumo', label: 'Resumo' },
  { id: 'evidencias', label: 'Evidências' },
  { id: 'agente', label: 'Para o agente' },
];

function tabFromHash(hash: string): string {
  const id = hash.replace(/^#/, '');
  return TASK_TABS.some((tab) => tab.id === id) ? id : 'resumo';
}

const IMAGE = /\.(png|jpe?g|gif|webp)$/i;
const VIDEO = /\.(mp4|webm|mov)$/i;
const TEXT = /\.(md|txt|yml|log|out|diff|csv|json|css|js)$/i;

type EvidenceFilter = 'todas' | 'a conferir' | 'conferidas';
const EVIDENCE_FILTERS: { value: EvidenceFilter; label: string }[] = [
  { value: 'todas', label: 'Todas' },
  { value: 'a conferir', label: 'A conferir' },
  { value: 'conferidas', label: 'Conferidas' },
];

function inFilter(name: string, filter: EvidenceFilter, reviews: Map<string, EvidenceReview>): boolean {
  if (filter === 'todas') return true;
  const status = reviews.get(name)?.status;
  return filter === 'a conferir' ? status === 'a conferir' : status === 'conferida';
}

/** As quatro primeiras linhas de um log gravado pelo harness: comando, data, commit e código de saída. */
const LOG_HEADER = [/^\$ /, /^# data:/, /^# commit:/, /^# código de saída:/];
const normalize = (text: string) => text.replace(/\s+/g, ' ').trim();

const ELLIPSIS = /\.{3}|…/;
const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** Índices das linhas da saída do log que o trecho aponta: as iguais a ele e, sem nenhuma, as que o contêm como palavra inteira (`0 passed` não casa com `10 passed`). `...` e `…` valem como qualquer trecho da mesma linha, e as partes entre eles aparecem em ordem. */
export function findLines(log: string, snippet: string): number[] {
  const parts = snippet.split(new RegExp(ELLIPSIS, 'g')).map(normalize);
  const wanted = parts.filter(Boolean);
  if (wanted.length === 0) return [];
  const lines = log.split('\n');
  let start = 0;
  while (start < LOG_HEADER.length && start < lines.length && LOG_HEADER[start].test(lines[start])) start += 1;
  const body = lines.map((line, index) => ({ text: normalize(line), index })).slice(start);
  const hasEllipsis = parts.length > 1;
  const before = parts[0] ? '(?<![\\p{L}\\p{N}])' : '';
  const after = parts[parts.length - 1] ? '(?![\\p{L}\\p{N}])' : '';
  const whole = new RegExp(`${before}${wanted.map(escapeRegExp).join('.*')}${after}`, 'u');
  const equal = hasEllipsis ? [] : body.filter((line) => line.text === wanted[0]);
  return (equal.length > 0 ? equal : body.filter((line) => whole.test(line.text))).map((line) => line.index);
}

function snippetsOf(value: string): string[] {
  return [...value.matchAll(/`([^`]+)`/g)].map((match) => match[1]);
}

export function TaskPage({ spec, task, version }: { spec: string; task: string; version: number }) {
  const detail = useResource<TaskDetail>(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}`, version);
  const specDetail = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(spec)}`, version);
  const references = useMemo(() => (specDetail.kind === 'ready' ? buildReferences(specDetail.data.markdown, specDetail.data.tasks, spec) : new Map()), [specDetail, spec]);
  const { bind, popover } = useReferencePopover(references);
  const notes = useMemo(() => evidenceNotes(detail.kind === 'ready' ? detail.data.files.verificacao ?? '' : ''), [detail]);
  const [tab, setTab] = useState(() => tabFromHash(window.location.hash));
  const [filter, setFilter] = useState<EvidenceFilter>('todas');
  function select(id: string) {
    setTab(id);
    history.replaceState(history.state, '', '#' + id);
  }

  if (detail.kind === 'loading') return <LoadingState label={`Carregando ${task}`} rows={8} />;
  if (detail.kind === 'error') return <ErrorState title={`Não foi possível abrir ${task}`} description={detail.message} retryLabel="Tentar de novo" onRetry={detail.retry} />;

  const { state, files, evidence, manual_evidence: manualList } = detail.data;
  const manual = manualList ?? [];
  const manualNames = new Set(manual.map((item) => item.name));
  const otherEvidence = evidence.filter((file) => !manualNames.has(file.name));
  const reviews = new Map((detail.data.reviews ?? []).map((item) => [item.name, item] as const));
  const shownManual = manual.filter((item) => inFilter(item.name, filter, reviews));
  const shownOther = otherEvidence.filter((file) => inFilter(file.name, filter, reviews));
  const total = manual.length + otherEvidence.length;
  const results: [string, string][] = [
    ['Testes', state.tests_result],
    ['Regressão', state.regression_result],
    ['Evidências', state.evidence_result],
  ];
  const waitingOn = listOf(state.waiting_on ?? '');
  const closesAfter = specDetail.kind === 'ready' ? listOf(state.closes_after ?? '').filter((id) => specDetail.data.tasks.find((item) => item.id === id)?.status !== 'done') : [];

  return (
    <div className={page.stack}>
      <PageHeader back={<BackLink fallback={`/specs/${spec}/quadro`} label={`Quadro de ${spec}`} />} eyebrow={`${spec} · ${state.id}`} title={state.title} subtitle={state.description?.replace(/`/g, "")} actions={
          <div className={styles.headerActions}>
            <Badge tone={statusTone(state.status)}>{statusLabel(state.status)}</Badge>
          </div>
        } />
      <div className={page.docLayout}>
        <div className={page.docMain}>
          <Tabs
            label="Partes da tarefa"
            value={tab}
            onChange={select}
            tabs={[
              {
                id: 'resumo',
                label: 'Resumo',
                panel: HUMAN.map((part) => (
                  <section key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
                    <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
                    {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Ainda não escrito.</p>}
                  </section>
                )),
              },
              {
                id: 'evidencias',
                label: 'Evidências',
                panel: (
                  <section aria-labelledby="evidencias" className={styles.part}>
                    <h2 id="evidencias" className={styles.partTitle}>Evidências</h2>
                    {total === 0 ? <p className={styles.muted}>Elas aparecem aqui quando a tarefa é finalizada.</p> : null}
                    {total > 0 ? (
                      <div role="group" aria-label="Filtrar evidências" className={styles.evidenceFilter}>
                        {EVIDENCE_FILTERS.map(({ value, label }) => (
                          <button key={value} type="button" aria-pressed={filter === value} onClick={() => setFilter(value)}>
                            {label} <span>{[...manual, ...otherEvidence].filter((item) => inFilter(item.name, value, reviews)).length}</span>
                          </button>
                        ))}
                      </div>
                    ) : null}
                    {total > 0 && shownManual.length === 0 && shownOther.length === 0 ? <p className={styles.muted}>Nenhuma evidência neste filtro.</p> : null}
                    {shownManual.length > 0 ? (
                      <ul aria-label="Evidências que o harness pede" className={styles.manualList}>
                        {shownManual.map((item) => (
                          <ManualEvidenceItem key={item.name} item={item} reviews={reviews} notes={notes} />
                        ))}
                      </ul>
                    ) : null}
                    {shownOther.length > 0 ? <Evidence files={shownOther} reviews={reviews} notes={notes} /> : null}
                  </section>
                ),
              },
              {
                id: 'agente',
                label: 'Para o agente',
                panel: (
                  <>
                    <p className={styles.tabIntro}>Roteiro que o agente segue: arquivos, passos, comandos de verificação e o registro de cada etapa.</p>
                    {AGENT.map((part) => (
                      <section key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
                        <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
                        {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Vazio.</p>}
                      </section>
                    ))}
                  </>
                ),
              },
            ]}
          />
        </div>
        <aside className={page.docAside}>
          <dl className={page.meta}>
            <div><dt>Status</dt><dd>{statusLabel(state.status)}</dd></div>
            {state.wave ? <div><dt>Onda</dt><dd>{state.wave}</dd></div> : null}
            {waitingOn.length > 0 ? <div><dt>Espera</dt><dd>{`${waitingOn.join(', ')}${state.waiting_reason ? `: ${state.waiting_reason}` : ''}`}</dd></div> : null}
            {closesAfter.length > 0 ? <div><dt>Fecha depois de</dt><dd>{closesAfter.map((id) => <a key={id} href={`/tarefas/${spec}/${id}`} onClick={onLinkClick} className={styles.dep}>{id}</a>)}</dd></div> : null}
            <div><dt>Depende de</dt><dd>{listOf(state.depends_on ?? '').map((dep) => <a key={dep} href={`/tarefas/${spec}/${dep}`} onClick={onLinkClick} className={styles.dep}>{dep}</a>)}{listOf(state.depends_on ?? '').length === 0 ? '—' : null}</dd></div>
            <div><dt>Cobre</dt><dd ref={bind} className={styles.refList}>{listOf(state.covers ?? '').map((id) => <button key={id} type="button" className="ref" data-ref={id} aria-haspopup="dialog">{id}</button>)}{listOf(state.covers ?? '').length === 0 ? '—' : null}</dd></div>
            {state.profile ? <div><dt>Perfil</dt><dd>{state.profile}</dd></div> : null}
            {state.persona ? <div><dt>Persona</dt><dd>{state.persona}</dd></div> : null}
            <div><dt>Modelo</dt><dd>{state.model || '—'}</dd></div>
            <div><dt>Aprovada</dt><dd>{formatDate(state.approved_at ?? '')}</dd></div>
            <div><dt>Tentativas</dt><dd>{state.attempts ?? '0'}</dd></div>
          </dl>
          <section>
            <p className={page.asideTitle}>Validação</p>
            <ul className={styles.results}>
              {results.map(([label, value]) => (
                <li key={label}>
                  <span>{label}</span>
                  <Badge tone={statusTone(value)}>{value && value !== 'null' ? statusLabel(value) : 'pendente'}</Badge>
                </li>
              ))}
            </ul>
          </section>
        </aside>
      </div>
      {popover}
    </div>
  );
}

function stripTitle(markdown: string): string {
  return markdown.replace(/^##+ .*\n/, '').replace(/^### /gm, '#### ');
}

interface EvidenceNotes {
  title?: string;
  proof?: string;
  expect?: string;
  excerpt?: string;
  failure?: string;
}

const NOTE_FIELDS: Record<string, keyof EvidenceNotes> = { Título: 'title', Prova: 'proof', Espera: 'expect', Trecho: 'excerpt', Falha: 'failure' };

/** Lê os subitens de cada evidência na seção `### Evidências` da verificação; o painel só mostra o que a tarefa escreveu. */
function evidenceNotes(verification: string): Map<string, EvidenceNotes> {
  const notes = new Map<string, EvidenceNotes>();
  let inSection = false;
  let current: EvidenceNotes | null = null;
  for (const line of verification.split('\n')) {
    if (line.startsWith('#')) {
      inSection = /^###\s+Evidências\s*$/.test(line);
      current = null;
      continue;
    }
    if (!inSection) continue;
    const opened = line.match(/^- `([^`]+)`/);
    if (opened) {
      current = {};
      notes.set(opened[1], current);
      continue;
    }
    const field = line.match(/^\s{2,}- (Título|Prova|Espera|Trecho|Falha):\s*(.*)$/);
    if (field && current) current[NOTE_FIELDS[field[1]]] = field[2];
  }
  return notes;
}

type Notes = Map<string, EvidenceNotes>;

function Evidence({ files, reviews, notes }: { files: FileRef[]; reviews: Map<string, EvidenceReview>; notes: Notes }) {
  return (
    <ul className={styles.evidenceList}>
      {files.map((file) => <EvidenceItem key={file.path} file={file} reviews={reviews} notes={notes} />)}
    </ul>
  );
}

interface ManualEvidenceItemProps {
  item: ManualEvidence;
  reviews: Map<string, EvidenceReview>;
  notes: Notes;
}

/** Evidência manual pedida pelo harness, com o que ela deve provar e o selo de anexada; o painel só mostra. */
function ManualEvidenceItem({ item, reviews, notes }: ManualEvidenceItemProps) {
  return (
    <li className={styles.manualItem}>
      <div className={styles.manualHead}>
        <div className={styles.manualText}>
          <span className={styles.evidenceName}>{item.name}</span>
          <p className={styles.manualAsk}>{item.description}</p>
        </div>
        <Badge tone={item.attached ? 'success' : 'warning'}>{item.attached ? 'anexado' : 'falta anexar'}</Badge>
      </div>
      {item.attached ? (
        <ul className={styles.evidenceList}>
          <EvidenceItem file={item} reviews={reviews} notes={notes} />
        </ul>
      ) : null}
    </li>
  );
}

interface Snippets {
  log: string | null;
  onActivate: (snippet: string) => void;
  onDeactivate: () => void;
}

const SNIPPET_COPY_MS = 1500;
const COPIED_MESSAGE = 'Copiado';
const COPY_FAILED_MESSAGE = 'Não foi possível copiar';

/** Texto do Trecho ou da Falha: cada parte em crases vira um botão que destaca a linha no log (e a copia ao clicar), com a contagem quando o log já carregou. */
function SnippetField({ label, value, kind, snippets }: { label: string; value: string; kind: 'excerpt' | 'failure'; snippets: Snippets }) {
  const found = new Set(snippetsOf(value));
  const [copy, setCopy] = useState<{ snippet: string; ok: boolean } | null>(null);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);

  function announce(snippet: string, ok: boolean) {
    setCopy({ snippet, ok });
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setCopy(null), SNIPPET_COPY_MS);
  }

  function activate(snippet: string) {
    snippets.onActivate(snippet);
    const written = navigator.clipboard?.writeText(snippet);
    if (!written) {
      announce(snippet, false);
      return;
    }
    written.then(() => announce(snippet, true), () => announce(snippet, false));
  }

  return (
    <div className={styles.evidenceField}>
      <span className={styles.evidenceLabel}>{label}</span>
      <p className={styles.snippetText}>
        {value.split(/(`[^`]+`)/).map((part, index) => {
          if (!part.startsWith('`') || !found.has(part.slice(1, -1))) return part;
          const snippet = part.slice(1, -1);
          const count = snippets.log === null ? null : findLines(snippets.log, snippet).length;
          return (
            <span key={index}>
              <button type="button" className={styles.snippet} aria-label={`Destacar ${snippet} no log`} onMouseEnter={() => snippets.onActivate(snippet)} onFocus={() => snippets.onActivate(snippet)} onClick={() => activate(snippet)} onMouseLeave={snippets.onDeactivate} onBlur={snippets.onDeactivate}>
                <code>{snippet}</code>
              </button>
              {count === null ? null : count > 0 ? <span className={styles.snippetCount}>{count === 1 ? '1 linha' : `${count} linhas`}</span> : <Badge tone={kind === 'excerpt' ? 'warning' : 'success'}>não encontrado</Badge>}
              {copy?.snippet === snippet ? (
                <span className={styles.snippetCopy} data-ok={copy.ok}>
                  {copy.ok ? (
                    <svg aria-hidden="true" data-icon="check" viewBox="0 0 16 16" width="12" height="12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M3 8.5l3.5 3.5L13 4.5" />
                    </svg>
                  ) : null}
                  {copy.ok ? COPIED_MESSAGE : COPY_FAILED_MESSAGE}
                </span>
              ) : null}
            </span>
          );
        })}
      </p>
      <span role="status" className={styles.srOnly}>{copy ? (copy.ok ? COPIED_MESSAGE : COPY_FAILED_MESSAGE) : ''}</span>
    </div>
  );
}

function EvidenceHead({ file, note, id, snippets }: { file: FileRef; note: EvidenceNotes; id?: string; snippets?: Snippets }) {
  return (
    <>
      <h3 className={styles.evidenceTitle}>
        {id ? <span className={styles.evidenceId}>[{id}]</span> : null}
        {id ? ' ' : null}
        {note.title ?? <span className={styles.evidenceName}>{file.name}</span>}
      </h3>
      {([['Prova', note.proof], ['Espera', note.expect], ['Trecho', note.excerpt], ['Falha', note.failure]] as const).map(([label, value]) => {
        if (!value) return null;
        if (snippets && (label === 'Trecho' || label === 'Falha')) return <SnippetField key={label} label={label} value={value} kind={label === 'Trecho' ? 'excerpt' : 'failure'} snippets={snippets} />;
        return (
          <div key={label} className={styles.evidenceField}>
            <span className={styles.evidenceLabel}>{label}</span>
            <Markdown source={value} prefix={`ev-${file.name}-${label}-`} />
          </div>
        );
      })}
    </>
  );
}

function EvidenceFoot({ file, note }: { file: FileRef; note: EvidenceNotes }) {
  return (
    <p className={styles.evidenceFile}>
      {note.title ? <span className={styles.evidenceName}>{file.name}</span> : null}
      <span>{formatSize(file.size)}</span>
    </p>
  );
}

function EvidenceItem({ file, reviews, notes }: { file: FileRef; reviews: Map<string, EvidenceReview>; notes: Notes }) {
  const mediaUrl = fileUrl(file.path);
  const media = IMAGE.test(file.name) || VIDEO.test(file.name);
  const [open, setOpen] = useState(media);
  const [text, setText] = useState<string | null>(null);
  const [total, setTotal] = useState<number | null>(null);
  const [active, setActive] = useState<string | null>(null);
  const preRef = useRef<HTMLPreElement>(null);
  const note = notes.get(file.name) ?? {};
  const review = reviews.get(file.name);
  const id = review?.id;
  const hits = useMemo(() => (active !== null && text !== null ? new Set(findLines(text, active)) : new Set<number>()), [active, text]);
  const first = hits.size > 0 ? Math.min(...hits) : null;

  useEffect(() => {
    if (first === null) return;
    preRef.current?.querySelector(`[data-line="${first}"]`)?.scrollIntoView({ block: 'nearest' });
  }, [active, text]);

  if (!media && !TEXT.test(file.name)) {
    return (
      <li className={styles.evidenceItem}>
        <EvidenceHead file={file} note={note} id={id} />
        {review ? <ReviewRow review={review} /> : null}
        <a className={styles.downloadLink} href={fileUrl(file.path)} download={file.name} aria-label={`Baixar ${file.name}`}>Baixar</a>
        <EvidenceFoot file={file} note={note} />
      </li>
    );
  }

  function load() {
    if (media || text !== null) return;
    if (file.size === 0) {
      setText('');
      return;
    }
    readTextPreview(file.path)
      .then((preview) => {
        setText(preview.text);
        setTotal(preview.total);
      })
      .catch(() => setText('Não foi possível ler o arquivo.'));
  }

  function toggle() {
    const next = !open;
    setOpen(next);
    if (next) load();
  }

  function activate(snippet: string) {
    setActive(snippet);
    setOpen(true);
    load();
  }

  const snippets = media ? undefined : { log: text, onActivate: activate, onDeactivate: () => setActive(null) };

  return (
    <li className={styles.evidenceItem}>
      <EvidenceHead file={file} note={note} id={id} snippets={snippets} />
      {review ? <ReviewRow review={review} /> : null}
      {media ? null : (
        <button type="button" className={styles.evidenceButton} aria-expanded={open} aria-label={`Ver ${file.name}`} onClick={toggle}>Ver conteúdo</button>
      )}
      {open ? (
        IMAGE.test(file.name) ? (
          <ImageViewer className={styles.evidenceMedia} src={mediaUrl} alt={`Evidência ${file.name}`} />
        ) : VIDEO.test(file.name) ? (
          <VideoPlayer className={styles.evidenceMedia} src={mediaUrl} label={`Vídeo ${file.name}`} />
        ) : (
          <>
            <div className={styles.evidenceBlock}>
              {text !== null ? <CopyButton className={styles.copy} text={text} label={`Copiar ${file.name}`} /> : null}
              <pre ref={preRef} className={styles.evidenceText}>
                {text !== null && active !== null
                  ? text.split('\n').map((line, index, lines) => (
                      <Fragment key={index}>
                        <span data-line={index} className={hits.has(index) ? styles.hit : undefined}>{line}</span>
                        {index < lines.length - 1 ? '\n' : ''}
                      </Fragment>
                    ))
                  : text ?? 'Carregando…'}
              </pre>
            </div>
            {total !== null ? (
              <p className={styles.reviewMessage} role="status">
                Mostrando o primeiro 1 MB de {formatSize(total)}. <a className={styles.downloadLink} href={fileUrl(file.path)} download={file.name}>Baixar o arquivo inteiro</a>
              </p>
            ) : null}
          </>
        )
      ) : null}
      <EvidenceFoot file={file} note={note} />
    </li>
  );
}

const REVIEW_TONE: Record<ReviewStatus, BadgeTone> = {
  conferida: 'success',
  rejeitada: 'danger',
  'a conferir': 'warning',
  'sem aval': 'neutral',
};

/** Estado da conferência da evidência: o painel só mostra o que a CLI calculou; quem confere é o usuário, pelo chat. */
function ReviewRow({ review }: { review: EvidenceReview }) {
  const tone = REVIEW_TONE[review.status];
  if (!tone) return null;
  return (
    <div className={styles.review}>
      <Badge tone={tone}>{review.status}</Badge>
      {review.status === 'rejeitada' && review.reason ? <p className={styles.reviewReason}>Motivo: {review.reason}</p> : null}
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1).replace('.', ',')} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1).replace('.', ',')} MB`;
}
