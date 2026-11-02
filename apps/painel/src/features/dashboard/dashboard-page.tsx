import { PageHeader } from '../../ui/page-header/page-header';

export function DashboardPage({ name }: { version: number; name: string }) {
  return <PageHeader eyebrow="Painel" title="Painel" />;
}
