import { useEffect, useMemo, useState } from 'react';
import { act, fileUrl, formatDate, listOf, useResource, type FileRef, type SpecDetail, type TaskDetail } from '../../shared/api';
import { BackLink } from '../../shared/back-link';
import { EditDialog } from '../../shared/edit-dialog';
import { buildReferences } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Markdown, slug, type Heading } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Toc } from '../../shared/toc';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { Card } from '../../ui/card/card';
import { ConfirmDialog } from '../../ui/confirm-dialog/confirm-dialog';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { taskActions } from '../../shared/attention';
import { ActionBar } from '../../shared/action-bar';
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

const IMAGE = /\.(png|jpe?g|gif|webp|svg)$/i;
const VIDEO = /\.(mp4|webm)$/i;

export function TaskPage({ spec, task, version }: { spec: string; task: string; version: number }) {
  const detail = useResource<TaskDetail>(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}`, version);
  const specDetail = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(spec)}`, version);
  const references = useMemo(() => (specDetail.kind === 'ready' ? buildReferences(specDetail.data.markdown, specDetail.data.tasks, spec) : new Map()), [specDetail, spec]);
  const { bind, popover } = useReferencePopover(references);
  const [editing, setEditing] = useState<{ file: string; title: string } | null>(null);
  const headings = useMemo<Heading[]>(() => {
    if (detail.kind !== 'ready') return [];
    const out: Heading[] = [];
    for (const part of HUMAN) out.push({ id: slug(part.title), text: part.title, depth: 2 });
    out.push({ id: 'evidencias', text: 'Evidências', depth: 2 });
    out.push({ id: 'para-o-agente', text: 'Para o agente', depth: 2 });
    for (const part of AGENT) out.push({ id: slug(part.title), text: part.title, depth: 3 });
    return out;
  }, [detail]);

  if (detail.kind === 'loading') return <LoadingState label={`Carregando ${task}`} rows={8} />;
  if (detail.kind === 'error') return <ErrorState title={`Não foi possível abrir ${task}`} description={detail.message} retryLabel="Tentar de novo" onRetry={detail.retry} />;

  const { state, files, evidence } = detail.data;
  const doneIds = new Set(specDetail.kind === 'ready' ? specDetail.data.tasks.filter((item) => item.status === 'done').map((item) => item.id) : []);
  const depsDone = listOf(state.depends_on ?? '').every((dep) => doneIds.has(dep));
  const editable = state.status !== 'done';
  const editButton = (file: string, title: string) =>
    editable ? <Button variant="ghost" className={styles.edit} onClick={() => setEditing({ file, title })}>Editar</Button> : null;
  const results: [string, string][] = [
    ['Testes', state.tests_result],
    ['Regressão', state.regression_result],
    ['Evidências', state.evidence_result],
  ];

  return (
    <div className={page.stack}>
      <PageHeader back={<BackLink fallback={`/specs/${spec}/quadro`} label={`Quadro de ${spec}`} />} eyebrow={`${spec} · ${state.id}`} title={state.title} subtitle={state.description?.replace(/`/g, "")} actions={
          <div className={styles.headerActions}>
            <Badge tone={statusTone(state.status)}>{statusLabel(state.status)}</Badge>
            <ActionBar items={taskActions(spec, state, depsDone)} />
          </div>
        } />
      <div className={page.docLayout}>
        <Toc headings={headings} label="Nesta tarefa" />
        <div className={page.docMain}>
          {HUMAN.map((part) => (
            <Card as="section" key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
              <div className={styles.partHeader}>
                <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
                {editButton(part.file, part.title)}
              </div>
              {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Ainda não escrito.</p>}
            </Card>
          ))}
          <Card as="section" aria-labelledby="evidencias" className={styles.part}>
            <h2 id="evidencias" className={styles.partTitle}>Evidências</h2>
            {evidence.length === 0 ? (
              <p className={styles.muted}>Elas aparecem aqui quando a tarefa é finalizada.</p>
            ) : (
              <Evidence
                files={evidence}
                review={{
                  prints: printNames(files.verificacao ?? ''),
                  registro: files.registro ?? '',
                  spec,
                  task: state.id,
                  reviewable: ['in_progress', 'validating'].includes(state.status),
                }}
              />
            )}
          </Card>
          <div className={styles.agentDivider} id="para-o-agente">
            <h2>Para o agente</h2>
            <p>Roteiro que o agente segue: arquivos, passos, comandos de verificação e o registro de cada etapa.</p>
          </div>
          {AGENT.map((part) => (
            <Card as="section" key={part.file} aria-labelledby={slug(part.title)} className={`${styles.part} ${styles.agentPart}`}>
              <div className={styles.partHeader}>
                <h3 id={slug(part.title)} className={styles.partTitle}>{part.title}</h3>
                {part.file === 'registro' ? null : editButton(part.file, part.title)}
              </div>
              {files[part.file] ? <Markdown source={stripTitle(files[part.file])} prefix={`${part.file}-`} references={references} bind={bind} /> : <p className={styles.muted}>Vazio.</p>}
            </Card>
          ))}
        </div>
        <aside className={page.docAside}>
          <dl className={page.meta}>
            <div><dt>Status</dt><dd>{statusLabel(state.status)}</dd></div>
            <div><dt>Onda</dt><dd>{state.wave}</dd></div>
            <div><dt>Depende de</dt><dd>{listOf(state.depends_on ?? '').map((dep) => <a key={dep} href={`/tarefas/${spec}/${dep}`} onClick={onLinkClick} className={styles.dep}>{dep}</a>)}{listOf(state.depends_on ?? '').length === 0 ? '—' : null}</dd></div>
            <div><dt>Cobre</dt><dd ref={bind} className={styles.refList}>{listOf(state.covers ?? '').map((id) => <button key={id} type="button" className="ref" data-ref={id} aria-haspopup="dialog">{id}</button>)}{listOf(state.covers ?? '').length === 0 ? '—' : null}</dd></div>
            <div><dt>Perfil</dt><dd>{state.profile}</dd></div>
            <div><dt>Persona</dt><dd>{state.persona}</dd></div>
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
      {editing ? <EditDialog path={`vostok/work/tasks/${spec}/${task}/${editing.file}.md`} title={`${editing.title} de ${task}`} onClose={() => setEditing(null)} /> : null}
    </div>
  );
}

function stripTitle(markdown: string): string {
  return markdown.replace(/^##+ .*\n/, '').replace(/^### /gm, '#### ');
}

/** Contexto da conferência dos prints da tarefa (RF-06). */
interface PrintReview {
  prints: string[];
  registro: string;
  spec: string;
  task: string;
  reviewable: boolean;
}

/** Os nomes dos itens `(print)` da seção Evidências. */
function printNames(verificacao: string): string[] {
  return [...verificacao.matchAll(/^- `([^`]+)` \(print\):/gm)].map((match) => match[1]);
}

/** O hash da última conferência do print no registro, ou `null` sem conferência. */
function reviewedHash(registro: string, name: string): string | null {
  const marker = `print ${name} conferido pelo usuário (sha256 `;
  const lines = registro.split('\n').filter((line) => line.includes(marker));
  const last = lines.at(-1);
  if (!last) return null;
  return last.slice(last.indexOf(marker) + marker.length).split(')')[0];
}

async function shortHash(url: string): Promise<string> {
  const bytes = await (await fetch(url)).arrayBuffer();
  const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', bytes));
  return Array.from(digest, (byte) => byte.toString(16).padStart(2, '0')).join('').slice(0, 8);
}

function Evidence({ files, review }: { files: FileRef[]; review: PrintReview }) {
  return (
    <ul className={styles.evidenceList}>
      {files.map((file) => <EvidenceItem key={file.path} file={file} review={review} />)}
    </ul>
  );
}

function EvidenceItem({ file, review }: { file: FileRef; review: PrintReview }) {
  const media = IMAGE.test(file.name) || VIDEO.test(file.name);
  const [open, setOpen] = useState(media);
  const [text, setText] = useState<string | null>(null);

  function toggle() {
    const next = !open;
    setOpen(next);
    if (next && !media && text === null) {
      fetch(fileUrl(file.path)).then((response) => response.text()).then(setText).catch(() => setText('Não foi possível ler o arquivo.'));
    }
  }

  return (
    <li className={styles.evidenceItem}>
      <button type="button" className={styles.evidenceButton} aria-expanded={open} onClick={toggle}>
        <span className={styles.evidenceName}>{file.name}</span>
        <span className={styles.evidenceSize}>{formatSize(file.size)}</span>
      </button>
      {open ? (
        IMAGE.test(file.name) ? (
          <img className={styles.evidenceMedia} src={fileUrl(file.path)} alt={`Evidência ${file.name}`} />
        ) : VIDEO.test(file.name) ? (
          <video className={styles.evidenceMedia} src={fileUrl(file.path)} controls />
        ) : (
          <pre className={styles.evidenceText}>{text ?? 'Carregando…'}</pre>
        )
      ) : null}
      {review.prints.includes(file.name) ? <PrintReviewRow file={file} review={review} /> : null}
    </li>
  );
}

type ReviewOutcome = { kind: 'idle' } | { kind: 'sending' } | { kind: 'done'; message: string } | { kind: 'failed'; message: string };

/** Selo de conferência do print e, na tarefa em execução, Aceitar e Rejeitar (rejeitar confirma antes). */
function PrintReviewRow({ file, review }: { file: FileRef; review: PrintReview }) {
  const [hash, setHash] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [outcome, setOutcome] = useState<ReviewOutcome>({ kind: 'idle' });
  useEffect(() => {
    shortHash(fileUrl(file.path)).then(setHash).catch(() => setHash(''));
  }, [file.path]);
  if (hash === null) return null;
  const reviewed = hash !== '' && reviewedHash(review.registro, file.name) === hash;

  async function send(accepted: boolean) {
    setConfirming(false);
    setOutcome({ kind: 'sending' });
    const query = `spec=${encodeURIComponent(review.spec)}&tarefa=${encodeURIComponent(review.task)}&print=${encodeURIComponent(file.name)}`;
    const result = await act('POST', `/api/conferencias?${query}&resultado=${accepted ? 'aceito' : 'rejeitado'}`);
    if (!result.ok) {
      setOutcome({ kind: 'failed', message: result.text });
      return;
    }
    setOutcome({ kind: 'done', message: accepted ? `Print ${file.name} conferido pelo painel.` : `Print ${file.name} rejeitado; a tarefa ficou bloqueada.` });
  }

  const sending = outcome.kind === 'sending';
  return (
    <div className={styles.review}>
      <Badge tone={reviewed ? 'success' : 'warning'}>{reviewed ? 'conferido' : 'a conferir'}</Badge>
      {review.reviewable && !reviewed && outcome.kind !== 'done' ? (
        <>
          <Button variant="primary" aria-label={'Aceitar ' + file.name} disabled={sending} onClick={() => void send(true)}>
            Aceitar
          </Button>
          <Button variant="secondary" aria-label={'Rejeitar ' + file.name} disabled={sending} onClick={() => setConfirming(true)}>
            Rejeitar
          </Button>
        </>
      ) : null}
      {outcome.kind === 'done' ? <p className={styles.reviewMessage} role="status">{outcome.message}</p> : null}
      {outcome.kind === 'failed' ? <p className={styles.reviewMessage} role="alert">{outcome.message}</p> : null}
      <ConfirmDialog
        open={confirming}
        title={`Rejeitar o print ${file.name}?`}
        description="A tarefa fica bloqueada até você decidir no painel, como numa falha da finalização."
        confirmLabel="Rejeitar print"
        cancelLabel="Cancelar"
        onConfirm={() => void send(false)}
        onCancel={() => setConfirming(false)}
      />
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1).replace('.', ',')} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1).replace('.', ',')} MB`;
}
