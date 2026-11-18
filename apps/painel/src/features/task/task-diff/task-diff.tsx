import { Fragment, useEffect, useState } from 'react';
import { useResource, type ChangedFile, type TaskChanges } from '../../../shared/api';
import { Badge, type BadgeTone } from '../../../ui/badge/badge';
import { EmptyState } from '../../../ui/empty-state/empty-state';
import { ErrorState } from '../../../ui/error-state/error-state';
import { LoadingState } from '../../../ui/loading-state/loading-state';
import { ViewSwitcher } from '../../../ui/view-switcher/view-switcher';
import styles from './task-diff.module.css';

const FORMAT_KEY = 'vostok.diff.formato';
const POLL_MS = 5000;

type DiffFormat = 'lado-a-lado' | 'unificada';

function SideBySideIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
      <rect x="3" y="4" width="8" height="16" rx="1.5" />
      <rect x="13" y="4" width="8" height="16" rx="1.5" />
    </svg>
  );
}

function UnifiedIcon() {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
      <path d="M4 6h16M4 12h16M4 18h16" />
    </svg>
  );
}

const FORMATS = [
  { value: 'lado-a-lado', label: 'Lado a lado', icon: <SideBySideIcon /> },
  { value: 'unificada', label: 'Unificada', icon: <UnifiedIcon /> },
];

function readFormat(): DiffFormat {
  try {
    const value = localStorage.getItem(FORMAT_KEY);
    return FORMATS.some((format) => format.value === value) ? (value as DiffFormat) : 'lado-a-lado';
  } catch {
    return 'lado-a-lado';
  }
}

function saveFormat(value: DiffFormat) {
  try {
    localStorage.setItem(FORMAT_KEY, value);
  } catch {
    // sem armazenamento disponível, a escolha vale só até recarregar
  }
}

const REASON_MESSAGES = {
  nao_iniciada: { title: 'A tarefa ainda não começou', description: 'As mudanças aparecem aqui quando a execução da tarefa começar.' },
  nao_gravada: { title: 'A diff não foi gravada', description: 'Esta tarefa foi concluída antes de o painel guardar as mudanças na conclusão.' },
  sem_base: { title: 'Sem base de comparação', description: 'A tarefa não tem o commit de início gravado, ou ele não está no repositório. Sem ele, não dá para separar o que esta tarefa mudou.' },
  sem_mudancas: { title: 'Nada mudou desde a base', description: 'Nenhum arquivo da tarefa mudou desde o commit de início.' },
};

const ACTION_TONES: Record<ChangedFile['action'], BadgeTone> = { criado: 'success', removido: 'danger', binário: 'neutral', alterado: 'info' };

type LineKind = 'add' | 'del' | 'ctx';
interface DiffLine { kind: LineKind; oldNumber: number | null; newNumber: number | null; text: string; parts?: WordPart[] }
interface UnifiedRow extends DiffLine { sign: string }
interface SideRow { left: DiffLine | null; right: DiffLine | null }

const HUNK_HEADER = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

export interface WordPart { text: string; changed: boolean }
export interface WordMarks { removed: WordPart[][]; added: WordPart[][] }

const TOKEN = /\s+|[\p{L}\p{N}_]+|[^\p{L}\p{N}_\s]/gu;
const WORD = /^[\p{L}\p{N}_]+$/u;
const MAX_PIECES = 2;
const SINGLE_LINE_SIMILARITY = 0.4;
const BLOCK_SIMILARITY = 0.6;

function wordTokens(text: string): string[] {
  return text.match(TOKEN) ?? [];
}

const isSpace = (token: string) => /^\s+$/.test(token);

/** Marca como casado cada token sem espaço que entra na maior subsequência comum das duas linhas. */
function matchTokens(left: string[], right: string[]): [boolean[], boolean[]] {
  const dp = Array.from({ length: left.length + 1 }, () => new Array<number>(right.length + 1).fill(0));
  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      dp[i][j] = left[i] === right[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const leftMatched = new Array<boolean>(left.length).fill(false);
  const rightMatched = new Array<boolean>(right.length).fill(false);
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) {
      leftMatched[i] = true;
      rightMatched[j] = true;
      i += 1;
      j += 1;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      i += 1;
    } else {
      j += 1;
    }
  }
  return [leftMatched, rightMatched];
}

