import { PageHeader } from '../../ui/page-header/page-header';

export function TaskPage({ task }: { spec: string; task: string; version: number }) {
  return <PageHeader eyebrow="Tarefa" title="Tarefa" />;
}
