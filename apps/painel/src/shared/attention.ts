import { listOf, type Overview, type Question, type TaskSummary } from './api';

export type ActionKind =
  | 'approve-spec'
  | 'approve-plan'
  | 'approve-task'
  | 'finish-task'
  | 'unblock-task'
  | 'answer'
  | 'adjust-spec'
  | 'plan-spec'
  | 'finalize-spec'
  | 'execute-task'
  | 'adjust-task';

export interface AttentionItem {
  key: string;
  kind: ActionKind;
  spec: string;
  task?: string;
  /** Tarefas aprovadas de uma vez ("N tarefa(s) pendente(s)" da spec). */
  tasks?: string[];
  title: string;
  reason: string;
  href: string;
  question?: Question;
}

/** O que espera uma decisão do usuário, derivado só dos arquivos do harness. */
export function attentionItems(overview: Overview): AttentionItem[] {
  const items: AttentionItem[] = [];
  for (const question of overview.questions) {
    const { id, spec, task } = question.meta;
    items.push({
      key: `q-${id}`,
      kind: 'answer',
      spec,
      task: task || undefined,
      title: firstLine(question.markdown),
      reason: 'O agente parou e espera a sua resposta.',
      href: task ? `/tarefas/${spec}/${task}` : `/specs/${spec}`,
      question,
    });
  }
  for (const { meta, plan, tasks } of overview.specs) {
    if (meta.status === 'refined') {
      items.push({ key: `s-${meta.id}`, kind: 'approve-spec', spec: meta.id, title: meta.title, reason: 'Spec refinada, esperando aprovação.', href: `/specs/${meta.id}` });
    }
    if (plan) {
      items.push({ key: `p-${meta.id}`, kind: 'approve-plan', spec: meta.id, title: meta.title, reason: 'Plano de tarefas para revisar.', href: `/specs/${meta.id}/plano` });
    }
    const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => task.id));
    for (const task of tasks) {
      const base = { spec: meta.id, task: task.id, title: task.title, href: `/tarefas/${meta.id}/${task.id}` };
      if (task.status === 'pending') items.push({ ...base, key: `a-${meta.id}-${task.id}`, kind: 'approve-task', reason: 'Tarefa detalhada, esperando aprovação.' });
      if (task.status === 'in_progress' && listOf(task.depends_on).every((dep) => done.has(dep))) {
        items.push({ ...base, key: `f-${meta.id}-${task.id}`, kind: 'finish-task', reason: 'Implementada; falta validar e concluir.' });
      }
      if (task.status === 'blocked') items.push({ ...base, key: `b-${meta.id}-${task.id}`, kind: 'unblock-task', reason: 'Bloqueada por falha ou regressão; precisa da sua decisão.' });
    }
  }
  return items;
}

function firstLine(markdown: string): string {
  const line = markdown.split('\n').map((text) => text.trim()).find((text) => text && !text.startsWith('#'));
  return line ?? 'Pergunta do agente';
}

