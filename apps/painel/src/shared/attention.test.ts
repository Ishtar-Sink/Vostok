import { describe, expect, it } from 'vitest';
import { ACTION_LABELS, actionLabel, activeRequest, agentBusyLabel, attentionItems, requestLabel, specActions, taskActions } from './attention';
import type { Overview, PanelRequest, TaskSummary } from './api';

const request = (id: string, acao: string, over: Partial<PanelRequest> = {}): PanelRequest => ({ id, acao, spec: 'SPEC-001', tarefa: '', onda: '', estado: 'pendente', atualizado_em: '2026-11-03T10:00:00Z', ...over });

function task(id: string, status: string, depends_on = '', wave = '1'): TaskSummary {
  return { id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on, covers: '', type: 'tarefa', description: '', last_outcome: null };
}

const meta = { id: 'SPEC-001', title: 'Spec', status: 'approved' };

describe('taskActions', () => {
  const kinds = (status: string, depsDone = true) => taskActions('SPEC-001', { id: 'TASK-01', title: 'T', status }, depsDone).map((item) => item.kind);

  it('offers approval for a pending task', () => {
    expect(kinds('pending')).toContain('approve-task');
  });

  it('offers a finish request for an in-progress task', () => {
    expect(kinds('in_progress')).toContain('finish-task');
  });

  it('oferece finalizar na tarefa em validação', () => {
    expect(kinds('validating')).toContain('finish-task');
    expect(kinds('validating')).toContain('adjust-task');
    expect(kinds('validating')).not.toContain('execute-task');
  });

  it('offers execution only when the dependencies are done', () => {
    expect(kinds('approved', true)).toContain('execute-task');
    expect(kinds('approved', false)).not.toContain('execute-task');
  });

  it('offers no action for a done task', () => {
    expect(kinds('done')).toEqual([]);
  });
});

describe('specActions', () => {
  it('groups the pending tasks into one approval', () => {
    const items = specActions(meta, false, [task('TASK-01', 'pending'), task('TASK-02', 'pending')]);
    expect(items.find((item) => item.kind === 'approve-task')?.tasks).toEqual(['TASK-01', 'TASK-02']);
  });

  it('offers planning for an approved spec without a plan', () => {
    expect(specActions(meta, false, []).map((item) => item.kind)).toContain('plan-spec');
  });

  it('oferece refinar para a spec em rascunho', () => {
    const kinds = specActions({ ...meta, status: 'draft' }, false, []).map((item) => item.kind);
    expect(kinds).toContain('refine-spec');
    expect(kinds).not.toContain('adjust-spec');
  });

  it('troca aprovar por refazer com o plano desatualizado', () => {
    const kinds = specActions(meta, true, [], true).map((item) => item.kind);
    expect(kinds).toContain('redo-plan');
    expect(kinds).not.toContain('approve-plan');
    expect(kinds).not.toContain('adjust-plan');
    expect(kinds).not.toContain('plan-spec');
  });

  it('não oferece ação de plano na spec refinada com plano desatualizado', () => {
    expect(specActions({ ...meta, status: 'refined' }, true, [], true).map((item) => item.kind)).toEqual(['approve-spec', 'adjust-spec']);
  });

  it('oferece ajustar e aprovar com o plano em dia', () => {
    const kinds = specActions(meta, true, [], false).map((item) => item.kind);
    expect(kinds).toContain('approve-plan');
    expect(kinds).toContain('adjust-plan');
    expect(kinds).not.toContain('redo-plan');
    expect(kinds.indexOf('approve-plan')).toBeLessThan(kinds.indexOf('adjust-plan'));
  });

  it('oferece executar a próxima onda executável', () => {
    const items = specActions({ ...meta, status: 'planned' }, false, [
      task('TASK-01', 'done', '', '1'),
      task('TASK-02', 'approved', '[TASK-01]', '2'),
      task('TASK-03', 'approved', '', '2'),
      task('TASK-04', 'approved', '[TASK-02]', '3'),
    ]);
    const wave = items.find((item) => item.kind === 'execute-wave');
    expect(wave?.wave).toBe('2');
    expect(wave?.tasks).toEqual(['TASK-02', 'TASK-03']);
    expect(wave?.title).toBe('Onda 2: TASK-02, TASK-03');
    expect(items.some((item) => item.kind === 'execute-task')).toBe(false);
  });

  it('não oferece executar onda sem tarefa pronta', () => {
    const items = specActions({ ...meta, status: 'planned' }, false, [task('TASK-01', 'approved', '[TASK-09]')]);
    expect(items.some((item) => item.kind === 'execute-wave')).toBe(false);
  });
});

