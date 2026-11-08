import { describe, expect, it } from 'vitest';
import type { OpenSpec, TaskSummary } from './api';
import { remainingTasks } from './attention';

const task = (id: string, status: string, wave = ''): TaskSummary => ({
  id,
  title: `Título ${id}`,
  status,
  wave,
  profile: '',
  persona: '',
  model: '',
  depends_on: '',
  covers: '',
  type: '',
  description: '',
});

const spec = (id: string, tasks: TaskSummary[]): OpenSpec => ({
  meta: { id, title: `Spec ${id}`, status: 'planned' } as OpenSpec['meta'],
  plan: false,
  plan_outdated: false,
  tasks,
});

describe('remainingTasks', () => {
  it('tira as tarefas concluídas', () => {
    const out = remainingTasks([spec('SPEC-001', [task('TASK-01', 'done'), task('TASK-02', 'approved')])]);
    expect(out.map((t) => t.id)).toEqual(['TASK-02']);
  });

  it('ordena por onda e deixa as sem onda por último', () => {
    const out = remainingTasks([
      spec('SPEC-001', [task('TASK-01', 'approved', '2'), task('TASK-02', 'approved', ''), task('TASK-03', 'approved', '1'), task('TASK-04', 'approved', '10')]),
    ]);
    expect(out.map((t) => t.wave)).toEqual(['1', '2', '10', '']);
  });

  it('desempata pelo ID dentro da onda', () => {
    const out = remainingTasks([spec('SPEC-001', [task('TASK-03', 'approved', '1'), task('TASK-01', 'approved', '1'), task('TASK-02', 'approved', '1')])]);
    expect(out.map((t) => t.id)).toEqual(['TASK-01', 'TASK-02', 'TASK-03']);
  });

  it('mantém a ordem das specs e põe a spec em cada tarefa', () => {
    const out = remainingTasks([spec('SPEC-002', [task('TASK-01', 'approved', '1')]), spec('SPEC-001', [task('TASK-05', 'pending', '1')])]);
    expect(out.map((t) => `${t.spec}/${t.id}`)).toEqual(['SPEC-002/TASK-01', 'SPEC-001/TASK-05']);
  });

  it('devolve vazio sem specs abertas', () => {
    expect(remainingTasks([])).toEqual([]);
  });
});