/** Partes da linha inteira e o número de pedaços trocados: o espaço entre dois tokens trocados entra no pedaço. */
function lineParts(tokens: string[], matched: boolean[]): { parts: WordPart[]; pieces: number } {
  const solid = tokens.map((token, index) => ({ token, index })).filter(({ token }) => !isSpace(token));
  const changedAt = new Array<boolean>(tokens.length).fill(false);
  let pieces = 0;
  solid.forEach(({ index }, position) => {
    if (matched[position]) return;
    changedAt[index] = true;
    const previous = solid[position - 1];
    if (previous && !matched[position - 1]) {
      for (let between = previous.index + 1; between < index; between += 1) changedAt[between] = true;
    } else {
      pieces += 1;
    }
  });
  const parts: WordPart[] = [];
  tokens.forEach((token, index) => {
    const last = parts[parts.length - 1];
    if (last && last.changed === changedAt[index]) last.text += token;
    else parts.push({ text: token, changed: changedAt[index] });
  });
  return { parts, pieces };
}

function comparePair(removed: string, added: string, minimum: number): [WordPart[], WordPart[]] | null {
  const removedTokens = wordTokens(removed);
  const addedTokens = wordTokens(added);
  const removedSolid = removedTokens.filter((token) => !isSpace(token));
  const addedSolid = addedTokens.filter((token) => !isSpace(token));
  const [removedMatched, addedMatched] = matchTokens(removedSolid, addedSolid);
  const words = (solid: string[]) => solid.filter((token) => WORD.test(token)).length;
  const common = removedSolid.filter((token, index) => removedMatched[index] && WORD.test(token)).length;
  const similarity = common / Math.max(words(removedSolid), words(addedSolid), 1);
  const left = lineParts(removedTokens, removedMatched);
  const right = lineParts(addedTokens, addedMatched);
  const passes = similarity >= minimum && left.pieces <= MAX_PIECES && right.pieces <= MAX_PIECES;
  return passes ? [left.parts, right.parts] : null;
}

/** Negrito de uma troca: só com o mesmo número de linhas dos dois lados e todos os pares quase iguais (RF-02). */
export function wordMarks(removed: string[], added: string[]): WordMarks | null {
  if (removed.length === 0 || removed.length !== added.length) return null;
  const minimum = removed.length === 1 ? SINGLE_LINE_SIMILARITY : BLOCK_SIMILARITY;
  const marks: WordMarks = { removed: [], added: [] };
  for (let index = 0; index < removed.length; index += 1) {
    const pair = comparePair(removed[index], added[index], minimum);
    if (!pair) return null;
    marks.removed.push(pair[0]);
    marks.added.push(pair[1]);
  }
  return marks;
}

export function parseHunk(hunk: string): DiffLine[] {
  const [header, ...body] = hunk.split('\n');
  const match = HUNK_HEADER.exec(header);
  let oldNumber = match ? Number(match[1]) : 1;
  let newNumber = match ? Number(match[2]) : 1;
  const lines: DiffLine[] = [];
  for (const raw of body) {
    if (raw.startsWith('\\')) continue;
    const mark = raw[0];
    const text = raw.slice(1);
    if (mark === '+') lines.push({ kind: 'add', oldNumber: null, newNumber: newNumber++, text });
    else if (mark === '-') lines.push({ kind: 'del', oldNumber: oldNumber++, newNumber: null, text });
    else lines.push({ kind: 'ctx', oldNumber: oldNumber++, newNumber: newNumber++, text });
  }
  markSwaps(lines);
  return lines;
}

