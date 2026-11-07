import { listOf, type LoopRound, type Overview, type PanelRequest, type Question, type TaskSummary } from './api';

export type ActionKind =
  | 'approve-spec'
  | 'approve-plan'
  | 'approve-task'
  | 'finish-task'
  | 'unblock-task'
  | 'answer'
  | 'adjust-spec'
  | 'plan-spec'
  | 'redo-plan'
  | 'adjust-plan'
  | 'finalize-spec'
  | 'execute-task'
  | 'adjust-task'
  | 'fazer-commit'
  | 'create-spec'
  | 'refine-spec'
  | 'execute-wave'
  | 'release-commits'
  | 'undo-commits';

export type Effort = 'decisao' | 'resposta' | 'conferencia' | 'teste-manual';

export interface AttentionItem {
  key: string;
  kind: ActionKind;
  spec: string;
  task?: string;
  /** Tarefas aprovadas de uma vez ("N tarefa(s) pendente(s)" da spec). */
  tasks?: string[];
  /** Onda pedida por "Executar onda". */
  wave?: string;
  title: string;
  reason: string;
  href: string;
  question?: Question;
  /** Próximo passo do fluxo, em destaque na barra de ações. */
  primary?: boolean;
  /** Tipo de esforço que o item pede do usuário. */
  effort?: Effort;
  /** Rótulo da ação que precisa vir antes deste item. */
  after?: string;
  /** Primeiro item da spec em Pendências ("Faça agora"). */
  now?: boolean;
}

/** O que espera uma decisão do usuário, derivado só dos arquivos do harness, agrupado por spec na ordem do fluxo. */
export function attentionItems(overview: Overview): AttentionItem[] {
  const requests = overview.requests ?? [];
  const groups = new Map<string, AttentionItem[]>(overview.specs.map(({ meta }) => [meta.id, []]));
  const groupOf = (spec: string) => {
    if (!groups.has(spec)) groups.set(spec, []);
    return groups.get(spec) as AttentionItem[];
  };
  for (const question of overview.questions) {
    const { id, spec, task } = question.meta;
    if (activeRequest(requests, spec, task || undefined) || (task && activeRequest(requests, spec))) continue;
    groupOf(spec).push({
      key: `q-${id}`,
      kind: 'answer',
      spec,
      task: task || undefined,
      title: questionTitle(question.markdown),
      reason: 'O agente parou e espera a sua resposta.',
      href: task ? `/tarefas/${spec}/${task}` : `/specs/${spec}`,
      question,
      effort: 'resposta',
    });
  }
  for (const { meta, plan, plan_outdated, tasks } of overview.specs) {
    if (activeRequest(requests, meta.id)) continue;
    const items = groupOf(meta.id);
    if (meta.status === 'refined') {
      items.push({ key: `s-${meta.id}`, kind: 'approve-spec', spec: meta.id, title: meta.title, reason: 'Spec refinada, esperando aprovação.', href: `/specs/${meta.id}`, effort: 'conferencia' });
    }
    if (plan && !plan_outdated) {
      items.push({ key: `p-${meta.id}`, kind: 'approve-plan', spec: meta.id, title: meta.title, reason: 'Plano de tarefas para revisar.', href: `/specs/${meta.id}/plano`, effort: 'conferencia' });
    }
    const free = tasks.filter((task) => !activeRequest(requests, meta.id, task.id));
    const pending = free.filter((task) => task.status === 'pending');
    const ids = pending.map((task) => task.id);
    if (ids.length > 0) {
      items.push({
        key: `a-${meta.id}`,
        kind: 'approve-task',
        spec: meta.id,
        tasks: ids,
        // Com uma tarefa só, o título dela diz mais que o da spec, já citada no item.
        title: ids.length === 1 ? pending[0].title : meta.title,
        reason: `${ids.length} tarefa(s) detalhada(s), esperando aprovação: ${ids.join(', ')}.`,
        href: `/specs/${meta.id}`,
        effort: 'conferencia',
      });
    }
    const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => task.id));
    for (const task of free) {
      const base = { spec: meta.id, task: task.id, title: task.title, href: `/tarefas/${meta.id}/${task.id}` };
      if (task.status === 'in_progress' && task.check_result === 'passed' && listOf(task.depends_on).every((dep) => done.has(dep))) {
        items.push({ ...base, key: `f-${meta.id}-${task.id}`, kind: 'finish-task', reason: 'Implementada; falta validar e concluir.', effort: 'conferencia' });
      }
      if (task.status === 'validating') {
        items.push({ ...base, key: `f-${meta.id}-${task.id}`, kind: 'finish-task', reason: 'Em validação; falta a sua conferência dos prints ou do teste manual.', effort: 'teste-manual', href: `${base.href}#evidencias` });
      }
      if (task.status === 'blocked' && !isWaiting(task)) items.push({ ...base, key: `b-${meta.id}-${task.id}`, kind: 'unblock-task', reason: 'Bloqueada por falha ou regressão; precisa da sua decisão.', effort: 'decisao' });
    }
  }
  return [...groups.values()].flatMap(guided);
}

