import { useState } from 'react';
import { ActionDialog, CommitList, performAction } from '../../shared/action-bar';
import { useResource, type Overview, type UnreadAnswer } from '../../shared/api';
import { ACTION_LABELS, attentionItems, type AttentionItem } from '../../shared/attention';
import { parseQuestionForm, QuestionForm } from '../../shared/question-form';
import { navigate, onLinkClick } from '../../shared/router';
import { Badge } from '../../ui/badge/badge';
import { Button } from '../../ui/button/button';
import { ConfirmDialog } from '../../ui/confirm-dialog/confirm-dialog';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import styles from './inbox-page.module.css';

const TAG_TONE = { Aprovação: 'primary', 'Pedido ao agente': 'info', Decisão: 'warning', Pergunta: 'accent', Commit: 'info' } as const;

type UndoOutcome = { kind: 'idle' } | { kind: 'sending' } | { kind: 'done'; message: string } | { kind: 'failed'; message: string };

/** Commits apontados pelo `vostok commit check`, com os pedidos de liberar e desfazer (RF-07). */
function PendingCommits({ version }: { version: number }) {
  const commits = useResource<{ linhas?: string[] }>('/api/commits', version);
  const [releasing, setReleasing] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [undo, setUndo] = useState<UndoOutcome>({ kind: 'idle' });
  const lines = commits.kind === 'ready' ? (commits.data.linhas ?? []) : [];
  if (lines.length === 0) return null;
  const text = lines.join('\n');
  const release: AttentionItem = { key: 'release', kind: 'release-commits', spec: '', title: text, reason: '', href: '/pendencias' };

  async function confirmUndo() {
    setConfirming(false);
    setUndo({ kind: 'sending' });
    const result = await performAction({ key: 'undo', kind: 'undo-commits', spec: '', title: text, reason: '', href: '/pendencias' }, '');
    setUndo(result.ok ? { kind: 'done', message: result.message } : { kind: 'failed', message: result.message });
  }

  return (
    <section className={styles.commits} aria-labelledby="commits-sem-liberacao">
      <h2 id="commits-sem-liberacao" className={styles.commitsTitle}>Commits sem liberação</h2>
      <p className={styles.reason}>Commits feitos sem liberação, apontados pela verificação de commits.</p>
      <CommitList lines={lines} />
      <div className={styles.commitActions}>
        <Button variant="primary" onClick={() => setReleasing(true)}>{ACTION_LABELS['release-commits'].button}</Button>
        <Button variant="secondary" disabled={undo.kind === 'sending'} onClick={() => setConfirming(true)}>{ACTION_LABELS['undo-commits'].button}</Button>
      </div>
      {undo.kind === 'done' ? <p className={styles.reason} role="status">{undo.message}</p> : null}
      {undo.kind === 'failed' ? <p className={styles.reason} role="alert">{undo.message}</p> : null}
      <ActionDialog item={releasing ? release : null} onClose={() => setReleasing(false)} />
      <ConfirmDialog
        open={confirming}
        title="Desfazer os commits apontados?"
        description="Os commits novos saem da branch atual; as alterações ficam na pasta de trabalho."
        confirmLabel={ACTION_LABELS['undo-commits'].button}
        cancelLabel="Cancelar"
        onConfirm={() => void confirmUndo()}
        onCancel={() => setConfirming(false)}
      />
    </section>
  );
}

export function InboxPage({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  if (overview.kind === 'loading') return <LoadingState label="Carregando as pendências" rows={4} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;
  const answers = overview.data.respostas_nao_lidas ?? [];
  return (
    <>
      <PageHeader
        eyebrow="Pendências"
        title="Aguardando você"
        subtitle="Aprovações, finalizações, bloqueios e perguntas do agente, em todas as specs abertas."
        actions={<RequestCommit />}
      />
      <PendingCommits version={version} />
      <div className={styles.groups}>
        <UnreadAnswers answers={answers} />
        <PendingList items={attentionItems(overview.data)} titles={new Map(overview.data.specs.map((item) => [item.meta.id, item.meta.title]))} hideEmpty={answers.length > 0} />
      </div>
    </>
  );
}

/** Um item por resposta não lida: o aviso fica até o usuário abrir a pergunta (RF-17). */
function UnreadAnswers({ answers }: { answers: UnreadAnswer[] }) {
  if (answers.length === 0) return null;
  return (
    <section className={styles.group} aria-labelledby="respostas-nao-lidas">
      <h2 id="respostas-nao-lidas" className={styles.groupTitle}>Respostas às suas perguntas</h2>
      <ul className={styles.list}>
        {answers.map((answer) => (
          <li key={answer.id} className={styles.item} data-kind="answer">
            <div className={styles.text}>
              <div className={styles.meta}>
                <Badge tone="accent">Resposta</Badge>
                {answer.spec ? (
                  <a href={`/specs/${answer.spec}`} onClick={onLinkClick} className={styles.ref}>{answer.spec}</a>
                ) : (
                  <span className={styles.ref}>Sem spec</span>
                )}
              </div>
              <p className={styles.title}>{answer.texto.split('\n')[0]}</p>
              <p className={styles.reason}>O agente terminou. O aviso fica aqui até você abrir.</p>
            </div>
            <Button variant="secondary" aria-label={`Abrir a resposta de ${answer.id}`} onClick={() => navigate(`/perguntas#${answer.id}`)}>
              Abrir resposta
            </Button>
          </li>
        ))}
      </ul>
    </section>
  );
}

const COMMIT_REQUEST: AttentionItem = { key: 'commit', kind: 'fazer-commit', spec: '', title: '', reason: '', href: '/pendencias' };

function RequestCommit() {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button variant="secondary" onClick={() => setOpen(true)}>
        {ACTION_LABELS['fazer-commit'].button}
      </Button>
      <ActionDialog item={open ? COMMIT_REQUEST : null} onClose={() => setOpen(false)} />
    </>
  );
}