/** Grava as partes em negrito em cada troca (removidas seguidas das acrescentadas logo depois) que passa na regra. */
function markSwaps(lines: DiffLine[]) {
  let index = 0;
  while (index < lines.length) {
    const removed: DiffLine[] = [];
    while (index < lines.length && lines[index].kind === 'del') removed.push(lines[index++]);
    const added: DiffLine[] = [];
    while (index < lines.length && lines[index].kind === 'add') added.push(lines[index++]);
    if (removed.length === 0 && added.length === 0) {
      index += 1;
      continue;
    }
    const marks = wordMarks(removed.map((line) => line.text), added.map((line) => line.text));
    if (!marks) continue;
    removed.forEach((line, position) => (line.parts = marks.removed[position]));
    added.forEach((line, position) => (line.parts = marks.added[position]));
  }
}

function LineText({ line }: { line: DiffLine }) {
  if (!line.parts) return <>{line.text}</>;
  const Mark = line.kind === 'del' ? 'del' : 'ins';
  const markClass = line.kind === 'del' ? styles.wordDel : styles.wordAdd;
  return (
    <>
      {line.parts.map((part, index) =>
        part.changed && part.text.trim() ? (
          <Mark key={index} className={markClass}>
            {part.text}
          </Mark>
        ) : (
          part.text
        ),
      )}
    </>
  );
}

const SIGNS: Record<LineKind, string> = { add: '+', del: '−', ctx: '' };

export function unifiedRows(hunk: string): UnifiedRow[] {
  return parseHunk(hunk).map((line) => ({ ...line, sign: SIGNS[line.kind] }));
}

export function sideBySideRows(hunk: string): SideRow[] {
  const lines = parseHunk(hunk);
  const rows: SideRow[] = [];
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    if (line.kind === 'ctx') {
      rows.push({ left: line, right: line });
      index += 1;
      continue;
    }
    const removed: DiffLine[] = [];
    const added: DiffLine[] = [];
    while (index < lines.length && lines[index].kind === 'del') removed.push(lines[index++]);
    while (index < lines.length && lines[index].kind === 'add') added.push(lines[index++]);
    for (let row = 0; row < Math.max(removed.length, added.length); row += 1) {
      rows.push({ left: removed[row] ?? null, right: added[row] ?? null });
    }
  }
  return rows;
}

function Cells({ line, side }: { line: DiffLine | null; side: 'old' | 'new' }) {
  const number = line ? (side === 'old' ? line.oldNumber : line.newNumber) : null;
  const kind = line && line.kind !== 'ctx' ? line.kind : undefined;
  return (
    <>
      <td className={styles.number} data-diff={kind}>{number}</td>
      <td className={styles.sign} data-diff={kind}>{line ? SIGNS[line.kind] : ''}</td>
      <td className={styles.code} data-diff={kind}>{line ? <LineText line={line} /> : null}</td>
    </>
  );
}

/** Colunas de largura fixa: todos os trechos dividem a largura do mesmo jeito. */
const COLUMNS: Record<DiffFormat, ('number' | 'sign' | 'code')[]> = {
  unificada: ['number', 'number', 'sign', 'code'],
  'lado-a-lado': ['number', 'sign', 'code', 'number', 'sign', 'code'],
};

const COLUMN_CLASS = { number: styles.colNumber, sign: styles.colSign, code: styles.colCode };

function Columns({ format }: { format: DiffFormat }) {
  return (
    <colgroup>
      {COLUMNS[format].map((kind, index) => (
        <col key={index} data-col={kind} className={COLUMN_CLASS[kind]} />
      ))}
    </colgroup>
  );
}

function HunkTable({ hunk, format }: { hunk: string; format: DiffFormat }) {
  return (
    <div className={styles.scroll}>
      <table className={styles.table}>
        <Columns format={format} />
        <tbody>
          {format === 'unificada'
            ? unifiedRows(hunk).map((row, index) => {
                const kind = row.kind === 'ctx' ? undefined : row.kind;
                return (
                  <tr key={index}>
                    <td className={styles.number} data-diff={kind}>{row.oldNumber}</td>
                    <td className={styles.number} data-diff={kind}>{row.newNumber}</td>
                    <td className={styles.sign} data-diff={kind}>{row.sign}</td>
                    <td className={styles.code} data-diff={kind}>
                      <LineText line={row} />
                    </td>
                  </tr>
                );
              })
            : sideBySideRows(hunk).map((row, index) => (
                <tr key={index}>
                  <Cells line={row.left} side="old" />
                  <Cells line={row.right} side="new" />
                </tr>
              ))}
        </tbody>
      </table>
    </div>
  );
}

