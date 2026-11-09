import { useEffect, useMemo, useState } from 'react';
import { fileUrl, formatDate, listOf, readTextPreview, useResource, type EvidenceReview, type FileRef, type ManualEvidence, type ReviewStatus, type SpecDetail, type TaskDetail } from '../../shared/api';
import { BackLink } from '../../shared/back-link';
import { buildReferences } from '../../shared/references';
import { useReferencePopover } from '../../shared/ref-popover';
import { Markdown, slug, type Heading } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { statusLabel, statusTone } from '../../shared/status';
import { Toc } from '../../shared/toc';
import { Badge, type BadgeTone } from '../../ui/badge/badge';
import { Card } from '../../ui/card/card';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
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
  const references = useMemo(() => (specDetail.kind === 'ready' ? buildReferences(specDetail.data.markdown, specDetail.data.tasks, spec) : new Map()), [specDetail, spec]);
  const { bind, popover } = useReferencePopover(references);
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

  const { state, files, evidence, manual_evidence: manualList } = detail.data;
  const manual = manualList ?? [];
  const manualNames = new Set(manual.map((item) => item.name));
  const otherEvidence = evidence.filter((file) => !manualNames.has(file.name));
  const reviews = new Map((detail.data.reviews ?? []).map((item) => [item.name, item] as const));
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
          </div>
        } />
      <div className={page.docLayout}>
        <Toc headings={headings} label="Nesta tarefa" />
        <div className={page.docMain}>
          {HUMAN.map((part) => (
            <Card as="section" key={part.file} aria-labelledby={slug(part.title)} className={styles.part}>
              <div className={styles.partHeader}>
                <h2 id={slug(part.title)} className={styles.partTitle}>{part.title}</h2>
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
                  <ManualEvidenceItem key={item.name} item={item} reviews={reviews} />
                ))}
              </ul>
            ) : null}
            {otherEvidence.length > 0 ? <Evidence files={otherEvidence} reviews={reviews} /> : null}
          </Card>
          <div className={styles.agentDivider} id="para-o-agente">
            <h2>Para o agente</h2>
            <p>Roteiro que o agente segue: arquivos, passos, comandos de verificação e o registro de cada etapa.</p>
          </div>
          {AGENT.map((part) => (
            <Card as="section" key={part.file} aria-labelledby={slug(part.title)} className={`${styles.part} ${styles.agentPart}`}>
              <div className={styles.partHeader}>
                <h3 id={slug(part.title)} className={styles.partTitle}>{part.title}</h3>
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
    </div>
  );
}

function stripTitle(markdown: string): string {
  return markdown.replace(/^##+ .*\n/, '').replace(/^### /gm, '#### ');
}

function Evidence({ files, reviews }: { files: FileRef[]; reviews: Map<string, EvidenceReview> }) {
  return (
    <ul className={styles.evidenceList}>
      {files.map((file) => <EvidenceItem key={file.path} file={file} reviews={reviews} />)}
    </ul>
  );
}

interface ManualEvidenceItemProps {
  item: ManualEvidence;
  reviews: Map<string, EvidenceReview>;
}

/** Evidência manual pedida pelo harness, com o que ela deve provar e o selo de anexada; o painel só mostra. */
function ManualEvidenceItem({ item, reviews }: ManualEvidenceItemProps) {
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
          <EvidenceItem file={item} reviews={reviews} />
        </ul>
      ) : null}
    </li>
  );
}

function EvidenceItem({ file, reviews }: { file: FileRef; reviews: Map<string, EvidenceReview> }) {
  const mediaUrl = fileUrl(file.path);
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
        {reviews.get(file.name) ? <ReviewRow review={reviews.get(file.name)!} /> : null}
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
      {reviews.get(file.name) ? <ReviewRow review={reviews.get(file.name)!} /> : null}
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
