import { useMemo, useState } from 'react';
import { formatDateTime, type TaskDetail } from '../../../shared/api';
import { Badge, type BadgeTone } from '../../../ui/badge/badge';
import { ListIcon } from '../../../ui/view-switcher/view-icons';
import { ViewSwitcher } from '../../../ui/view-switcher/view-switcher';
import { CATEGORIES, type EvidenceCategory, type EvidenceEntry, evidenceEntries } from './evidence-items';
import { GridIcon } from './grid-icon';
import styles from './evidence-list.module.css';

const STORAGE_KEY = 'vostok.evidencias.visualizacao';

const VIEWS = [
  { value: 'lista', label: 'Lista', icon: <ListIcon /> },
  { value: 'cards', label: 'Cards', icon: <GridIcon /> },
];

function readView(): string {
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    return VIEWS.some((view) => view.value === value) ? (value as string) : 'lista';
  } catch {
    return 'lista';
  }
}

function saveView(value: string) {
  try {
    localStorage.setItem(STORAGE_KEY, value);
  } catch {
    // sem armazenamento disponível, a escolha vale só até recarregar
  }
}

const STATE_BADGE: Record<string, { label: string; tone: BadgeTone }> = {
  'a conferir': { label: 'A conferir', tone: 'warning' },
  recusada: { label: 'Recusada', tone: 'danger' },
  conferida: { label: 'Conferida', tone: 'success' },
};
const NO_REVIEW = { label: 'Sem aval', tone: 'neutral' as BadgeTone };

function StateBadge({ entry }: { entry: EvidenceEntry }) {
  const state = (entry.review && STATE_BADGE[entry.review.status]) || NO_REVIEW;
  return (
    <Badge tone={state.tone} className={styles.state}>
      {state.label}
    </Badge>
  );
}

function reviewedAt(entry: EvidenceEntry): string {
  return entry.review?.reviewed_at ? formatDateTime(entry.review.reviewed_at) : '—';
}

/** Nome do arquivo sob o título; some quando o título já é o nome. */
function fileLine(entry: EvidenceEntry): string {
  const missing = entry.manual && !entry.manual.attached ? 'falta anexar' : '';
  const name = entry.title === entry.name ? '' : entry.name;
  return [name, missing].filter(Boolean).join(' · ');
}

interface ItemProps {
  entry: EvidenceEntry;
  onOpen?: (name: string) => void;
}

function Row({ entry, onOpen }: ItemProps) {
  const file = fileLine(entry);
  return (
    <button type="button" className={styles.row} aria-haspopup="dialog" onClick={() => onOpen?.(entry.name)}>
      <span className={styles.id}>{entry.id}</span>
      <span className={styles.rowTitle}>
        <span className={styles.titleText}>{entry.title}</span>
        {file && <small>{file}</small>}
      </span>
      <Badge tone="neutral" className={styles.kind}>
        {entry.kindLabel}
      </Badge>
      <StateBadge entry={entry} />
      <span className={styles.date}>{reviewedAt(entry)}</span>
      <span className={styles.arrow} aria-hidden="true">
        ›
      </span>
    </button>
  );
}

function Card({ entry, onOpen }: ItemProps) {
  const file = fileLine(entry);
  return (
    <button type="button" className={styles.card} aria-haspopup="dialog" onClick={() => onOpen?.(entry.name)}>
      <span className={styles.meta}>
        <span className={styles.id}>{entry.id}</span>
        <Badge tone="neutral">{entry.kindLabel}</Badge>
      </span>
      <span className={styles.cardTitle}>{entry.title}</span>
      {file && <small className={styles.cardFile}>{file}</small>}
      <span className={styles.foot}>
        <StateBadge entry={entry} />
        <span className={styles.date}>{reviewedAt(entry)}</span>
      </span>
    </button>
  );
}

export interface EvidenceListProps {
  detail: TaskDetail;
  headingId: string;
  /** Chamado com o nome do arquivo da evidência clicada. */
  onOpen?: (name: string) => void;
}

/** Aba Evidências: resumo por estado, filtro em pills e as categorias em lista ou cards. */
export function EvidenceList({ detail, headingId, onOpen }: EvidenceListProps) {
  const entries = useMemo(() => evidenceEntries(detail), [detail]);
  const [view, setView] = useState(readView);
  const [filters, setFilters] = useState<EvidenceCategory[]>([]);
  const [collapsed, setCollapsed] = useState<EvidenceCategory[]>([]);

  if (entries.length === 0) {
    return (
      <>
        <h2 id={headingId} className={styles.title}>
          Evidências
        </h2>
        <p className={styles.muted}>Elas aparecem aqui quando a tarefa é finalizada.</p>
      </>
    );
  }

  const byCategory = (id: EvidenceCategory) => entries.filter((entry) => entry.category === id);
  const toggleIn = (list: EvidenceCategory[], id: EvidenceCategory) =>
    list.includes(id) ? list.filter((item) => item !== id) : [...list, id];
  const choose = (value: string) => {
    setView(value);
    saveView(value);
  };
  const visible = (filters.length === 0 ? CATEGORIES : CATEGORIES.filter((category) => filters.includes(category.id))).filter(
    (category) => byCategory(category.id).length > 0,
  );

  return (
    <>
      <div className={styles.head}>
        <div className={styles.headStart}>
          <h2 id={headingId} className={styles.title}>
            Evidências
          </h2>
          <div role="group" aria-label="Filtrar por estado" className={styles.summary}>
            {CATEGORIES.map((category) => {
              const count = byCategory(category.id).length;
              return (
                <button
                  key={category.id}
                  type="button"
                  className={styles.pill}
                  data-category={category.id}
                  aria-pressed={filters.includes(category.id)}
                  onClick={() => setFilters((current) => toggleIn(current, category.id))}
                >
                  {`${count} ${count === 1 ? category.one : category.many}`}
                </button>
              );
            })}
          </div>
        </div>
        <ViewSwitcher label="Visualização das evidências" options={VIEWS} value={view} onChange={choose} />
      </div>
      {visible.length === 0 && <p className={styles.muted}>Nenhuma evidência nos estados escolhidos.</p>}
      {visible.map((category) => {
        const items = byCategory(category.id);
        const open = !collapsed.includes(category.id);
        const groupId = `${headingId}-${category.id}`;
        return (
          <section key={category.id} className={styles.group} aria-labelledby={groupId}>
            <h3 id={groupId} className={styles.groupTitle}>
              <button
                type="button"
                aria-expanded={open}
                aria-controls={`${groupId}-itens`}
                onClick={() => setCollapsed((current) => toggleIn(current, category.id))}
              >
                <span className={styles.chevron} aria-hidden="true">
                  ›
                </span>
                {category.label}
                <span className={styles.count}>{items.length}</span>
                {category.hint && <span className={styles.hint}>{category.hint}</span>}
              </button>
            </h3>
            <div id={`${groupId}-itens`} hidden={!open} className={view === 'cards' ? styles.cards : styles.list}>
              {items.map((entry) =>
                view === 'cards' ? (
                  <Card key={entry.name} entry={entry} onOpen={onOpen} />
                ) : (
                  <Row key={entry.name} entry={entry} onOpen={onOpen} />
                ),
              )}
            </div>
          </section>
        );
      })}
    </>
  );
}
