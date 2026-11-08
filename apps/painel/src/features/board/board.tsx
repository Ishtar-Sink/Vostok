import { useState } from 'react';
import { listOf, type TaskSummary } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { BOARD_SECTIONS } from '../../shared/status';
import { Badge } from '../../ui/badge/badge';
import styles from './board.module.css';

/** Cor de status das colunas do fluxo; as de entrada (para aprovar e aprovadas) ficam sem cor. */
const COLUMN_TONE: Record<string, 'info' | 'accent' | 'warning' | 'success'> = { ready: 'success', running: 'info', validating: 'accent', blocked: 'warning', done: 'success' };

export interface BoardTask extends TaskSummary {
  spec: string;
}

interface WaveGroup {
  spec: string;
  wave: string;
  cards: BoardTask[];
}

/** Cartões aprovados por spec e onda, na ordem das ondas. */
function waveGroups(cards: BoardTask[]): WaveGroup[] {
  const groups = new Map<string, WaveGroup>();
  for (const task of cards) {
    const key = `${task.spec}/${task.wave}`;
    const group = groups.get(key) ?? { spec: task.spec, wave: task.wave, cards: [] };
    group.cards.push(task);
    groups.set(key, group);
  }
  return [...groups.values()].sort((a, b) => a.spec.localeCompare(b.spec) || Number(a.wave === '') - Number(b.wave === '') || Number(a.wave) - Number(b.wave));
}

/** Quadro kanban só de leitura: uma seção por etapa do fluxo da tarefa; as aprovadas ficam agrupadas por onda. */
export function Board({ tasks, showSpec = false }: { tasks: BoardTask[]; showSpec?: boolean }) {
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => `${task.spec}/${task.id}`));
  const isReady = (task: BoardTask) => task.status === 'approved' && listOf(task.depends_on).every((dep) => done.has(`${task.spec}/${dep}`));
  const running = (group: WaveGroup) => tasks.some((task) => task.spec === group.spec && task.wave === group.wave && (task.status === 'in_progress' || task.status === 'validating'));
  const inSection = (task: BoardTask, id: string) => (id === 'ready' ? isReady(task) : id === 'approved' ? !isReady(task) : true);
  const [doneOpen, setDoneOpen] = useState(false);

  const renderCard = (task: BoardTask) => {
    const deps = listOf(task.depends_on);
    const waiting = deps.filter((dep) => !done.has(`${task.spec}/${dep}`));
    const facts = [task.profile, task.persona].filter(Boolean);
    return (
      <li key={`${task.spec}/${task.id}`} className={styles.cardItem}>
        <a href={`/tarefas/${task.spec}/${task.id}`} onClick={onLinkClick} className={styles.card}>
          <span className={styles.cardTop}>
            <span className={styles.id}>{showSpec ? `${task.spec} · ${task.id}` : task.id}</span>
            {task.wave ? <span className={styles.wave}>onda {task.wave}</span> : null}
          </span>
          <span className={styles.title}>{task.title}</span>
          {facts.length > 0 ? <span className={styles.facts}>{facts.join(' · ')}</span> : null}
          {task.status !== 'done' && waiting.length > 0 ? <span className={styles.waiting}>Espera {waiting.join(', ')}</span> : null}
        </a>
      </li>
    );
  };

  return (
      <div className={styles.board}>
        {BOARD_SECTIONS.map((column) => {
          const cards = tasks.filter((task) => column.statuses.includes(task.status) && inSection(task, column.id));
          return (
            <section key={column.id} className={styles.column} data-column={column.id} aria-labelledby={`col-${column.id}`}>
              <header className={styles.columnHeader}>
                <h3 id={`col-${column.id}`}>
                  {column.id === 'done' && cards.length > 0 ? (
                    <button type="button" className={styles.sectionToggle} aria-expanded={doneOpen} onClick={() => setDoneOpen((open) => !open)}>
                      {COLUMN_TONE[column.id] ? <span className={styles.dot} data-tone={COLUMN_TONE[column.id]} aria-hidden="true" /> : null}
                      {column.label}
                    </button>
                  ) : (
                    <>
                      {COLUMN_TONE[column.id] ? <span className={styles.dot} data-tone={COLUMN_TONE[column.id]} aria-hidden="true" /> : null}
                      {column.label}
                    </>
                  )}
                </h3>
                <span className={styles.count}>{cards.length}</span>
              </header>
              {column.id === 'done' && cards.length > 0 && !doneOpen ? null : (
                <ul className={styles.cards}>
                  {column.id === 'approved' || column.id === 'ready'
                    ? waveGroups(cards).map((group) => {
                        const name = `${showSpec ? `${group.spec} · ` : ''}${group.wave ? `Onda ${group.wave}` : 'Sem onda'}`;
                        return (
                          <li key={`${group.spec}/${group.wave}`}>
                            <div role="group" aria-label={name} className={styles.waveGroup}>
                              <div className={styles.waveHeader}>
                                <span className={styles.waveTitle}>{name}</span>
                                {group.wave && running(group) ? <Badge tone="info">em execução</Badge> : null}
                              </div>
                              <ul className={styles.cards}>{group.cards.map(renderCard)}</ul>
                            </div>
                          </li>
                        );
                      })
                    : cards.map(renderCard)}
                  {cards.length === 0 ? <li className={styles.empty}>Nenhuma</li> : null}
                </ul>
              )}
            </section>
          );
        })}
      </div>
  );
}