export const ACTION_LABELS: Record<ActionKind, { button: string; tag: string; how: string }> = {
  'approve-spec': { button: 'Aprovar spec', tag: 'Aprovação', how: 'O painel chama a mesma aprovação da CLI (`vostok spec approve`), grava a origem `painel` e o aval em `work/avais.log`.' },
  'approve-plan': { button: 'Aprovar plano', tag: 'Aprovação', how: 'Aprovar o plano exige detalhar as tarefas: o painel grava um pedido, e o agente em loop o processa pela skill approve-plan.' },
  'approve-task': { button: 'Aprovar tarefa', tag: 'Aprovação', how: 'O painel chama a mesma aprovação da CLI (`vostok task approve`), grava a origem `painel` e o aval em `work/avais.log`.' },
  'finish-task': { button: 'Pedir finalização', tag: 'Pedido ao agente', how: 'O painel grava um pedido de finalização; o agente em loop roda a verificação e conclui pela skill finish-task.' },
  'unblock-task': { button: 'Decidir', tag: 'Decisão', how: 'O painel grava a sua decisão (corrigir, aceitar a diferença ou ajustar) como pedido ao agente.' },
  answer: { button: 'Responder', tag: 'Pergunta', how: 'A resposta vira um pedido ao agente em loop, que a grava na pergunta e continua o trabalho.' },
  'adjust-spec': { button: 'Pedir ajuste', tag: 'Pedido ao agente', how: 'Você descreve o ajuste; o painel grava um pedido e o agente em loop o aplica pela skill adjust-spec.' },
  'plan-spec': { button: 'Planejar tarefas', tag: 'Pedido ao agente', how: 'O painel grava um pedido de planejamento; o agente em loop cria o plano pela skill plan-tasks.' },
  'finalize-spec': { button: 'Finalizar spec', tag: 'Pedido ao agente', how: 'O painel grava um pedido de finalização; o agente em loop gera os documentos e arquiva a spec.' },
  'execute-task': { button: 'Pedir execução', tag: 'Pedido ao agente', how: 'O painel grava um pedido de execução; o agente em loop inicia a tarefa e coleta o baseline.' },
  'adjust-task': { button: 'Pedir ajuste', tag: 'Pedido ao agente', how: 'Você descreve o ajuste; o painel grava um pedido e o agente em loop o aplica pela skill adjust-task.' },
};

const PRIMARY: ActionKind[] = ['approve-spec', 'approve-plan', 'approve-task', 'answer', 'finish-task', 'finalize-spec', 'execute-task', 'plan-spec'];
export const isPrimary = (kind: ActionKind) => PRIMARY.includes(kind);

/** Ações disponíveis na página de uma spec, pelo status dela e das tarefas. */
export function specActions(meta: { id: string; title: string; status: string }, plan: boolean, tasks: TaskSummary[]): AttentionItem[] {
  const base = { spec: meta.id, title: meta.title, reason: '', href: `/specs/${meta.id}` };
  const items: AttentionItem[] = [];
  if (meta.status === 'refined') items.push({ ...base, key: 'approve-spec', kind: 'approve-spec' });
  if (plan) items.push({ ...base, key: 'approve-plan', kind: 'approve-plan' });
  if (meta.status === 'approved' && !plan) items.push({ ...base, key: 'plan-spec', kind: 'plan-spec' });
  if (meta.status === 'implemented') items.push({ ...base, key: 'finalize-spec', kind: 'finalize-spec' });
  if (meta.status === 'refined' || meta.status === 'approved') items.push({ ...base, key: 'adjust-spec', kind: 'adjust-spec' });
  if (tasks.some((task) => task.status === 'pending')) {
    const pending = tasks.filter((task) => task.status === 'pending').map((task) => task.id);
    items.push({ ...base, key: 'approve-tasks', kind: 'approve-task', tasks: pending, title: `${pending.length} tarefa(s) pendente(s): ${pending.join(', ')}` });
  }
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => task.id));
  const ready = tasks.filter((task) => task.status === 'approved' && listOf(task.depends_on).every((dep) => done.has(dep)));
  if (ready.length > 0) {
    items.push({ ...base, key: 'execute-ready', kind: 'execute-task', title: ready.map((task) => task.id).join(', ') });
  }
  return items;
}

/** Ações disponíveis na página de uma tarefa, pelo status dela e das dependências. */
export function taskActions(spec: string, state: Record<string, string>, depsDone: boolean): AttentionItem[] {
  const base = { spec, task: state.id, title: state.title, reason: '', href: `/tarefas/${spec}/${state.id}` };
  const items: AttentionItem[] = [];
  if (state.status === 'pending') items.push({ ...base, key: 'approve', kind: 'approve-task' });
  if (state.status === 'approved' && depsDone) items.push({ ...base, key: 'execute', kind: 'execute-task' });
  if (state.status === 'in_progress') items.push({ ...base, key: 'finish', kind: 'finish-task' });
  if (state.status === 'blocked') items.push({ ...base, key: 'unblock', kind: 'unblock-task' });
  if (state.status !== 'done') items.push({ ...base, key: 'adjust', kind: 'adjust-task' });
  return items;
}
