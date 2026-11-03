import { useState } from 'react';
import { act } from './api';
import { ACTION_LABELS, isPrimary, type ActionKind, type AttentionItem } from './attention';
import { Markdown } from './markdown';
import { Button } from '../ui/button/button';
import { Dialog } from '../ui/dialog/dialog';
import styles from './action-bar.module.css';

/** Ação que vira pedido ao agente, pelo tipo; aprovar spec e tarefa não está aqui: o painel aprova direto. */
const REQUEST_ACTION: Partial<Record<ActionKind, string>> = {
  'approve-plan': 'aprovar-plano',
  'plan-spec': 'planejar-spec',
  'adjust-spec': 'ajustar-spec',
  'adjust-task': 'ajustar-tarefa',
  'execute-task': 'executar-tarefa',
  'finish-task': 'finalizar-tarefa',
  'unblock-task': 'desbloquear-tarefa',
  'finalize-spec': 'finalizar-spec',
  'fazer-commit': 'fazer-commit',
  'create-spec': 'criar-spec',
  'refine-spec': 'refinar-spec',
  'execute-wave': 'executar-onda',
  'release-commits': 'liberar-commits',
  'undo-commits': 'desfazer-commits',
  answer: 'responder-pergunta',
};

/** Pedidos que não fazem sentido sem um texto do usuário. */
const NEEDS_TEXT: ActionKind[] = ['answer', 'adjust-spec', 'adjust-task', 'unblock-task', 'fazer-commit', 'create-spec'];

const TEXT_LABEL: Partial<Record<ActionKind, string>> = { 'unblock-task': 'Sua decisão', 'fazer-commit': 'O que commitar', 'create-spec': 'Assunto da spec' };

const COMMIT_LISTS: ActionKind[] = ['release-commits', 'undo-commits'];

type Outcome = { kind: 'idle' } | { kind: 'sending' } | { kind: 'done'; message: string } | { kind: 'failed'; message: string };

export async function performAction(item: AttentionItem, text: string): Promise<{ ok: boolean; message: string }> {
  const spec = encodeURIComponent(item.spec);
  const specParam = item.spec ? `&spec=${spec}` : '';
  if (item.kind === 'approve-spec' || item.kind === 'approve-task') {
    const tasks = item.kind === 'approve-task' ? (item.tasks ?? [item.task ?? '']).join(',') : '';
    const result = await act('POST', `/api/aprovacoes?spec=${spec}${tasks ? `&tarefas=${encodeURIComponent(tasks)}` : ''}`);
    if (!result.ok) return { ok: false, message: result.text };
    const approved = (JSON.parse(result.text) as { aprovadas: string[] }).aprovadas;
    return { ok: true, message: `Aprovado pelo painel: ${approved.join(', ')}. O aval ficou registrado em vostok/work/avais.log.` };
  }
  const action = REQUEST_ACTION[item.kind];
  if (!action) return { ok: false, message: 'Ação sem gravação no protótipo.' };
  const task = item.task ? `&tarefa=${encodeURIComponent(item.task)}` : '';
  const wave = item.wave ? '&onda=' + encodeURIComponent(item.wave) : '';
  const question = item.question ? '&pergunta=' + encodeURIComponent(item.question.meta.id) : '';
  // O pedido de liberar ou desfazer leva os commits apontados, para o aval valer para eles.
  const body = COMMIT_LISTS.includes(item.kind) ? [item.title, text].filter(Boolean).join('\n\n') : text;
  const result = await act('POST', `/api/pedidos?acao=${action}${specParam}${task}${wave}${question}`, body);
  if (!result.ok) return { ok: false, message: result.text };
  const { id } = JSON.parse(result.text) as { id: string };
  return { ok: true, message: `Pedido ${id} gravado em vostok/work/pedidos/. O chat em loop o processa e marca o estado.` };
}

