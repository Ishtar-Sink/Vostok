import { listOf, type TaskSummary } from '../../shared/api';
import { onLinkClick } from '../../shared/router';
import { BOARD_COLUMNS } from '../../shared/status';
import styles from './board.module.css';

export interface BoardTask extends TaskSummary {
  spec: string;
}

/** Quadro kanban: uma coluna por etapa do fluxo da tarefa. As tarefas não andam em fila: o cartão
 *  mostra a onda e se as dependências já foram concluídas. */
export function Board({ tasks, showSpec = false }: { tasks: BoardTask[]; showSpec?: boolean }) {
  const done = new Set(tasks.filter((task) => task.status === 'done').map((task) => `${task.spec}/${task.id}`));
  return (
    <div className={styles.board}>
      {BOARD_COLUMNS.map((column) => {
        const cards = tasks.filter((task) => column.statuses.includes(task.status));
        return (
          <section key={column.id} className={styles.column} data-column={column.id} aria-labelledby={`col-${column.id}`}>
            <header className={styles.columnHeader}>
              <h3 id={`col-${column.id}`}>{column.label}</h3>
              <span className={styles.count}>{cards.length}</span>
            </header>
            <ul className={styles.cards}>
              {cards.map((task) => {
                const deps = listOf(task.depends_on);
                const waiting = deps.filter((dep) => !done.has(`${task.spec}/${dep}`));
                const ready = task.status === 'approved' && waiting.length === 0;
                return (
                  <li key={`${task.spec}/${task.id}`}>
                    <a href={`/tarefas/${task.spec}/${task.id}`} onClick={onLinkClick} className={styles.card}>
                      <span className={styles.cardTop}>
                        <span className={styles.id}>{showSpec ? `${task.spec} · ${task.id}` : task.id}</span>
                        <span className={styles.wave}>onda {task.wave}</span>
                      </span>
                      <span className={styles.title}>{task.title}</span>
                      <span className={styles.facts}>
                        <span>{task.profile}</span>
                        <span aria-hidden="true">·</span>
                        <span>{task.persona}</span>
                      </span>
                      {task.status === 'done' ? null : waiting.length > 0 ? (
                        <span className={styles.waiting}>Espera {waiting.join(', ')}</span>
                      ) : ready ? (
                        <span className={styles.ready}>Pronta para executar</span>
                      ) : null}
                    </a>
                  </li>
                );
              })}
              {cards.length === 0 ? <li className={styles.empty}>Nenhuma</li> : null}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
