import { useEffect, useMemo, useRef, useState } from 'react';
import { act, formatDateTime, uploadAttachment, useResource, type AttachmentLimits, type FreeQuestion, type Overview } from '../../shared/api';
import { Markdown } from '../../shared/markdown';
import { onLinkClick } from '../../shared/router';
import { Badge } from '../../ui/badge/badge';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { SelectField } from '../../ui/select-field/select-field';
import { formatSize, QuestionComposer, type PendingAttachment } from './question-composer';
import styles from './ask-page.module.css';

const STATE_BADGE = {
  pendente: { label: 'Na fila', tone: 'info' },
  em_andamento: { label: 'Respondendo', tone: 'accent' },
  feito: { label: 'Respondida', tone: 'success' },
  recusado: { label: 'Recusada', tone: 'warning' },
} as const;

const isConcluded = (question: FreeQuestion) => question.estado === 'feito' || question.estado === 'recusado';
const firstLine = (text: string) => text.split('\n').find((line) => line.trim() !== '')?.trim() ?? '';

const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'webp'];
const isImageName = (name: string) => IMAGE_EXTENSIONS.includes(name.split('.').pop()?.toLowerCase() ?? '');
const fileUrl = (path: string) => `/arquivos/${path.split('/').map(encodeURIComponent).join('/')}`;
const withoutFinalDot = (text: string) => text.replace(/\.+$/, '');

function uploadFailure(name: string, status: number, text: string): string {
  if (status === 0) return `A conexão caiu durante o envio de ${name}. Nada entrou na fila: tente de novo.`;
  if (status === 403) return 'A sessão do painel mudou. Copie o texto e recarregue a página.';
  return `Não foi possível enviar ${name}: ${withoutFinalDot(text)}. Nada entrou na fila.`;
}

type SendOutcome = { kind: 'idle' } | { kind: 'sent'; id: string };