export function ActionDialog({ item, onClose }: { item: AttentionItem | null; onClose: () => void }) {
  const [answer, setAnswer] = useState('');
  const [outcome, setOutcome] = useState<Outcome>({ kind: 'idle' });
  if (!item) return null;
  const label = ACTION_LABELS[item.kind];
  const options = item.question ? optionsOf(item.question.markdown) : [];
  const missingText = NEEDS_TEXT.includes(item.kind) && !answer.trim();
  const close = () => {
    setAnswer('');
    setOutcome({ kind: 'idle' });
    onClose();
  };
  const confirm = async () => {
    setOutcome({ kind: 'sending' });
    const result = await performAction(item, answer.trim());
    setOutcome(result.ok ? { kind: 'done', message: result.message } : { kind: 'failed', message: result.message });
  };

  return (
    <Dialog
      open
      title={[label.button, item.task ?? item.spec].filter(Boolean).join(': ')}
      onClose={close}
      actions={
        outcome.kind === 'done' ? (
          <Button variant="primary" onClick={close}>Fechar</Button>
        ) : (
          <>
            <Button variant="ghost" onClick={close}>Fechar</Button>
            <Button variant="primary" disabled={missingText || outcome.kind === 'sending'} onClick={() => void confirm()}>
              {outcome.kind === 'sending' ? 'Enviando…' : label.button}
            </Button>
          </>
        )
      }
    >
      {outcome.kind === 'done' ? (
        <p className={styles.done} role="status">{outcome.message}</p>
      ) : item.question ? (
        <div className={styles.question}>
          <Markdown source={questionText(item.question.markdown)} />
          {options.length > 0 ? (
            <fieldset className={styles.options}>
              <legend>Opções do agente</legend>
              {options.map((option) => (
                <label key={option} className={styles.option}>
                  <input type="radio" name="opcao" value={option} onChange={() => setAnswer(option)} />
                  <span>{option}</span>
                </label>
              ))}
            </fieldset>
          ) : null}
          <label className={styles.free}>
            <span>Ou escreva a resposta</span>
            <textarea rows={3} value={answer} onChange={(event) => setAnswer(event.target.value)} />
          </label>
        </div>
      ) : NEEDS_TEXT.includes(item.kind) ? (
        <>
          {item.title ? <p className={styles.dialogTitle}>{item.title}</p> : null}
          <label className={styles.free}>
            <span>{TEXT_LABEL[item.kind] ?? 'O que ajustar'}</span>
            <textarea rows={4} value={answer} onChange={(event) => setAnswer(event.target.value)} />
          </label>
        </>
      ) : (
        <>
          <p className={styles.dialogTitle}>{item.title}</p>
          {REQUEST_ACTION[item.kind] ? (
            <label className={styles.free}>
              <span>Observação para o agente (opcional)</span>
              <textarea rows={2} value={answer} onChange={(event) => setAnswer(event.target.value)} />
            </label>
          ) : null}
        </>
      )}
      {outcome.kind === 'done' ? null : <p className={styles.how}>{label.how}</p>}
      {outcome.kind === 'failed' ? <p className={styles.failed} role="alert">{outcome.message}</p> : null}
    </Dialog>
  );
}

function section(markdown: string, title: string): string {
  const match = new RegExp(`## ${title}\\n([\\s\\S]*?)(?=\\n## |$)`).exec(markdown);
  return match ? match[1].trim() : '';
}

function questionText(markdown: string): string {
  return section(markdown, 'Pergunta') || markdown;
}

function optionsOf(markdown: string): string[] {
  return section(markdown, 'Opções')
    .split('\n')
    .map((line) => line.replace(/^[-*]\s+/, '').trim())
    .filter(Boolean);
}

/** Barra de ações de uma spec ou tarefa: o primeiro botão em destaque, os demais secundários. */
export function ActionBar({ items }: { items: AttentionItem[] }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  if (items.length === 0) return null;
  return (
    <div className={styles.actionBar}>
      {items.map((item) => (
        <Button key={item.key} variant={isPrimary(item.kind) ? 'primary' : 'secondary'} onClick={() => setSelected(item)}>
          {ACTION_LABELS[item.kind].button}
        </Button>
      ))}
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </div>
  );
}