/** Parte a frase nas crases: o texto entre um par de crases é código; crase sem par deixa tudo como texto. */
export function noteParts(note: string): { text: string; code: boolean }[] {
  const pieces = note.split('`');
  if (pieces.length % 2 === 0) return [{ text: note, code: false }];
  return pieces.map((text, index) => ({ text, code: index % 2 === 1 })).filter((part) => part.text !== '');
}

function HunkNote({ note }: { note: string | null | undefined }) {
  if (typeof note !== 'string' || note.trim() === '') return null;
  return (
    <p data-diff-note className={styles.note}>
      {noteParts(note.trim()).map((part, index) => (part.code ? <code key={index}>{part.text}</code> : <Fragment key={index}>{part.text}</Fragment>))}
    </p>
  );
}

function FileBlock({ file, format }: { file: ChangedFile; format: DiffFormat }) {
  return (
    <details data-diff-file={file.path} className={styles.file}>
      <summary className={styles.fileSummary}>
        <code className={styles.path}>{file.path}</code>
        <Badge tone={ACTION_TONES[file.action]}>{file.action}</Badge>
        {file.before ? <Badge tone="warning">já tinha mudanças antes</Badge> : null}
        <span className={styles.counts}>
          <span className={styles.plus}>{`+${file.added}`}</span> <span className={styles.minus}>{`−${file.removed}`}</span>
        </span>
      </summary>
      {file.action === 'binário' || file.hunks.length === 0 ? (
        <p className={styles.binary}>Arquivo binário: sem trechos para mostrar.</p>
      ) : (
        file.hunks.map((hunk, index) => (
          <Fragment key={index}>
            <HunkNote note={file.notes?.[index]} />
            <HunkTable hunk={hunk} format={format} />
          </Fragment>
        ))
      )}
    </details>
  );
}

export function TaskDiff({ spec, task, status, version }: { spec: string; task: string; status: string; version: number }) {
  const [format, setFormat] = useState<DiffFormat>(readFormat);
  const [tick, setTick] = useState(0);
  useEffect(() => {
    if (status === 'done') return undefined;
    const timer = setInterval(() => setTick((value) => value + 1), POLL_MS);
    return () => clearInterval(timer);
  }, [status]);
  const changes = useResource<TaskChanges>(`/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}/mudancas`, version * 100000 + tick);

  if (changes.kind === 'loading') return <LoadingState label="Carregando as mudanças" rows={4} />;
  if (changes.kind === 'error') return <ErrorState title="Não foi possível carregar as mudanças" description={changes.message} retryLabel="Tentar de novo" onRetry={changes.retry} />;

  const { data } = changes;
  const files = data.files ?? [];
  const reason = data.reason ?? (files.length === 0 ? 'sem_mudancas' : null);
  if (reason) return <EmptyState title={REASON_MESSAGES[reason].title} description={REASON_MESSAGES[reason].description} />;

  const added = data.added ?? files.reduce((sum, file) => sum + file.added, 0);
  const removed = data.removed ?? files.reduce((sum, file) => sum + file.removed, 0);
  return (
    <div>
      <div data-diff-summary className={styles.summary}>
        <strong>{files.length === 1 ? '1 arquivo' : `${files.length} arquivos`}</strong>
        <span className={styles.plus}>{`+${added}`}</span>
        <span className={styles.minus}>{`−${removed}`}</span>
        {data.frozen ? <Badge tone="neutral">Congelada na conclusão</Badge> : <Badge tone="info">Ao vivo</Badge>}
        <ViewSwitcher
          label="Formato da diff"
          options={FORMATS}
          value={format}
          onChange={(value) => {
            setFormat(value as DiffFormat);
            saveFormat(value as DiffFormat);
          }}
          className={styles.switcher}
        />
      </div>
      <div className={styles.files}>
        {files.map((file) => (
          <FileBlock key={file.path} file={file} format={format} />
        ))}
      </div>
    </div>
  );
}