const EFFORT_LABELS = { decisao: 'Decisão', resposta: 'Resposta', conferencia: 'Conferência', 'teste-manual': 'Teste manual' } as const;

function buttonLabel(item: AttentionItem): string {
  if (item.kind === 'approve-task' && (item.tasks?.length ?? 0) > 1) return `Aprovar ${item.tasks!.length} tarefas`;
  return ACTION_LABELS[item.kind].button;
}

function bySpec(items: AttentionItem[]): { spec: string; items: AttentionItem[] }[] {
  const groups: { spec: string; items: AttentionItem[] }[] = [];
  for (const item of items) {
    const group = groups.find((entry) => entry.spec === item.spec);
    if (group) group.items.push(item);
    else groups.push({ spec: item.spec, items: [item] });
  }
  return groups;
}

function sectionOf(markdown: string, title: string): string {
  const lines = markdown.split('\n');
  const start = lines.findIndex((line) => line.trim() === `## ${title}`);
  if (start < 0) return '';
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith('## '));
  return (end >= 0 ? rest.slice(0, end) : rest).join('\n').trim();
}

function PendingList({ items, titles, hideEmpty = false }: { items: AttentionItem[]; titles: Map<string, string>; hideEmpty?: boolean }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  if (items.length === 0) {
    if (hideEmpty) return null;
    return <EmptyState title="Nada esperando você" description="Aprovações, finalizações, bloqueios e perguntas do agente aparecem aqui assim que surgirem." />;
  }
  return (
    <>
      {bySpec(items).map((group) => (
        <section key={group.spec} className={styles.group} aria-labelledby={`fila-${group.spec}`}>
          <h2 id={`fila-${group.spec}`} className={styles.groupTitle}>
            {group.spec}
            {titles.get(group.spec) ? ` · ${titles.get(group.spec)}` : ''}
          </h2>
          <ul className={styles.list}>
            {group.items.map((item) => <PendingItem key={item.key} item={item} onSelect={setSelected} />)}
          </ul>
        </section>
      ))}
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </>
  );
}

function PendingItem({ item, onSelect }: { item: AttentionItem; onSelect: (item: AttentionItem) => void }) {
  const label = ACTION_LABELS[item.kind];
  const { intro, questions } = item.question ? parseQuestionForm(item.question.markdown) : { intro: '', questions: [] };
  const [open, setOpen] = useState(item.now === true);
  const [outcome, setOutcome] = useState<UndoOutcome>({ kind: 'idle' });
  const title = questions.length > 0 ? questions[0].title + (questions.length > 1 ? ` e mais ${questions.length - 1}` : '') : item.title;
  const variant = item.now ? 'primary' : 'secondary';

  async function submit(answer: string) {
    setOutcome({ kind: 'sending' });
    const result = await performAction(item, answer);
    setOutcome(result.ok ? { kind: 'done', message: result.message } : { kind: 'failed', message: result.message });
  }

  return (
    <li className={styles.item} data-kind={item.kind} data-effort={item.effort} data-after={item.after ? '' : undefined}>
      <div className={styles.text}>
        <div className={styles.meta}>
          {item.now ? <Badge tone="primary">Faça agora</Badge> : null}
          <Badge tone={TAG_TONE[label.tag as keyof typeof TAG_TONE]}>{item.effort ? EFFORT_LABELS[item.effort] : label.tag}</Badge>
          <a href={item.href} onClick={onLinkClick} className={styles.ref}>
            {item.spec}
            {item.task ? ` · ${item.task}` : ''}
          </a>
        </div>
        <p className={styles.title}>{title}</p>
        <p className={styles.reason}>{item.reason}</p>
        {item.after ? <p className={styles.after}>{`Depois de: ${item.after}`}</p> : null}
        {item.href.endsWith('#evidencias') ? <a href={item.href} onClick={onLinkClick} className={styles.ref}>Abrir as evidências</a> : null}
      </div>
      {item.after ? null : questions.length > 0 ? (
        <Button variant={variant} aria-expanded={open} onClick={() => setOpen(!open)}>
          {open ? 'Recolher' : 'Responder'}
        </Button>
      ) : (
        <Button variant={variant} onClick={() => onSelect(item)}>
          {buttonLabel(item)}
        </Button>
      )}
      {questions.length > 0 && open && item.question ? (
        <div className={styles.answer}>
          {outcome.kind === 'done' ? (
            <p className={styles.reason} role="status">{outcome.message}</p>
          ) : (
            <>
              <QuestionForm intro={intro} questions={questions} notes={sectionOf(item.question.markdown, 'Inferências')} sending={outcome.kind === 'sending'} onSubmit={(answer) => void submit(answer)} />
              {outcome.kind === 'failed' ? <p className={styles.reason} role="alert">{outcome.message}</p> : null}
            </>
          )}
        </div>
      ) : null}
    </li>
  );
}
