import { useEffect, useMemo, useRef, useState } from 'react';
import { act, fileUrl, formatDate, listOf, readTextPreview, uploadEvidence, useResource, type FileRef, type ManualEvidence, type Overview, type SpecDetail, type TaskDetail } from '../../shared/api';
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
import { ProgressBar } from '../../ui/progress-bar/progress-bar';
import { ConfirmDialog } from '../../ui/confirm-dialog/confirm-dialog';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { activeRequest, taskActions } from '../../shared/attention';
import { ActionBar } from '../../shared/action-bar';
import { RequestState } from '../../shared/request-state';
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

const IMAGE = /\.(png|jpe?g|gif|webp)$/i;
const VIDEO = /\.(mp4|webm|mov)$/i;
const TEXT = /\.(md|txt|yml|log|out|diff|csv|json|css|js)$/i;

export function TaskPage({ spec, task, version }: { spec: string; task: string; version: number }) {
  const detail = useResource<TaskDetail>(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}`, version);
  const specDetail = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(spec)}`, version);
  const overview = useResource<Overview>('/api/visao', version);
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

  const ready = detail.kind === 'ready';
  useEffect(() => {
    if (!ready || window.location.hash !== '#evidencias') return;
    const heading = document.getElementById('evidencias');
    heading?.scrollIntoView({ block: 'start' });
    heading?.focus({ preventScroll: true });
  }, [ready]);

  if (detail.kind === 'loading') return <LoadingState label={`Carregando ${task}`} rows={8} />;
  if (detail.kind === 'error') return <ErrorState title={`Não foi possível abrir ${task}`} description={detail.message} retryLabel="Tentar de novo" onRetry={detail.retry} />;

  const { state, files, evidence, manual_evidence: manual, attach_limit_mb: limitMb } = detail.data;
  const manualNames = new Set(manual.map((item) => item.name));
  const otherEvidence = evidence.filter((file) => !manualNames.has(file.name));
  const review: PrintReview = {
    prints: printNames(files.verificacao ?? ''),
    registro: files.registro ?? '',
    spec,
    task: state.id,
    reviewable: ['in_progress', 'validating'].includes(state.status),
  };
  const requests = overview.kind === 'ready' ? (overview.data.requests ?? []) : [];
  const request = activeRequest(requests, spec, state.id);
  const outcome = detail.data.last_outcome ?? null;
  const waitingAnswer = !request && outcome?.desfecho === 'aguardando-pergunta';
  const doneIds = new Set(specDetail.kind === 'ready' ? specDetail.data.tasks.filter((item) => item.status === 'done').map((item) => item.id) : []);
  const depsDone = listOf(state.depends_on ?? '').every((dep) => doneIds.has(dep));
  const waitingOn = listOf(state.waiting_on ?? '');
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
            <ActionBar items={waitingAnswer ? [] : taskActions(spec, state, depsDone, requests)} />
          </div>
        } />
      <RequestState spec={spec} task={state.id} request={request} outcome={outcome} queued={requests.filter((item) => item.spec === spec && item.tarefa === state.id).length} />
      {waitingOn.length > 0 ? (
        <Card as="section" tone="warning" aria-label="Espera" className={styles.part}>
          <p className={styles.partTitle}>
            Espera{' '}
            {waitingOn.map((dep, index) => (
              <span key={dep}>
                {index > 0 ? ', ' : null}
                <a href={`/tarefas/${spec}/${dep}`} onClick={onLinkClick} className={styles.dep}>{dep}</a>
              </span>
            ))}
          </p>
          {state.waiting_reason ? <p className={styles.muted}>{state.waiting_reason}</p> : null}
          <p className={styles.muted}>Ela volta sozinha para Prontas para execução quando a espera acabar.</p>
        </Card>
      ) : null}
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
            <h2 id="evidencias" tabIndex={-1} className={styles.partTitle}>Evidências</h2>
            {manual.length === 0 && otherEvidence.length === 0 ? <p className={styles.muted}>Elas aparecem aqui quando a tarefa é finalizada.</p> : null}
            {manual.length > 0 ? (
              <ul aria-label="Evidências que o harness pede" className={styles.manualList}>
                {manual.map((item) => (
                  <ManualEvidenceItem key={item.name} item={item} spec={spec} task={state.id} limitMb={limitMb} attachable={['in_progress', 'validating'].includes(state.status)} review={review} onAttached={detail.retry} />
                ))}
              </ul>
            ) : null}
            {otherEvidence.length > 0 ? <Evidence files={otherEvidence} review={review} /> : null}
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
            {state.wave ? <div><dt>Onda</dt><dd>{state.wave}</dd></div> : null}
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

interface ManualEvidenceItemProps {
  item: ManualEvidence;
  spec: string;
  task: string;
  limitMb: number;
  attachable: boolean;
  review: PrintReview;
  onAttached: () => void;
}

type AttachOutcome = { kind: 'idle' } | { kind: 'sending'; percent: number } | { kind: 'done'; message: string } | { kind: 'failed'; message: string };

