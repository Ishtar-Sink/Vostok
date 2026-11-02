import { marked, type Token, type Tokens } from 'marked';
import type { TaskSummary } from './api';

export interface Reference {
  id: string;
  /** Seção da spec onde a referência é definida (ex.: "Requisitos funcionais"). */
  section: string;
  /** Pares rótulo e valor da linha da tabela, ou o texto do item. */
  fields: [string, string][];
  text?: string;
  task?: { spec: string; status: string };
}

export type ReferenceIndex = Map<string, Reference>;

const ID = /^(?:[A-Z]{1,4}-\d{2,3}|TASK-\d{2})$/;
const LEADING_ID = /^\s*(?:\[[ x]\]\s*)?\*{0,2}([A-Z]{1,4}-\d{2,3})\*{0,2}\s*[:(—-]\s*/;

function plain(text: string): string {
  return text.replace(/`/g, '').replace(/\*\*/g, '').trim();
}

/** Índice das definições da spec: linhas de tabela que começam por um ID (RF-01, CB-02...) e itens
 *  de lista que começam por um ID (CA-01: ...). As tarefas entram pelo título e pelo status. */
export function buildReferences(markdown: string, tasks: TaskSummary[] = [], spec = ''): ReferenceIndex {
  const index: ReferenceIndex = new Map();
  let section = '';
  const visit = (tokens: Token[]) => {
    for (const token of tokens) {
      if (token.type === 'heading') section = plain((token as Tokens.Heading).text);
      if (token.type === 'table') {
        const table = token as Tokens.Table;
        for (const row of table.rows) {
          const id = plain(row[0]?.text ?? '');
          if (!ID.test(id) || index.has(id)) continue;
          index.set(id, { id, section, fields: table.header.slice(1).map((cell, column) => [plain(cell.text), plain(row[column + 1]?.text ?? '')]) });
        }
      }
      if (token.type === 'list') {
        for (const item of (token as Tokens.List).items) {
          const match = LEADING_ID.exec(item.text);
          if (match && !index.has(match[1])) {
            index.set(match[1], { id: match[1], section, fields: [], text: plain(item.text.slice(match[0].length)) });
          }
        }
      }
    }
  };
  visit(marked.lexer(markdown));
  for (const task of tasks) {
    index.set(task.id, { id: task.id, section: 'Tarefa', fields: [['Onda', task.wave], ['Perfil', task.profile], ['Cobre', task.covers.replace(/[[\]]/g, '')]], text: task.title, task: { spec, status: task.status } });
  }
  return index;
}

/** Envolve, no HTML já renderizado, cada ID conhecido num botão de referência. Não mexe em
 *  títulos, blocos de código nem na própria célula que define o ID. */
export function linkReferences(root: HTMLElement, index: ReferenceIndex) {
  if (index.size === 0) return;
  const pattern = new RegExp(`\\b(${[...index.keys()].map((id) => id.replace('-', '\\-')).join('|')})\\b`, 'g');
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode(node) {
      const parent = node.parentElement;
      if (!parent || parent.closest('h1, h2, h3, h4, pre, button, a')) return NodeFilter.FILTER_REJECT;
      const cell = parent.closest('td');
      if (cell && cell === cell.parentElement?.firstElementChild) return NodeFilter.FILTER_REJECT;
      return pattern.test(node.textContent ?? '') ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT;
    },
  });
  const nodes: Text[] = [];
  while (walker.nextNode()) nodes.push(walker.currentNode as Text);
  for (const node of nodes) {
    const text = node.textContent ?? '';
    const fragment = document.createDocumentFragment();
    let last = 0;
    pattern.lastIndex = 0;
    for (const match of text.matchAll(pattern)) {
      fragment.append(text.slice(last, match.index));
      const button = document.createElement('button');
      button.type = 'button';
      button.className = 'ref';
      button.dataset.ref = match[1];
      button.setAttribute('aria-haspopup', 'dialog');
      button.textContent = match[1];
      fragment.append(button);
      last = (match.index ?? 0) + match[1].length;
    }
    fragment.append(text.slice(last));
    node.replaceWith(fragment);
  }
}