const STAGE: Partial<Record<ActionKind, number>> = { answer: 0, 'approve-spec': 1, 'approve-plan': 2, 'approve-task': 3, 'unblock-task': 4, 'finish-task': 4 };

/** Ordena os itens de uma spec pela etapa e marca o "Faça agora" e o "Depois de". */
function guided(items: AttentionItem[]): AttentionItem[] {
  const sorted = [...items].sort((a, b) => (STAGE[a.kind] ?? 5) - (STAGE[b.kind] ?? 5));
  return sorted.map((item, index) => {
    const blocker = sorted
      .slice(0, index)
      .reverse()
      .find((before) => blocks(before, item));
    return { ...item, now: index === 0, after: blocker ? afterLabel(blocker) : undefined };
  });
}

function blocks(before: AttentionItem, item: AttentionItem): boolean {
  if (before.kind === 'answer') {
    if (item.kind === 'answer') return false;
    if (!before.task) return true;
    return item.task === before.task || Boolean(item.tasks?.includes(before.task));
  }
  return before.kind === 'approve-plan' && item.kind === 'approve-task';
}

function afterLabel(item: AttentionItem): string {
  return item.kind === 'answer' && item.question ? `Responder à ${item.question.meta.id}` : actionLabel(item);
}

function questionTitle(markdown: string): string {
  const heading = markdown.match(/^#\s+\d+\.\s+(.+)$/m);
  if (heading) return heading[1].trim();
  const line = markdown
    .split('\n')
    .map((text) => text.trim())
    .find((text) => text && !text.startsWith('#') && !text.startsWith('Escolha:'));
  return line ?? 'Pergunta do agente';
}

export const ACTION_LABELS: Record<ActionKind, { button: string; tag: string; how: string }> = {
  'approve-spec': { button: 'Aprovar spec', tag: 'Aprovação', how: 'Aprova a spec nesta versão e registra que a aprovação veio de você, pelo painel. Depois dela, o plano de tarefas pode ser criado.' },
  'approve-plan': { button: 'Aprovar plano', tag: 'Aprovação', how: 'O agente detalha cada tarefa do plano. Elas ficam esperando a sua aprovação antes de serem executadas.' },
  'approve-task': { button: 'Aprovar tarefa', tag: 'Aprovação', how: 'Aprova a tarefa nesta versão e registra que a aprovação veio de você, pelo painel. Depois dela, a tarefa pode ser executada.' },
  'finish-task': { button: 'Pedir finalização', tag: 'Pedido ao agente', how: 'O agente roda a verificação da tarefa e a conclui. Se houver prints, eles ficam esperando a sua conferência.' },
  'unblock-task': { button: 'Decidir', tag: 'Decisão', how: 'Diga como seguir: corrigir, aceitar a diferença ou ajustar. O agente retoma a tarefa com a sua decisão.' },
  answer: { button: 'Responder', tag: 'Pergunta', how: 'O agente recebe a sua resposta e continua o trabalho de onde parou.' },
  'adjust-spec': { button: 'Ajustar a spec', tag: 'Pedido ao agente', how: 'Descreva o ajuste. O agente o aplica na spec e registra a mudança.' },
  'plan-spec': { button: 'Planejar tarefas', tag: 'Pedido ao agente', how: 'O agente divide a spec em tarefas e monta o plano para você revisar.' },
  'redo-plan': { button: 'Refazer o plano', tag: 'Pedido ao agente', how: 'A spec mudou depois do plano, e ele não cobre a versão aprovada. O agente monta o plano de novo para você revisar.' },
  'adjust-plan': { button: 'Ajustar o plano', tag: 'Pedido ao agente', how: 'Descreva o ajuste. O agente o aplica no plano e registra a mudança.' },
  'finalize-spec': { button: 'Finalizar spec', tag: 'Pedido ao agente', how: 'O agente gera os documentos finais e arquiva a spec.' },
  'execute-task': { button: 'Pedir execução', tag: 'Pedido ao agente', how: 'O agente começa a tarefa e segue os passos dela, sem fazer commit.' },
  'adjust-task': { button: 'Ajustar a tarefa', tag: 'Pedido ao agente', how: 'Descreva o ajuste. O agente o aplica na tarefa e registra a mudança.' },
  'create-spec': { button: 'Nova spec', tag: 'Pedido ao agente', how: 'Descreva o assunto. O agente cria o rascunho da spec.' },
  'refine-spec': { button: 'Refinar', tag: 'Pedido ao agente', how: 'O agente detalha o rascunho. Se tiver dúvidas, as perguntas aparecem em Pendências.' },
  'execute-wave': { button: 'Executar onda', tag: 'Pedido ao agente', how: 'O agente executa as tarefas prontas desta onda.' },
  'release-commits': { button: 'Liberar commits', tag: 'Commit', how: 'O agente libera os commits apontados, com a sua autorização registrada.' },
  'undo-commits': { button: 'Desfazer commits', tag: 'Commit', how: 'O agente tira os commits novos da branch atual e mantém as alterações na pasta de trabalho.' },
  'fazer-commit': { button: 'Pedir commit', tag: 'Commit', how: 'Descreva o que entra no commit. O agente o faz com a sua autorização registrada.' },
};

const PRIMARY: ActionKind[] = ['approve-spec', 'approve-plan', 'approve-task', 'answer', 'finish-task', 'finalize-spec', 'execute-task', 'plan-spec', 'redo-plan', 'create-spec', 'refine-spec', 'execute-wave'];
export const isPrimary = (kind: ActionKind) => PRIMARY.includes(kind);

/** Rótulo do botão do item, com o número de tarefas no lote de aprovação. */
export function actionLabel(item: AttentionItem): string {
  if (item.kind === 'approve-task' && item.tasks && item.tasks.length > 1) return `Aprovar ${item.tasks.length} tarefas`;
  return ACTION_LABELS[item.kind].button;
}

/** Ação da pergunta livre: ela só consulta e não muda a spec nem a tarefa, então não esconde as ações nem põe o selo. */
export const FREE_QUESTION = 'perguntar-agente';

/** Pedido ativo da spec (sem `task`) ou da tarefa: o de menor ID, que o agente atende primeiro. */
export function activeRequest(requests: PanelRequest[], spec: string, task?: string): PanelRequest | undefined {
  const idNumber = (request: PanelRequest) => Number(request.id.replace(/\D/g, ''));
  return requests
    .filter((request) => request.acao !== FREE_QUESTION && request.spec === spec && (task ? request.tarefa === task : !request.tarefa))
    .reduce<PanelRequest | undefined>((oldest, request) => (!oldest || idNumber(request) < idNumber(oldest) ? request : oldest), undefined);
}

const RUNNING_LABELS: Record<string, string> = {
  'refinar-spec': 'Refinando',
  'planejar-spec': 'Planejando',
  'aprovar-plano': 'Detalhando as tarefas',
  'ajustar-spec': 'Ajustando a spec',
  'ajustar-plano': 'Ajustando o plano',
  'ajustar-tarefa': 'Ajustando a tarefa',
};

/** Estado do pedido em palavras: na fila ou o verbo da ação em andamento. */
export function requestLabel(request: PanelRequest): string {
  if (request.estado === 'pendente') return 'Aguardando o agente';
  if (request.acao === 'finalizar-tarefa') return `Finalizando a ${request.tarefa || 'tarefa'}`;
  if (request.acao === 'executar-tarefa') return `Executando a ${request.tarefa || 'tarefa'}`;
  if (request.acao === 'executar-onda') return request.onda ? `Executando a onda ${request.onda}` : 'Executando a onda';
  return RUNNING_LABELS[request.acao] ?? 'Trabalhando no pedido';
}

/** Aviso do loop de pedidos para a barra lateral (TASK-10): nenhum agente ou agente ocupado ou parado. */
export function agentBusyLabel(loop: LoopRound | null, requests: PanelRequest[], now: Date): string | null {
  if (!requests.some((request) => request.estado === 'pendente')) return null;
  if (!loop) return 'Nenhum agente em loop';
  const last = new Date(loop.ultima_rodada);
  if (now.getTime() - last.getTime() <= 2 * loop.intervalo * 1000) return null;
  const hh = String(last.getHours()).padStart(2, '0');
  const mm = String(last.getMinutes()).padStart(2, '0');
  return `O agente está ocupado ou parado desde ${hh}:${mm}`;
}

/** Aviso de falta de nota, só enquanto o pedido apontado pela rodada continuar em andamento: sem nova rodada, o `.loop` fica velho. */
export function noteWarningLabel(loop: LoopRound | null, requests: PanelRequest[]): string | null {
  if (!loop?.sem_nota_pedido || !loop.sem_nota_desde) return null;
  const id = loop.sem_nota_pedido;
  if (!requests.some((request) => request.id === id && request.estado === 'em_andamento')) return null;
  const since = new Date(loop.sem_nota_desde);
  const hh = String(since.getHours()).padStart(2, '0');
  const mm = String(since.getMinutes()).padStart(2, '0');
  return `Sem nota do agente em ${id} desde ${hh}:${mm}`;
}

const FLOW: ActionKind[] = ['answer', 'refine-spec', 'approve-spec', 'redo-plan', 'plan-spec', 'approve-plan', 'approve-task', 'execute-wave', 'execute-task', 'unblock-task', 'finish-task', 'finalize-spec'];

/** Marca como primário só o item mais cedo na ordem do fluxo, sem mudar a ordem da lista. */
function withPrimary(items: AttentionItem[]): AttentionItem[] {
  if (items.length === 0) return items;
  const position = (item: AttentionItem) => (FLOW.includes(item.kind) ? FLOW.indexOf(item.kind) : FLOW.length);
  const first = items.reduce((best, item, index) => (position(item) < position(items[best]) ? index : best), 0);
  return items.map((item, index) => (index === first ? { ...item, primary: true } : item));
}

/** Ações disponíveis na página de uma spec, pelo status dela e das tarefas. */
export function specActions(meta: { id: string; title: string; status: string }, plan: boolean, tasks: TaskSummary[], planOutdated = false, requests: PanelRequest[] = []): AttentionItem[] {
  if (activeRequest(requests, meta.id)) return [];
  const free = tasks.filter((task) => !activeRequest(requests, meta.id, task.id));
  const base = { spec: meta.id, title: meta.title, reason: '', href: `/specs/${meta.id}` };
  const items: AttentionItem[] = [];
  if (meta.status === 'draft') items.push({ ...base, key: 'refine-spec', kind: 'refine-spec' });
  if (meta.status === 'refined') items.push({ ...base, key: 'approve-spec', kind: 'approve-spec' });
  if (plan && !planOutdated) {
    items.push({ ...base, key: 'approve-plan', kind: 'approve-plan' });
    items.push({ ...base, key: 'adjust-plan', kind: 'adjust-plan' });
  }
  if (plan && planOutdated && meta.status === 'approved') items.push({ ...base, key: 'redo-plan', kind: 'redo-plan' });
  if (meta.status === 'approved' && !plan) items.push({ ...base, key: 'plan-spec', kind: 'plan-spec' });
  if (meta.status === 'implemented') items.push({ ...base, key: 'finalize-spec', kind: 'finalize-spec' });
  if (meta.status === 'refined' || meta.status === 'approved') items.push({ ...base, key: 'adjust-spec', kind: 'adjust-spec' });
  const pending = free.filter((task) => task.status === 'pending').map((task) => task.id);
  if (pending.length > 0) {
    items.push({ ...base, key: 'approve-tasks', kind: 'approve-task', tasks: pending, title: `${pending.length} tarefa(s) pendente(s): ${pending.join(', ')}` });
  }
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => task.id));
  const ready = free.filter((task) => task.status === 'approved' && listOf(task.depends_on).every((dep) => done.has(dep)));
  if (meta.status === 'planned' && ready.length > 0) {
    const wave = String(Math.min(...ready.map((task) => Number(task.wave))));
    const ids = ready.filter((task) => task.wave === wave).map((task) => task.id);
    items.push({ ...base, key: 'execute-wave', kind: 'execute-wave', wave, tasks: ids, title: 'Onda ' + wave + ': ' + ids.join(', ') });
  }
  return withPrimary(items);
}

export const isWaiting = (task: { waiting_on?: string }) => listOf(task.waiting_on ?? '').length > 0;

/** Ações disponíveis na página de uma tarefa, pelo status dela e das dependências. */
export function taskActions(spec: string, state: Record<string, string>, depsDone: boolean, requests: PanelRequest[] = []): AttentionItem[] {
  if (activeRequest(requests, spec, state.id) || activeRequest(requests, spec)) return [];
  const base = { spec, task: state.id, title: state.title, reason: '', href: `/tarefas/${spec}/${state.id}` };
  const items: AttentionItem[] = [];
  if (state.status === 'pending') items.push({ ...base, key: 'approve', kind: 'approve-task' });
  if (state.status === 'approved' && depsDone) items.push({ ...base, key: 'execute', kind: 'execute-task' });
  if (state.status === 'in_progress' || state.status === 'validating') items.push({ ...base, key: 'finish', kind: 'finish-task' });
  if (state.status === 'blocked' && !isWaiting(state)) items.push({ ...base, key: 'unblock', kind: 'unblock-task' });
  if (state.status !== 'done') items.push({ ...base, key: 'adjust', kind: 'adjust-task' });
  return withPrimary(items);
}
