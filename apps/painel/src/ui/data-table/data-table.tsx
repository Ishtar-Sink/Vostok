import type { ReactNode } from 'react';
import { navigate } from '../../shared/router';
import styles from './data-table.module.css';

export interface Column<T> {
  id: string;
  header: string;
  cell: (row: T) => ReactNode;
  /** Números e datas à direita, com algarismos de largura fixa. */
  align?: 'start' | 'end' | 'center';
  width?: string;
  /** A coluna principal ocupa o espaço livre e vira o topo do cartão no celular. */
  primary?: boolean;
}

export interface DataTableProps<T> {
  caption: string;
  columns: Column<T>[];
  rows: T[];
  rowKey: (row: T) => string;
  /** Destino da linha inteira; a coluna principal leva o link real, para teclado e leitor de tela. */
  rowHref?: (row: T) => string;
  empty?: ReactNode;
}

/** Tabela padrão: cabeçalho discreto, linha inteira clicável, coluna principal com ID e título,
 *  números à direita e, abaixo de 720 px, uma pilha de cartões com rótulo e valor. */
export function DataTable<T>({ caption, columns, rows, rowKey, rowHref, empty }: DataTableProps<T>) {
  if (rows.length === 0 && empty) return <>{empty}</>;
  return (
    <div className={styles.frame}>
      <table className={styles.table}>
        <caption className={styles.caption}>{caption}</caption>
        <colgroup>
          {columns.map((column) => (
            <col key={column.id} style={column.width ? { width: column.width } : undefined} />
          ))}
        </colgroup>
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.id} scope="col" data-align={column.align ?? 'start'}>
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const href = rowHref?.(row);
            return (
              <tr
                key={rowKey(row)}
                data-link={href ? '' : undefined}
                onClick={(event) => {
                  if (!href || (event.target as HTMLElement).closest('a, button')) return;
                  navigate(href);
                }}
              >
                {columns.map((column) => (
                  <td key={column.id} data-align={column.align ?? 'start'} data-primary={column.primary ? '' : undefined} data-label={column.header}>
                    {column.cell(row)}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

/** Célula principal: o ID em monoespaçada acima do título, com o link da linha. */
export function PrimaryCell({ id, title, href, onClick }: { id: string; title: string; href?: string; onClick?: (event: React.MouseEvent<HTMLAnchorElement>) => void }) {
  const content = (
    <>
      <span className={styles.id}>{id}</span>
      <span className={styles.title}>{title}</span>
    </>
  );
  return href ? (
    <a href={href} onClick={onClick} className={styles.primaryLink}>
      {content}
    </a>
  ) : (
    <span className={styles.primaryLink}>{content}</span>
  );
}

/** Progresso compacto: barra fina e a contagem. */
export function ProgressCell({ done, total }: { done: number; total: number }) {
  if (total === 0) return <span className={styles.muted}>—</span>;
  return (
    <span className={styles.progress} title={`${done} de ${total} concluídas`}>
      <span className={styles.track} aria-hidden="true">
        <span className={styles.fill} style={{ width: `${(done / total) * 100}%` }} />
      </span>
      <span className={styles.count}>
        {done}/{total}
      </span>
    </span>
  );
}