describe('espera por dependência', () => {
  const overviewWith = (tasks: TaskSummary[]) => ({ project: 'p', specs: [{ meta, plan: false, tasks }], archived: [], activity: [], questions: [] }) as unknown as Overview;
  const unblockKeys = (tasks: TaskSummary[]) => attentionItems(overviewWith(tasks)).filter((item) => item.kind === 'unblock-task').map((item) => item.task);

  it('não lista a tarefa em espera nas Pendências', () => {
    const waiting = { ...task('TASK-02', 'blocked'), waiting_on: '[TASK-01]' };
    expect(unblockKeys([waiting])).toEqual([]);
    expect(unblockKeys([waiting, task('TASK-03', 'blocked')])).toEqual(['TASK-03']);
  });

  it('não oferece Decidir na tarefa em espera', () => {
    const kinds = taskActions('SPEC-001', { id: 'TASK-02', title: 'T', status: 'blocked', waiting_on: '[TASK-01]' }, true).map((item) => item.kind);
    expect(kinds).not.toContain('unblock-task');
    expect(kinds).toContain('adjust-task');
  });

  it.each(['', '[]'])('waiting_on vazio ou [] mantém a decisão (%j)', (waiting_on) => {
    expect(unblockKeys([{ ...task('TASK-02', 'blocked'), waiting_on }])).toEqual(['TASK-02']);
    const kinds = taskActions('SPEC-001', { id: 'TASK-02', title: 'T', status: 'blocked', waiting_on }, true).map((item) => item.kind);
    expect(kinds).toContain('unblock-task');
  });
});

describe('attentionItems', () => {
  it('leva à seção Evidências a tarefa em validação', () => {
    const hrefOf = (status: string) => {
      const overview = { project: 'p', specs: [{ meta, plan: false, tasks: [{ ...task('TASK-01', status), check_result: 'passed' }] }], archived: [], activity: [], questions: [] } as unknown as Overview;
      return attentionItems(overview).find((item) => item.kind === 'finish-task')?.href ?? '';
    };
    expect(hrefOf('validating').endsWith('#evidencias')).toBe(true);
    expect(hrefOf('in_progress').endsWith('#evidencias')).toBe(false);
  });

  it('lists a pending task as an approval and a blocked one as a decision', () => {
    const overview = {
      project: 'p',
      specs: [{ meta, plan: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'blocked')] }],
      archived: [],
      activity: [],
      questions: [],
    } as unknown as Overview;
    expect(attentionItems(overview).map((item) => item.kind)).toEqual(['approve-task', 'unblock-task']);
  });

  it('não lista aprovar plano com o plano desatualizado', () => {
    const kindsFor = (plan_outdated: boolean) => {
      const overview = { project: 'p', specs: [{ meta, plan: true, plan_outdated, tasks: [] }], archived: [], activity: [], questions: [] } as unknown as Overview;
      return attentionItems(overview).map((item) => item.kind);
    };
    expect(kindsFor(true)).not.toContain('approve-plan');
    expect(kindsFor(false)).toContain('approve-plan');
  });

  it('só pede finalização com a conferência passada', () => {
    const inProgress = (check_result?: string): TaskSummary => ({ ...task('TASK-01', 'in_progress'), check_result });
    const kindsFor = (...tasks: TaskSummary[]) => {
      const overview = { project: 'p', specs: [{ meta, plan: false, tasks }], archived: [], activity: [], questions: [] } as unknown as Overview;
      return attentionItems(overview).map((item) => item.kind);
    };
    expect(kindsFor(inProgress())).not.toContain('finish-task');
    expect(kindsFor(inProgress('passed'))).toContain('finish-task');
    expect(kindsFor(inProgress('failed'))).not.toContain('finish-task');
  });
});

