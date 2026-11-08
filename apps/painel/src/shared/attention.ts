import type { OpenSpec, TaskSummary } from './api';

/** Tarefa que falta concluir, com a spec dela. */
export interface RemainingTask extends TaskSummary {
  spec: string;
}

const waveOrder = (wave: string) => (wave === '' ? Number.POSITIVE_INFINITY : Number(wave));

/** As tarefas não concluídas de cada spec, na ordem das specs, por onda (as sem onda por último) e por ID. */
export function remainingTasks(specs: OpenSpec[]): RemainingTask[] {
  return specs.flatMap((spec) =>
    spec.tasks
      .filter((task) => task.status !== 'done')
      .map((task) => ({ ...task, spec: spec.meta.id }))
      .sort((a, b) => waveOrder(a.wave) - waveOrder(b.wave) || a.id.localeCompare(b.id)),
  );
}
