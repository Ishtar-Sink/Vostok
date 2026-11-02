import { PageHeader } from '../../ui/page-header/page-header';

export function SpecPage({ id }: { id: string; view: 'documento' | 'quadro' | 'plano'; version: number }) {
  return <PageHeader eyebrow="Spec" title="Spec" />;
}