function AskForm({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  const limits = useResource<AttachmentLimits>('/api/pedidos/anexos/limites', version);
  const [spec, setSpec] = useState('');
  const [outcome, setOutcome] = useState<SendOutcome>({ kind: 'idle' });
  const options = useMemo(
    () => [{ value: '', label: 'Sem spec' }, ...(overview.kind === 'ready' ? overview.data.specs.map((item) => ({ value: item.meta.id, label: `${item.meta.id} · ${item.meta.title}` })) : [])],
    [overview],
  );

  async function submit(text: string, attachments: PendingAttachment[], report: (id: string, percent: number) => void): Promise<string | null> {
    const base = `/api/pedidos?acao=perguntar-agente${spec ? `&spec=${encodeURIComponent(spec)}` : ''}`;
    let draftQuery = '';
    if (attachments.length > 0) {
      const draft = crypto.randomUUID();
      for (const item of attachments) {
        report(item.id, 0);
        const sent = await uploadAttachment(draft, item.file, item.name, (percent) => report(item.id, percent));
        if (!sent.ok) return uploadFailure(item.name, sent.status, sent.text);
      }
      draftQuery = `&rascunho=${encodeURIComponent(draft)}`;
    }
    const result = await act('POST', base + draftQuery, text.trim());
    if (!result.ok) return attachments.length > 0 ? `Não foi possível criar a pergunta: ${withoutFinalDot(result.text)}.` : result.text || 'Não foi possível enviar a pergunta.';
    let id = '';
    try {
      id = (JSON.parse(result.text) as { id?: string }).id ?? '';
    } catch {
      id = '';
    }
    setOutcome({ kind: 'sent', id });
    return null;
  }

  return (
    <div className={styles.form}>
      <SelectField label="Spec (opcional)" options={options} value={spec} onChange={setSpec} className={styles.spec} />
      <QuestionComposer limits={limits.kind === 'ready' ? limits.data : null} onSubmit={submit} />
      {outcome.kind === 'sent' ? <p className={styles.sent} role="status">Pergunta enviada: {outcome.id}.</p> : null}
    </div>
  );
}

function AttachmentSection({ attachments }: { attachments: FreeQuestion['anexos'] }) {
  if (attachments.length === 0) return null;
  return (
    <>
      <h3>Anexos</h3>
      <ul className={styles.attachments}>
        {attachments.map((attachment) => (
          <li key={attachment.caminho} className={styles.attachment}>
            {isImageName(attachment.nome) ? <img src={fileUrl(attachment.caminho)} alt={attachment.nome} loading="lazy" className={styles.attachmentImage} /> : null}
            <span className={styles.attachmentInfo}>
              {isImageName(attachment.nome) ? <span className={styles.attachmentName}>{attachment.nome}</span> : <a href={fileUrl(attachment.caminho)} download className={styles.attachmentName}>{attachment.nome}</a>}
              <span className={styles.attachmentSize}>{formatSize(attachment.tamanho)}</span>
            </span>
          </li>
        ))}
      </ul>
    </>
  );
}

function QuestionItem({ question, open, onToggle }: { question: FreeQuestion; open: boolean; onToggle: () => void }) {
  const badge = STATE_BADGE[question.estado];
  const unread = isConcluded(question) && !question.lida_em;
  const panel = `resposta-${question.id}`;
  return (
    <li className={styles.item}>
      <button type="button" id={`pergunta-${question.id}`} className={styles.toggle} aria-expanded={open} aria-controls={panel} onClick={onToggle}>
        <span className={styles.title}>{firstLine(question.texto)}</span>
        <span className={styles.badges}>
          {unread ? <Badge tone="primary">Não lida</Badge> : null}
          <Badge tone={badge.tone}>{badge.label}</Badge>
        </span>
      </button>
      <p className={styles.meta}>
        <span>{question.id}</span>
        <span>{formatDateTime(question.atualizado_em)}</span>
        {question.spec ? <a href={`/specs/${question.spec}`} onClick={onLinkClick}>{question.spec}</a> : null}
      </p>
      <div id={panel} className={styles.panel} hidden={!open}>
        {open ? (
          <>
            <h3>Pergunta</h3>
            <Markdown source={question.texto} untrusted />
            <AttachmentSection attachments={question.anexos ?? []} />
            {isConcluded(question) ? (
              <>
                <h3>{question.estado === 'recusado' ? 'Motivo da recusa' : 'Resposta'}</h3>
                <Markdown source={question.resultado || 'O agente não deixou texto.'} untrusted />
              </>
            ) : (
              <p className={styles.waiting}>O agente ainda não respondeu.</p>
            )}
          </>
        ) : null}
      </div>
    </li>
  );
}

function QuestionList({ questions }: { questions: FreeQuestion[] }) {
  const [opened, setOpened] = useState<Set<string>>(new Set());
  const [warning, setWarning] = useState('');
  const marked = useRef(new Set<string>());
  const [anchor, setAnchor] = useState(() => window.location.hash.slice(1));

  useEffect(() => {
    const update = () => setAnchor(window.location.hash.slice(1));
    window.addEventListener('popstate', update);
    return () => window.removeEventListener('popstate', update);
  }, []);

  function markRead(question: FreeQuestion) {
    if (!isConcluded(question) || question.lida_em || marked.current.has(question.id)) return;
    marked.current.add(question.id);
    void act('POST', `/api/pedidos/${encodeURIComponent(question.id)}/lida`).then((result) => {
      if (result.ok) return;
      marked.current.delete(question.id);
      setWarning('Não foi possível marcar como lida; o aviso continua.');
    });
  }

  function toggle(question: FreeQuestion) {
    const isOpen = opened.has(question.id);
    setOpened((current) => {
      const next = new Set(current);
      if (isOpen) next.delete(question.id);
      else next.add(question.id);
      return next;
    });
    if (!isOpen) markRead(question);
  }

  useEffect(() => {
    if (!anchor) return;
    const target = questions.find((question) => question.id === anchor);
    if (!target) return;
    setOpened((current) => new Set(current).add(target.id));
    markRead(target);
    document.getElementById(`pergunta-${target.id}`)?.focus();
    setAnchor('');
    // eslint-disable-next-line react-hooks/exhaustive-deps -- só reage ao endereço e à lista
  }, [anchor, questions]);

  return (
    <>
      {warning ? <p className={styles.failed} role="alert">{warning}</p> : null}
      <ul className={styles.list}>
        {questions.map((question) => (
          <QuestionItem key={question.id} question={question} open={opened.has(question.id)} onToggle={() => toggle(question)} />
        ))}
      </ul>
    </>
  );
}

export function AskPage({ version }: { version: number }) {
  const questions = useResource<FreeQuestion[]>('/api/perguntas-livres', version);
  return (
    <>
      <PageHeader eyebrow="Perguntas" title="Perguntas ao agente" subtitle="Tire uma dúvida com o agente, sem mudar nada no projeto. A resposta aparece aqui." />
      <AskForm version={version} />
      <section className={styles.sentList} aria-labelledby="perguntas-enviadas">
        <h2 id="perguntas-enviadas" className={styles.heading}>Perguntas enviadas</h2>
        {questions.kind === 'loading' ? <LoadingState label="Carregando as perguntas" rows={3} /> : null}
        {questions.kind === 'error' ? <ErrorState title="Não foi possível ler as perguntas" description={questions.message} retryLabel="Tentar de novo" onRetry={questions.retry} /> : null}
        {questions.kind === 'ready' && questions.data.length === 0 ? (
          <EmptyState title="Nenhuma pergunta ainda" description="As perguntas que você enviar aparecem aqui, com a resposta do agente." />
        ) : null}
        {questions.kind === 'ready' && questions.data.length > 0 ? <QuestionList questions={questions.data} /> : null}
      </section>
    </>
  );
}