/** Evidência manual pedida pelo harness, com o que ela deve provar, Anexar ou Trocar e o progresso do envio. */
function ManualEvidenceItem({ item, spec, task, limitMb, attachable, review, onAttached }: ManualEvidenceItemProps) {
  const input = useRef<HTMLInputElement>(null);
  const [outcome, setOutcome] = useState<AttachOutcome>({ kind: 'idle' });
  const [revision, setRevision] = useState(0);
  const sending = outcome.kind === 'sending';
  const accept = item.extension === '' ? undefined : /^jpe?g$/i.test(item.extension) ? '.jpg,.jpeg' : `.${item.extension}`;

  function pick() {
    if (sending) return;
    input.current?.click();
  }

  async function chosen(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.currentTarget.files?.[0];
    event.currentTarget.value = '';
    if (!file || sending) return;
    if (file.size > limitMb * 1024 * 1024) {
      setOutcome({ kind: 'failed', message: `O arquivo tem ${formatSize(file.size)}, e o limite é de ${String(limitMb)} MB. Nada foi enviado.` });
      return;
    }
    setOutcome({ kind: 'sending', percent: 0 });
    const result = await uploadEvidence(spec, task, item.name, file, (percent) => setOutcome({ kind: 'sending', percent }));
    if (!result.ok) {
      setOutcome({ kind: 'failed', message: result.status === 0 ? 'A conexão caiu antes de o arquivo chegar inteiro. Nada mudou; tente de novo.' : result.text });
      return;
    }
    setRevision((value) => value + 1);
    setOutcome({ kind: 'done', message: `${file.name} anexado como ${item.name}.` });
    onAttached();
  }

  return (
    <li className={styles.manualItem}>
      <div className={styles.manualHead}>
        <div className={styles.manualText}>
          <span className={styles.evidenceName}>{item.name}</span>
          <p className={styles.manualAsk}>{item.description}</p>
        </div>
        <Badge tone={item.attached ? 'success' : 'warning'}>{item.attached ? 'anexado' : 'falta anexar'}</Badge>
        {attachable ? (
          <>
            <Button variant={item.attached ? 'secondary' : 'primary'} aria-label={`${item.attached ? 'Trocar' : 'Anexar'} ${item.name}`} onClick={pick}>
              {item.attached ? 'Trocar' : 'Anexar'}
            </Button>
            <input ref={input} type="file" hidden aria-label={`Arquivo para ${item.name}`} accept={accept} onChange={(event) => void chosen(event)} />
          </>
        ) : null}
      </div>
      {outcome.kind === 'sending' ? (
        <div className={styles.manualProgress}>
          <ProgressBar label={`Enviando ${item.name}`} value={outcome.percent} />
          <span role="status" className={styles.reviewMessage}>{`Enviando… ${String(outcome.percent)}%`}</span>
        </div>
      ) : null}
      {outcome.kind === 'done' ? <p className={styles.reviewMessage} role="status">{outcome.message}</p> : null}
      {outcome.kind === 'failed' ? <p className={styles.reviewMessage} role="alert">{outcome.message}</p> : null}
      {item.attached ? (
        <ul className={styles.evidenceList}>
          <EvidenceItem file={item} review={review} revision={revision} />
        </ul>
      ) : null}
    </li>
  );
}

function EvidenceItem({ file, review, revision = 0 }: { file: FileRef; review: PrintReview; revision?: number }) {
  const mediaUrl = revision > 0 ? `${fileUrl(file.path)}?v=${String(revision)}` : fileUrl(file.path);
  const media = IMAGE.test(file.name) || VIDEO.test(file.name);
  const [open, setOpen] = useState(media);
  const [text, setText] = useState<string | null>(null);
  const [total, setTotal] = useState<number | null>(null);

  if (!media && !TEXT.test(file.name)) {
    return (
      <li className={styles.evidenceItem}>
        <div className={styles.evidenceDownload}>
          <span className={styles.evidenceName}>{file.name}</span>
          <span className={styles.evidenceSize}>{formatSize(file.size)}</span>
          <a className={styles.downloadLink} href={fileUrl(file.path)} download={file.name} aria-label={`Baixar ${file.name}`}>Baixar</a>
        </div>
        {review.prints.includes(file.name) ? <PrintReviewRow file={file} review={review} /> : null}
      </li>
    );
  }

  function toggle() {
    const next = !open;
    setOpen(next);
    if (!next || media || text !== null) return;
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

  return (
    <li className={styles.evidenceItem}>
      <button type="button" className={styles.evidenceButton} aria-expanded={open} onClick={toggle}>
        <span className={styles.evidenceName}>{file.name}</span>
        <span className={styles.evidenceSize}>{formatSize(file.size)}</span>
      </button>
      {open ? (
        IMAGE.test(file.name) ? (
          <img className={styles.evidenceMedia} src={mediaUrl} alt={`Evidência ${file.name}`} />
        ) : VIDEO.test(file.name) ? (
          <video className={styles.evidenceMedia} src={mediaUrl} controls />
        ) : (
          <>
            <pre className={styles.evidenceText}>{text ?? 'Carregando…'}</pre>
            {total !== null ? (
              <p className={styles.reviewMessage} role="status">
                Mostrando o primeiro 1 MB de {formatSize(total)}. <a className={styles.downloadLink} href={fileUrl(file.path)} download={file.name}>Baixar o arquivo inteiro</a>
              </p>
            ) : null}
          </>
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
