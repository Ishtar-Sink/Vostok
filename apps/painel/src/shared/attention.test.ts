import { describe, expect, it } from 'vitest';
import { ACTION_LABELS, attentionItems, specActions, taskActions } from './attention';
import type { Overview, TaskSummary } from './api';

function task(id: string, status: string, depends_on = '', wave = '1'): TaskSummary {
  return { id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on, covers: '', type: 'tarefa', description: '' };
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

describe('attentionItems', () => {
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
  it('cita o log de avais da spec na ajuda das aprovações', () => {
    for (const kind of ['approve-spec', 'approve-task'] as const) {
      expect(ACTION_LABELS[kind].how).toContain('work/avais/<SPEC>.log');
      expect(ACTION_LABELS[kind].how).not.toMatch(/avais\.log/);
    }
  });
});
