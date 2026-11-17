import type { CheckItem } from '../../../ui/check-list/check-list';
import { failureReason, findLines, snippetsOf, unsearchable } from '../snippet-match';

export { snippetsOf, unsearchable };

export interface SnippetCheck extends CheckItem {
  /** Índices das linhas do log em que o texto aparece, a partir de 0. */
  lines: number[];
}

/** Resultado de cada texto: bom quando aparece no Trecho (`expectFound`) ou quando falta na Falha. */
export function checksOf(log: string, snippets: string[], expectFound: boolean): SnippetCheck[] {
  return snippets.map((snippet, index) => {
    const id = String(index);
    const reason = unsearchable(snippet);
    if (reason) return { id, label: snippet, result: `não dá para buscar: ${reason}`, tone: 'bad', lines: [] };
    const lines = findLines(log, snippet);
    const found = lines.length > 0;
    return {
      id,
      label: snippet,
      result: found ? `apareceu · ${lines.length} ${lines.length === 1 ? 'linha' : 'linhas'}` : 'não apareceu',
      tone: found === expectFound ? 'good' : 'bad',
      lines,
    };
  });
}

/** Resultado da Falha: o texto que repete o Trecho ou tem condição não é buscado e fica em tom neutro. */
export function failureChecksOf(log: string, failure: string | undefined, excerpt: string | undefined): SnippetCheck[] {
  const snippets = snippetsOf(failure);
  return checksOf(log, snippets, false).map((check, index) => {
    const reason = failureReason(excerpt, failure ?? '', snippets[index]);
    if (!reason) return check;
    return { id: check.id, label: check.label, result: `não dá para buscar: ${reason}`, tone: 'neutral', lines: [] };
  });
}

export const isVisual = (label: string) => label === 'Print' || label === 'Vídeo';
