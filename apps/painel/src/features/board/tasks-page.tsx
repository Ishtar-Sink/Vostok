import { useState } from 'react';
import { useSearchParam } from '../../shared/router';
import { Button } from '../../ui/button/button';
import { useResource, type Overview } from '../../shared/api';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import { PageHeader } from '../../ui/page-header/page-header';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ViewSwitcher, type ViewOption } from '../../ui/view-switcher/view-switcher';
import { BlocksIcon, ListIcon } from '../../ui/view-switcher/view-icons';
import { Board, type BoardGroup, type BoardOrder, type BoardView } from './board';
import styles from './board.module.css';

const GROUPS: { value: BoardGroup; label: string }[] = [{ value: 'etapa', label: 'Etapa' }, { value: 'spec', label: 'Spec' }, { value: 'onda', label: 'Onda' }];
const VIEWS: ViewOption[] = [{ value: 'lista', label: 'Lista', icon: <ListIcon /> }, { value: 'blocos', label: 'Blocos', icon: <BlocksIcon /> }];
const ORDERS: { value: BoardOrder; label: string }[] = [{ value: 'numero', label: 'Número' }, { value: 'spec', label: 'Spec' }, { value: 'onda', label: 'Onda' }];

function readChoice(key: string, allowed: string[] | null, fallback: string): string {
  try {
    const value = localStorage.getItem(`vostok.quadro.${key}`);
    return value === null || (allowed && !allowed.includes(value)) ? fallback : value;
  } catch {
    return fallback;
  }
}

function saveChoice(key: string, value: string) {
  try {
    localStorage.setItem(`vostok.quadro.${key}`, value);
  } catch {
    // sem armazenamento disponível, a escolha vale só até recarregar
  }
}

/** Opções do filtro, sempre com o valor escolhido para o filtro guardado continuar visível. */
function withSelected(values: string[], selected: string): string[] {
  return selected === 'todas' || values.includes(selected) ? values : [...values, selected];
}

function Select({ label, value, options, onChange }: { label: string; value: string; options: { value: string; label: string }[]; onChange: (value: string) => void }) {
  return (
    <label className={styles.control}>
      {label}
      <span className={styles.selectWrap}>
        <select value={value} onChange={(event) => onChange((event.target as HTMLSelectElement).value)}>
          {options.map((option) => (
            <option key={option.value} value={option.value}>{option.label}</option>
          ))}
        </select>
        <svg className={styles.chevron} viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
          <path d="m6 9 6 6 6-6" />
        </svg>
      </span>
    </label>
  );
}

export function TasksPage({ version }: { version: number }) {
  const overview = useResource<Overview>('/api/visao', version);
  const [specParam, setSpecParam] = useSearchParam('spec');
  const spec = specParam ?? 'todas';
  const setSpec = (value: string) => setSpecParam(value === 'todas' ? null : value);
  const [visualizacao, setVisualizacao] = useState(() => readChoice('visualizacao', VIEWS.map((item) => item.value), 'lista'));
  const [agrupamento, setAgrupamento] = useState(() => readChoice('agrupamento', GROUPS.map((item) => item.value), 'etapa'));
  const [ordem, setOrdem] = useState(() => readChoice('ordem', ORDERS.map((item) => item.value), 'numero'));
  const [onda, setOnda] = useState(() => readChoice('onda', null, 'todas'));
  const [persona, setPersona] = useState(() => readChoice('persona', null, 'todas'));
  const choose = (key: string, set: (value: string) => void) => (value: string) => { set(value); saveChoice(key, value); };
  if (overview.kind === 'loading') return <LoadingState label="Carregando as tarefas" rows={6} />;
  if (overview.kind === 'error') return <ErrorState title="Não foi possível ler o harness" description={overview.message} retryLabel="Tentar de novo" onRetry={overview.retry} />;

  const specs = overview.data.specs.filter((item) => item.tasks.length > 0);
  const chosenSpec = specs.find((item) => item.meta.id === spec);
  const everyTask = specs.flatMap((item) => item.tasks);
  const waves = withSelected([...new Set(everyTask.map((task) => task.wave).filter(Boolean))].sort((a, b) => Number(a) - Number(b)), onda);
  const personas = withSelected([...new Set(everyTask.map((task) => task.persona).filter(Boolean))].sort(), persona);
  const tasks = specs
    .filter((item) => spec === 'todas' || item.meta.id === spec)
    .flatMap((item) => item.tasks.map((task) => ({ ...task, spec: item.meta.id })))
    .filter((task) => (onda === 'todas' || task.wave === onda) && (persona === 'todas' || task.persona === persona));

  const specFilters = (
    <div className={styles.filters} role="group" aria-label="Filtrar por spec">
      <button type="button" aria-pressed={spec === 'todas'} onClick={() => setSpec('todas')}>
        Todas <span>{specs.reduce((sum, item) => sum + item.tasks.length, 0)}</span>
      </button>
      {specs.map((item) => (
        <button key={item.meta.id} type="button" aria-pressed={spec === item.meta.id} onClick={() => setSpec(item.meta.id)} title={item.meta.title}>
          {item.meta.id} <span>{item.tasks.length}</span>
        </button>
      ))}
    </div>
  );
  const header = (
    <PageHeader
      eyebrow="Tarefas"
      title={spec === 'todas' ? 'Quadro de tarefas' : `Quadro de tarefas · ${spec}`}
      subtitle={chosenSpec ? chosenSpec.meta.title : 'Todas as tarefas das specs abertas, pela etapa em que estão.'}
    />
  );

  if (spec !== 'todas' && !chosenSpec) {
    return (
      <>
        {header}
        {specFilters}
        <EmptyState title="Spec não encontrada" description={`A ${spec} não está entre as specs abertas com tarefas.`} action={<Button variant="secondary" onClick={() => setSpec('todas')}>Ver todas as specs</Button>} />
      </>
    );
  }

  return (
    <>
      {header}
      {specFilters}
      <div className={styles.controls} role="group" aria-label="Organizar o quadro">
        <div className={`${styles.control} ${styles.viewControl}`}>
          <span aria-hidden="true">Visualização</span>
          <ViewSwitcher label="Visualização" options={VIEWS} value={visualizacao} onChange={choose('visualizacao', setVisualizacao)} />
        </div>
        {visualizacao === 'lista' ? <Select label="Agrupar por" value={agrupamento} options={GROUPS} onChange={choose('agrupamento', setAgrupamento)} /> : null}
        <Select label="Ordenar por" value={ordem} options={ORDERS} onChange={choose('ordem', setOrdem)} />
        <Select label="Onda" value={onda} options={[{ value: 'todas', label: 'Todas' }, ...waves.map((wave) => ({ value: wave, label: `Onda ${wave}` }))]} onChange={choose('onda', setOnda)} />
        <Select label="Persona" value={persona} options={[{ value: 'todas', label: 'Todas' }, ...personas.map((item) => ({ value: item, label: item }))]} onChange={choose('persona', setPersona)} />
      </div>
      {tasks.length === 0 ? (
        <EmptyState title="Nenhuma tarefa com esses filtros" description="Troque a spec, a onda ou a persona para ver outras tarefas." />
      ) : (
        <Board tasks={tasks} showSpec={spec === 'todas'} view={visualizacao as BoardView} group={visualizacao === 'blocos' ? 'etapa' : (agrupamento as BoardGroup)} order={ordem as BoardOrder} />
      )}
    </>
  );
}