describe('ACTION_LABELS', () => {
  it('tem as 19 ações', () => {
    expect(Object.keys(ACTION_LABELS)).toHaveLength(19);
  });

  it('explica cada ação sem comando, caminho, nome de skill nem crase', () => {
    const proibidos = [
      /`/, /\/vostok /i, /work\//, /\.log\b/, /<SPEC>/, /skill/i, /\bCLI\b/,
      ...['adjust-plan', 'adjust-spec', 'adjust-task', 'approve-commit', 'approve-plan', 'approve-spec', 'approve-task', 'create-spec', 'execute-task', 'finalize-spec', 'finish-task', 'plan-tasks', 'process-requests', 'refine-spec'].map((nome) => new RegExp(nome)),
    ];
    for (const [kind, label] of Object.entries(ACTION_LABELS)) {
      for (const texto of [label.how, label.button, label.tag]) {
        for (const proibido of proibidos) {
          expect(texto, `${kind}: ${proibido}`).not.toMatch(proibido);
        }
      }
    }
  });
});

describe('activeRequest', () => {
  it('devolve o pedido de menor ID da spec', () => {
    const requests = [request('PED-010', 'ajustar-spec'), request('PED-002', 'planejar-spec'), request('PED-003', 'finalizar-tarefa', { tarefa: 'TASK-01' })];
    expect(activeRequest(requests, 'SPEC-001')?.id).toBe('PED-002');
    expect(activeRequest(requests, 'SPEC-009')).toBeUndefined();
  });

  it('separa os pedidos da tarefa dos da spec', () => {
    const requests = [request('PED-001', 'planejar-spec'), request('PED-004', 'executar-tarefa', { tarefa: 'TASK-02' })];
    expect(activeRequest(requests, 'SPEC-001', 'TASK-02')?.id).toBe('PED-004');
    expect(activeRequest(requests, 'SPEC-001', 'TASK-01')).toBeUndefined();
  });
});

describe('requestLabel', () => {
  it('diz aguardando o agente na fila', () => {
    expect(requestLabel(request('PED-001', 'planejar-spec'))).toBe('Aguardando o agente');
  });

  it('usa o verbo da ação em andamento', () => {
    const running = (acao: string, over: Partial<PanelRequest> = {}) => requestLabel(request('PED-001', acao, { estado: 'em_andamento', ...over }));
    expect(running('refinar-spec')).toBe('Refinando');
    expect(running('planejar-spec')).toBe('Planejando');
    expect(running('aprovar-plano')).toBe('Detalhando as tarefas');
    expect(running('executar-onda', { onda: '2' })).toBe('Executando a onda 2');
    expect(running('executar-onda')).toBe('Executando a onda');
    expect(running('finalizar-tarefa', { tarefa: 'TASK-03' })).toBe('Finalizando a TASK-03');
    expect(running('executar-tarefa', { tarefa: 'TASK-04' })).toBe('Executando a TASK-04');
    expect(running('ajustar-spec')).toBe('Ajustando a spec');
    expect(running('ajustar-plano')).toBe('Ajustando o plano');
    expect(running('ajustar-tarefa')).toBe('Ajustando a tarefa');
    expect(running('fazer-commit')).toBe('Trabalhando no pedido');
  });
});

describe('pedido ativo', () => {
  it('specActions sem ações com pedido ativo da spec', () => {
    expect(specActions(meta, true, [task('TASK-01', 'pending')], false, [request('PED-001', 'aprovar-plano', { estado: 'em_andamento' })])).toEqual([]);
    const items = specActions(meta, true, [task('TASK-01', 'pending')], false, [request('PED-001', 'ajustar-tarefa', { tarefa: 'TASK-01' })]);
    expect(items).not.toEqual([]);
    expect(items.some((item) => item.kind === 'approve-task')).toBe(false);
  });

  it('taskActions sem ações com pedido ativo da tarefa ou da spec', () => {
    const state = { id: 'TASK-01', title: 'T', status: 'in_progress' };
    expect(taskActions('SPEC-001', state, true, [request('PED-001', 'finalizar-tarefa', { tarefa: 'TASK-01' })])).toEqual([]);
    expect(taskActions('SPEC-001', state, true, [request('PED-002', 'executar-onda')])).toEqual([]);
    expect(taskActions('SPEC-001', state, true, [request('PED-003', 'ajustar-tarefa', { tarefa: 'TASK-09' })]).map((item) => item.kind)).toContain('finish-task');
  });

  it('tira do lote e da onda as tarefas com pedido ativo', () => {
    const items = specActions({ ...meta, status: 'planned' }, false, [task('TASK-01', 'pending'), task('TASK-02', 'pending'), task('TASK-03', 'approved')], false, [
      request('PED-001', 'ajustar-tarefa', { tarefa: 'TASK-02' }),
      request('PED-002', 'executar-tarefa', { tarefa: 'TASK-03' }),
    ]);
    expect(items.find((item) => item.kind === 'approve-task')?.tasks).toEqual(['TASK-01']);
    expect(items.some((item) => item.kind === 'execute-wave')).toBe(false);
  });

  it('tira as tarefas em detalhamento durante o aprovar-plano', () => {
    const running = [request('PED-001', 'aprovar-plano', { estado: 'em_andamento' })];
    const overview = {
      project: 'p',
      specs: [{ meta: { ...meta, status: 'approved' }, plan: true, plan_outdated: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'pending')] }],
      archived: [],
      activity: [],
      questions: [],
      requests: running,
    } as unknown as Overview;
    expect(attentionItems(overview)).toEqual([]);
    expect(specActions(meta, true, [task('TASK-01', 'pending')], false, running).some((item) => item.kind === 'approve-task')).toBe(false);
  });

  it('attentionItems tira as specs e tarefas com pedido ativo', () => {
    const overview = {
      project: 'p',
      specs: [
        { meta: { ...meta, status: 'refined' }, plan: false, tasks: [] },
        { meta: { ...meta, id: 'SPEC-002' }, plan: false, tasks: [task('TASK-01', 'blocked'), task('TASK-02', 'blocked')] },
      ],
      archived: [],
      activity: [],
      questions: [],
      requests: [request('PED-001', 'ajustar-spec'), request('PED-002', 'desbloquear-tarefa', { spec: 'SPEC-002', tarefa: 'TASK-01' })],
    } as unknown as Overview;
    expect(attentionItems(overview).map((item) => item.task ?? item.kind)).toEqual(['TASK-02']);
  });
});

describe('ação principal', () => {
  it('marca um só primário na spec com plano e tarefas pendentes', () => {
    const items = specActions(meta, true, [task('TASK-01', 'pending'), task('TASK-02', 'pending')]);
    const primary = items.filter((item) => item.primary);
    expect(primary).toHaveLength(1);
    expect(primary[0].kind).toBe('approve-plan');
    expect(items.filter((item) => !item.primary).map(actionLabel)).toEqual(['Ajustar o plano', 'Ajustar a spec', 'Aprovar 2 tarefas']);
  });

  it('marca um só primário em cada status da tarefa', () => {
    for (const status of ['pending', 'approved', 'in_progress', 'validating', 'blocked']) {
      const primary = taskActions('SPEC-001', { id: 'TASK-01', title: 'T', status }, true).filter((item) => item.primary);
      expect(primary).toHaveLength(1);
      expect(primary[0].kind).not.toBe('adjust-task');
    }
    const waiting = taskActions('SPEC-001', { id: 'TASK-01', title: 'T', status: 'approved' }, false);
    expect(waiting.map((item) => item.kind)).toEqual(['adjust-task']);
    expect(waiting[0].primary).toBe(true);
  });

  it('marca um só primário em cada status da spec', () => {
    const cases: [string, boolean, TaskSummary[], string][] = [
      ['draft', false, [], 'refine-spec'],
      ['refined', false, [], 'approve-spec'],
      ['approved', false, [], 'plan-spec'],
      ['planned', false, [task('TASK-01', 'approved')], 'execute-wave'],
      ['implemented', false, [], 'finalize-spec'],
    ];
    for (const [status, plan, tasks, kind] of cases) {
      const primary = specActions({ ...meta, status }, plan, tasks).filter((item) => item.primary);
      expect(primary).toHaveLength(1);
      expect(primary[0].kind).toBe(kind);
    }
  });

  it('diz onde agem os rótulos de ajuste', () => {
    expect(ACTION_LABELS['adjust-spec'].button).toBe('Ajustar a spec');
    expect(ACTION_LABELS['adjust-plan'].button).toBe('Ajustar o plano');
    expect(ACTION_LABELS['adjust-task'].button).toBe('Ajustar a tarefa');
    const batch = { key: 'a', kind: 'approve-task' as const, spec: 'SPEC-001', title: '', reason: '', href: '' };
    expect(actionLabel({ ...batch, tasks: ['TASK-01', 'TASK-02', 'TASK-03'] })).toBe('Aprovar 3 tarefas');
    expect(actionLabel({ ...batch, tasks: ['TASK-01'] })).toBe('Aprovar tarefa');
  });
});

describe('fila guiada', () => {
  const overviewOf = (specs: unknown[], questions: unknown[] = []) => ({ project: 'p', specs, archived: [], activity: [], questions, requests: [] }) as unknown as Overview;
  const questionOf = (id: string, spec: string, markdown: string) => ({ meta: { id, spec, task: '', status: 'aberta', created_at: '', author: '' }, path: 'p.md', markdown });

  it('ordena pela ordem do fluxo com faça agora e depois de', () => {
    const items = attentionItems(
      overviewOf([{ meta, plan: true, plan_outdated: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'pending')] }], [questionOf('P-001', 'SPEC-001', '# 1. Stack\n\nQual stack?')]),
    );
    expect(items.map((item) => item.kind)).toEqual(['answer', 'approve-plan', 'approve-task']);
    expect(items.map((item) => item.now)).toEqual([true, false, false]);
    expect(items.map((item) => item.after)).toEqual([undefined, 'Responder à P-001', 'Aprovar plano']);
    expect(items.map((item) => item.effort)).toEqual(['resposta', 'conferencia', 'conferencia']);
    expect(items.find((item) => item.kind === 'approve-task')?.tasks).toEqual(['TASK-01', 'TASK-02']);
  });

  it('mantém sem dependência os itens independentes', () => {
    const items = attentionItems(overviewOf([{ meta: { ...meta, status: 'planned' }, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'blocked')] }]));
    expect(items.map((item) => item.kind)).toEqual(['approve-task', 'unblock-task']);
    expect(items.map((item) => item.now)).toEqual([true, false]);
    expect(items.map((item) => item.after)).toEqual([undefined, undefined]);
    expect(items.find((item) => item.kind === 'unblock-task')?.effort).toBe('decisao');
  });

  it('pede teste manual na tarefa em validação', () => {
    const items = attentionItems(overviewOf([{ meta: { ...meta, status: 'planned' }, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'validating')] }]));
    expect(items.map((item) => [item.kind, item.effort])).toEqual([['finish-task', 'teste-manual']]);
  });

  it('usa o título da pergunta pelo cabeçalho', () => {
    const titleOf = (markdown: string) => attentionItems(overviewOf([], [questionOf('P-001', 'SPEC-001', markdown)])).find((item) => item.kind === 'answer')?.title;
    expect(titleOf('# 1. Stack do app\n\nEscolha: única\n\n| Opção | Descrição |\n|---|---|\n| A | Rust (recomendada) |')).toBe('Stack do app');
    expect(titleOf('## Pergunta\nQual caminho seguir?')).toBe('Qual caminho seguir?');
  });

  it('agrupa por spec na ordem das specs', () => {
    const items = attentionItems(
      overviewOf(
        [
          { meta: { ...meta, status: 'refined' }, plan: false, plan_outdated: false, tasks: [] },
          { meta: { ...meta, id: 'SPEC-002', status: 'planned' }, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'blocked')] },
        ],
        [questionOf('P-001', 'SPEC-002', '# 1. Dúvida\n\nQual?')],
      ),
    );
    expect(items.map((item) => item.spec)).toEqual(['SPEC-001', 'SPEC-002', 'SPEC-002']);
    expect(items.map((item) => item.kind)).toEqual(['approve-spec', 'answer', 'unblock-task']);
    expect(items.map((item) => item.now)).toEqual([true, true, false]);
  });
});

describe('agentBusyLabel', () => {
  const now = new Date(2026, 10, 3, 14, 10);
  const loop = (minutos: number) => ({ ultima_rodada: new Date(now.getTime() - minutos * 60_000).toISOString(), intervalo: 60, agente: 'claude' });

  it('avisa nenhum agente em loop sem o registro', () => {
    expect(agentBusyLabel(null, [request('PED-001', 'planejar-spec')], now)).toBe('Nenhum agente em loop');
  });

  it('avisa o agente ocupado depois de dois intervalos', () => {
    expect(agentBusyLabel(loop(5), [request('PED-001', 'planejar-spec')], now)).toBe('O agente está ocupado ou parado desde 14:05');
  });

  it('não avisa dentro de dois intervalos nem sem pedido na fila', () => {
    expect(agentBusyLabel(loop(2), [request('PED-001', 'planejar-spec')], now)).toBeNull();
    expect(agentBusyLabel(loop(5), [request('PED-001', 'planejar-spec', { estado: 'em_andamento' })], now)).toBeNull();
    expect(agentBusyLabel(null, [], now)).toBeNull();
  });
});

describe('pergunta livre', () => {
  const free = request('PED-009', 'perguntar-agente', { estado: 'pendente' });

  it('pergunta livre não conta como pedido ativo', () => {
    expect(activeRequest([free], 'SPEC-001')).toBeUndefined();
    const other = request('PED-010', 'ajustar-spec');
    expect(activeRequest([free, other], 'SPEC-001')?.id).toBe('PED-010');
    expect(activeRequest([other, free], 'SPEC-001')?.id).toBe('PED-010');
  });

  it('pergunta livre não tira as ações da spec e da tarefa', () => {
    const tasks = [task('TASK-01', 'pending'), task('TASK-02', 'approved')];
    const kinds = (requests: PanelRequest[]) => specActions({ ...meta, status: 'planned' }, false, tasks, false, requests).map((item) => item.kind);
    expect(kinds([free])).toEqual(kinds([]));
    expect(kinds([free]).length).toBeGreaterThan(0);
    const state = { id: 'TASK-01', title: 'T', status: 'in_progress' };
    const own = { ...free, tarefa: 'TASK-01' };
    expect(taskActions('SPEC-001', state, true, [own]).map((item) => item.kind)).toEqual(taskActions('SPEC-001', state, true).map((item) => item.kind));
  });

  it('pergunta livre não tira os itens das Pendências', () => {
    const overview = (requests: PanelRequest[]) =>
      ({
        project: 'p',
        specs: [{ meta: { ...meta, status: 'planned' }, plan: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'blocked')] }],
        archived: [],
        activity: [],
        questions: [],
        requests,
      }) as unknown as Overview;
    const keys = (requests: PanelRequest[]) => attentionItems(overview(requests)).map((item) => item.key);
    expect(keys([free])).toEqual(keys([]));
    expect(keys([free]).length).toBeGreaterThan(0);
    expect(keys([request('PED-011', 'ajustar-spec')])).toEqual([]);
  });
});
