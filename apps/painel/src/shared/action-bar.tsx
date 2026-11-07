import { createContext, useContext, useEffect, useId, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';
import { act, useResource, type Overview, type SpecDetail } from './api';
import { ACTION_LABELS, actionLabel, type ActionKind, type AttentionItem } from './attention';
import { Markdown } from './markdown';
import { parseQuestionForm, questionBody, QuestionForm } from './question-form';
import { statusLabel } from './status';
import { Button } from '../ui/button/button';
import { Checkbox } from '../ui/checkbox/checkbox';
import { Dialog } from '../ui/dialog/dialog';
import styles from './action-bar.module.css';

/** Ação que vira pedido ao agente, pelo tipo; aprovar spec e tarefa não está aqui: o painel aprova direto. */
const REQUEST_ACTION: Partial<Record<ActionKind, string>> = {
  'approve-plan': 'aprovar-plano',
  'plan-spec': 'planejar-spec',
  'redo-plan': 'planejar-spec',
  'adjust-plan': 'ajustar-plano',
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
const NEEDS_TEXT: ActionKind[] = ['answer', 'adjust-spec', 'adjust-plan', 'adjust-task', 'unblock-task', 'fazer-commit', 'create-spec'];

const TEXT_LABEL: Partial<Record<ActionKind, string>> = { 'unblock-task': 'Sua decisão', 'fazer-commit': 'O que commitar', 'create-spec': 'Assunto da spec' };

const COMMIT_LISTS: ActionKind[] = ['release-commits', 'undo-commits'];

type Outcome = { kind: 'idle' } | { kind: 'sending' } | { kind: 'failed'; message: string };

export interface Refusal {
  summary: string;
  groups: { task: string; lines: string[] }[];
  rest: string[];
}

const TASK_LINE = /^(TASK-\d+):\s*(.*)$/;

/** Separa a recusa da CLI em resumo, problemas por tarefa (prefixo `TASK-NN:`) e o resto. */
export function parseRefusal(message: string): Refusal {
  const lines = message.split('\n').map((line) => line.replace(/^[-*]\s+/, '').trim()).filter(Boolean);
  const groups: Refusal['groups'] = [];
  const rest: string[] = [];
  for (const line of lines) {
    const match = TASK_LINE.exec(line);
    if (!match) {
      rest.push(line);
      continue;
    }
    const group = groups.find((item) => item.task === match[1]);
    if (group) group.lines.push(match[2]);
    else groups.push({ task: match[1], lines: [match[2]] });
  }
  const summary = (rest.shift() ?? 'O painel recusou a ação.').replace(/:$/, '.');
  return { summary, groups, rest };
}

/** Versão ao vivo do painel, entregue às janelas de ação para acompanharem as mudanças. */
export const LiveVersion = createContext(0);

const REGISTER_TEXT = 'Só registrar a edição manual, sem outras mudanças. Não aprove.';
const MANUAL_EDIT = 'edição manual sem registro';

/** Troca a linha de recusa da CLI por uma frase simples; o motivo desconhecido segue como veio. */
function plainProblem(line: string, subject: string): string {
  if (line.includes(MANUAL_EDIT)) return `${subject} foi editada, e a edição ainda não foi registrada.`;
  const task = /^status '(\w+)'; só tarefas/.exec(line);
  if (task) return `Esta tarefa não está mais à espera de aprovação: agora está ${statusLabel(task[1])}.`;
  const spec = /^a spec (\S+) está com status '(\w+)'/.exec(line);
  if (spec) return `A spec ${spec[1]} está ${statusLabel(spec[2])}; só uma spec refinada pode ser aprovada.`;
  return line;
}

/** Itens da recusa com edição manual sem registro: uma tarefa por grupo, ou a própria spec (`task` vazio). */
function manualEditTargets(message: string): { task: string }[] {
  const { groups, rest } = parseRefusal(message);
  const targets = groups.filter((group) => group.lines.some((line) => line.includes(MANUAL_EDIT))).map((group) => ({ task: group.task }));
  if (rest.some((line) => line.includes(MANUAL_EDIT))) targets.push({ task: '' });
  return targets;
}

function RegisterEdits({ spec, targets, onDone }: { spec: string; targets: { task: string }[]; onDone: () => void }) {
  const version = useContext(LiveVersion);
  const overview = useResource<Overview>('/api/visao', version);
  const [sent, setSent] = useState<string[]>([]);
  const [failures, setFailures] = useState<string[]>([]);
  const [sending, setSending] = useState(false);
  if (targets.length === 0) return null;
  const requests = overview.kind === 'ready' ? overview.data.requests ?? [] : [];
  const active = (task: string) => requests.some((item) => item.acao === (task ? 'ajustar-tarefa' : 'ajustar-spec') && item.spec === spec && item.tarefa === task);
  const pending = targets.filter((target) => !active(target.task) && !sent.includes(target.task));
  const register = async () => {
    setSending(true);
    const done: string[] = [];
    const failed: string[] = [];
    for (const target of pending) {
      const query = target.task ? `acao=ajustar-tarefa&spec=${encodeURIComponent(spec)}&tarefa=${encodeURIComponent(target.task)}` : `acao=ajustar-spec&spec=${encodeURIComponent(spec)}`;
      const result = await act('POST', `/api/pedidos?${query}`, REGISTER_TEXT);
      if (result.ok) done.push(target.task);
      else failed.push(result.text);
    }
    setSent((current) => [...current, ...done]);
    setFailures(failed);
    setSending(false);
    if (failed.length === 0) onDone();
  };
  return (
    <div className={styles.register}>
      <Button variant="secondary" disabled={pending.length === 0 || sending} onClick={() => void register()}>
        Pedir o registro da edição
      </Button>
      {failures.length > 0 ? <p className={styles.registerFailed}>{failures.join('\n')}</p> : null}
    </div>
  );
}

function RefusalMessage({ message, spec, onDone }: { message: string; spec: string; onDone: () => void }) {
  const { summary, groups, rest } = parseRefusal(message);
  return (
    <div className={styles.failed} role="alert">
      <p className={styles.failedSummary}>{plainProblem(summary, `A spec ${spec}`)}</p>
      {groups.map((group) => (
        <div key={group.task}>
          <p className={styles.failedTask}>{group.task}</p>
          <ul>
            {group.lines.map((line, index) => <li key={index}>{plainProblem(line, 'Esta tarefa')}</li>)}
          </ul>
        </div>
      ))}
      {rest.length > 0 ? (
        <ul>
          {rest.map((line, index) => <li key={index}>{plainProblem(line, `A spec ${spec}`)}</li>)}
        </ul>
      ) : null}
      <RegisterEdits spec={spec} targets={manualEditTargets(message)} onDone={onDone} />
    </div>
  );
}

/** Endereços cuja mudança desfaz a recusa da janela: as tarefas ou a spec que ela cita. */
function watchUrls(item: AttentionItem): string[] {
  if (item.kind === 'approve-task') return (item.tasks ?? [item.task ?? '']).filter(Boolean).map((id) => `/api/tarefas/${encodeURIComponent(item.spec)}/${encodeURIComponent(id)}`);
  if (item.kind === 'approve-spec') return [`/api/specs/${encodeURIComponent(item.spec)}`];
  return [];
}

/** Não desenha nada: lê as URLs a cada versão ao vivo e avisa quando o conteúdo muda desde a primeira leitura. */
function RefusalWatch({ urls, onChange }: { urls: string[]; onChange: () => void }) {
  const version = useContext(LiveVersion);
  const first = useRef<string | null>(null);
  const key = urls.join('|');
  useEffect(() => {
    let active = true;
    void (async () => {
      try {
        const texts = await Promise.all(urls.map(async (url) => (await fetch(url)).text()));
        if (!active) return;
        const signature = texts.join('\u0000');
        if (first.current === null) first.current = signature;
        else if (first.current !== signature) onChange();
      } catch {
        // Falha de rede não muda nada.
      }
    })();
    return () => {
      active = false;
    };
  }, [version, key]);
  return null;
}

function TaskPicker({ spec, ids, chosen, onChange, onGone }: { spec: string; ids: string[]; chosen: string[]; onChange: (ids: string[]) => void; onGone: (ids: string[]) => void }) {
  const detail = useResource<SpecDetail>(`/api/specs/${encodeURIComponent(spec)}`, useContext(LiveVersion));
  const known = detail.kind === 'ready' && Array.isArray(detail.data.tasks) ? detail.data.tasks : [];
  useEffect(() => {
    const gone = known.filter((task) => task.status !== 'pending').map((task) => task.id);
    if (gone.length > 0) onGone(gone);
  }, [detail]);
  const toggle = (id: string, checked: boolean) => onChange(ids.filter((other) => (other === id ? checked : chosen.includes(other))));
  return (
    <fieldset className={styles.options}>
      <legend>Tarefas a aprovar</legend>
      {ids.map((id) => {
        const found = known.find((task) => task.id === id);
        return (
          <Checkbox
            key={id}
            label={found ? [id, found.title, found.wave ? `Onda ${found.wave}` : ''].filter(Boolean).join(' · ') : id}
            checked={chosen.includes(id)}
            onChange={(event) => toggle(id, event.currentTarget.checked)}
          />
        );
      })}
      {chosen.length === 0 ? <p className={styles.hint}>Marque ao menos uma tarefa para aprovar.</p> : null}
    </fieldset>
  );
}

const COMMIT_LINE = /^(?:- )?[0-9a-f]{7,40}\s/;
const COMMITS_SHOWN_OPEN = 10;

/** Linhas do `vostok commit check`; acima do limite, ficam recolhidas, com a contagem de commits, sem os títulos. */
export function CommitList({ lines }: { lines: string[] }) {
  const count = lines.filter((line) => COMMIT_LINE.test(line)).length;
  const list = <ul className={styles.commitLines}>{lines.map((line, index) => <li key={index}>{line}</li>)}</ul>;
  if (count <= COMMITS_SHOWN_OPEN) return list;
  return (
    <details className={styles.commitDetails}>
      <summary>{`${count} commits apontados`}</summary>
      {list}
    </details>
  );
}

export async function performAction(item: AttentionItem, text: string): Promise<{ ok: boolean; message: string }> {
  const spec = encodeURIComponent(item.spec);
  const specParam = item.spec ? `&spec=${spec}` : '';
  if (item.kind === 'approve-spec' || item.kind === 'approve-task') {
    const tasks = item.kind === 'approve-task' ? (item.tasks ?? [item.task ?? '']).join(',') : '';
    const result = await act('POST', `/api/aprovacoes?spec=${spec}${tasks ? `&tarefas=${encodeURIComponent(tasks)}` : ''}`);
    if (!result.ok) return { ok: false, message: result.text };
    const approved = (JSON.parse(result.text) as { aprovadas: string[] }).aprovadas;
    return { ok: true, message: `Aprovado: ${approved.join(', ')}. A aprovação ficou registrada.` };
  }
  const action = REQUEST_ACTION[item.kind];
  if (!action) return { ok: false, message: 'Esta ação ainda não está disponível no painel.' };
  const task = item.task ? `&tarefa=${encodeURIComponent(item.task)}` : '';
  const wave = item.wave ? '&onda=' + encodeURIComponent(item.wave) : '';
  const question = item.question ? '&pergunta=' + encodeURIComponent(item.question.meta.id) : '';
  // O pedido de liberar ou desfazer leva os commits apontados, para o aval valer para eles.
  const body = COMMIT_LISTS.includes(item.kind) ? [item.title, text].filter(Boolean).join('\n\n') : text;
  const result = await act('POST', `/api/pedidos?acao=${action}${specParam}${task}${wave}${question}`, body);
  if (!result.ok) return { ok: false, message: result.text };
  const { id } = JSON.parse(result.text) as { id: string };
  return { ok: true, message: `Pedido ${id} enviado ao agente. O andamento aparece no painel.` };
}

export function ActionDialog({ item, onClose }: { item: AttentionItem | null; onClose: () => void }) {
  const [answer, setAnswer] = useState('');
  const [chosen, setChosen] = useState<string[] | null>(null);
  const [outcome, setOutcome] = useState<Outcome>({ kind: 'idle' });
  const [gone, setGone] = useState<string[]>([]);
  if (!item) return null;
  const label = ACTION_LABELS[item.kind];
  const batch = item.kind === 'approve-task' && item.tasks !== undefined;
  const open = (item.tasks ?? []).filter((id) => !gone.includes(id));
  const selection = (chosen ?? open).filter((id) => open.includes(id));
  const target = batch ? { ...item, tasks: selection } : item;
  const missingTasks = batch && selection.length === 0;
  const form = item.question ? parseQuestionForm(item.question.markdown) : null;
  const useForm = (form?.questions.length ?? 0) > 0;
  const options = item.question ? optionsOf(item.question.markdown) : [];
  const missingText = NEEDS_TEXT.includes(item.kind) && !answer.trim();
  const close = () => {
    setAnswer('');
    setChosen(null);
    setGone([]);
    setOutcome({ kind: 'idle' });
    onClose();
  };
  const onGone = (ids: string[]) => {
    const all = [...new Set([...gone, ...ids.filter((id) => (item.tasks ?? []).includes(id))])];
    if (all.length === gone.length) return;
    setGone(all);
    if ((item.tasks ?? []).length > 0 && (item.tasks ?? []).every((id) => all.includes(id))) close();
  };
  const send = async (text: string) => {
    setOutcome({ kind: 'sending' });
    const result = await performAction(target, text);
    if (result.ok) close();
    else setOutcome({ kind: 'failed', message: result.message });
  };
  const confirm = () => send(answer.trim());

  return (
    <Dialog
      open
      title={[actionLabel(batch ? { ...item, tasks: open } : item), item.task ?? item.spec].filter(Boolean).join(': ')}
      onClose={close}
      size="content"
      actions={
        <>
          <Button variant="ghost" onClick={close}>Fechar</Button>
          {useForm ? null : <Button variant="primary" disabled={missingText || missingTasks || outcome.kind === 'sending'} onClick={() => void confirm()}>
            {outcome.kind === 'sending' ? 'Enviando…' : actionLabel(target)}
          </Button>}
        </>
      }
    >
      {item.question && useForm && form ? (
        <QuestionForm intro={form.intro} questions={form.questions} notes={section(item.question.markdown, 'Inferências')} sending={outcome.kind === 'sending'} onSubmit={(text) => void send(text)} />
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
          {batch ? <TaskPicker spec={item.spec} ids={open} chosen={selection} onChange={setChosen} onGone={onGone} /> : COMMIT_LISTS.includes(item.kind) ? <CommitList lines={item.title.split('\n')} /> : <p className={styles.dialogTitle}>{item.title}</p>}
          {REQUEST_ACTION[item.kind] ? (
            <label className={styles.free}>
              <span>Observação para o agente (opcional)</span>
              <textarea rows={2} value={answer} onChange={(event) => setAnswer(event.target.value)} />
            </label>
          ) : null}
        </>
      )}
      <p className={styles.how}>{label.how}</p>
      {outcome.kind === 'failed' ? (
        <>
          <RefusalMessage message={outcome.message} spec={item.spec} onDone={close} />
          <RefusalWatch urls={watchUrls(target)} onChange={() => setOutcome({ kind: 'idle' })} />
        </>
      ) : null}
    </Dialog>
  );
}

function section(markdown: string, title: string): string {
  const match = new RegExp(`## ${title}\\n([\\s\\S]*?)(?=\\n## |$)`).exec(markdown);
  return match ? match[1].trim() : '';
}

function questionText(markdown: string): string {
  return questionBody(markdown).trim() || markdown;
}

function optionsOf(markdown: string): string[] {
  return section(markdown, 'Opções')
    .split('\n')
    .map((line) => line.replace(/^[-*]\s+/, '').trim())
    .filter(Boolean);
}

/** Barra de ações de uma spec ou tarefa: o próximo passo em destaque e as demais em "Mais ações". */
export function ActionBar({ items }: { items: AttentionItem[] }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);
  useEffect(() => {
    if (selected && !items.some((item) => item.key === selected.key)) setSelected(null);
  }, [items.map((item) => item.key).join('|'), selected]);
  if (items.length === 0) return null;
  const primary = items.find((item) => item.primary) ?? items[0];
  const others = items.filter((item) => item !== primary);
  return (
    <div className={styles.actionBar}>
      <Button variant="primary" onClick={() => setSelected(primary)}>
        {actionLabel(primary)}
      </Button>
      {others.length > 0 ? <MoreActions items={others} onChoose={setSelected} /> : null}
      <ActionDialog item={selected} onClose={() => setSelected(null)} />
    </div>
  );
}

/** Menu das ações secundárias: abre pelo botão, anda pelas setas, Home e End, fecha com Esc, Tab ou clique fora. */
function MoreActions({ items, onChoose }: { items: AttentionItem[]; onChoose: (item: AttentionItem) => void }) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const entries = () => Array.from(rootRef.current?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []);
  const trigger = () => rootRef.current?.querySelector<HTMLElement>('[aria-haspopup="menu"]');

  useEffect(() => {
    if (!open) return;
    entries()[0]?.focus();
    const onPointer = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('mousedown', onPointer);
    return () => document.removeEventListener('mousedown', onPointer);
  }, [open]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (!open) return;
    const list = entries();
    const index = list.indexOf(document.activeElement as HTMLElement);
    const focusAt = (next: number) => {
      event.preventDefault();
      list[(next + list.length) % list.length]?.focus();
    };
    if (event.key === 'ArrowDown') focusAt(index + 1);
    else if (event.key === 'ArrowUp') focusAt(index - 1);
    else if (event.key === 'Home') focusAt(0);
    else if (event.key === 'End') focusAt(list.length - 1);
    else if (event.key === 'Tab') setOpen(false);
    else if (event.key === 'Escape') {
      event.preventDefault();
      setOpen(false);
      trigger()?.focus();
    }
  };

  return (
    <div ref={rootRef} className={styles.more} onKeyDown={onKeyDown}>
      <Button aria-haspopup="menu" aria-expanded={open} aria-controls={open ? menuId : undefined} onClick={() => setOpen((value) => !value)}>
        Mais ações
      </Button>
      {open ? (
        <div id={menuId} role="menu" aria-label="Mais ações" className={styles.menu}>
          {items.map((item) => (
            <button
              key={item.key}
              type="button"
              role="menuitem"
              tabIndex={-1}
              className={styles.menuItem}
              onClick={() => {
                setOpen(false);
                trigger()?.focus();
                onChoose(item);
              }}
            >
              {actionLabel(item)}
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}
