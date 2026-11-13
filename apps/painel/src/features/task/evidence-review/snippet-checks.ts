import type { CheckItem } from '../../../ui/check-list/check-list';
import { findLines } from '../snippet-match';

export interface SnippetCheck extends CheckItem {
  /** Índices das linhas do log em que o texto aparece, a partir de 0. */
  lines: number[];
}

/** Textos entre crases do Trecho ou da Falha, na ordem em que aparecem. */
export function snippetsOf(value?: string): string[] {
  return [...(value ?? '').matchAll(/`([^`]+)`/g)].map((match) => match[1]);
}

const ASCII_PUNCTUATION = /^[!-/:-@[-`{-~]+$/;

/** Motivo de um texto não ser buscado, ou vazio quando dá para buscar. */
export function unsearchable(snippet: string): string {
  // a barra na ponta é o que sobra de uma crase escapada, que cortou o texto no meio
  if (snippet.startsWith('\\') || snippet.endsWith('\\')) return 'crase dentro do texto';
  // pontuação solta, como `+` ou `|`, acharia quase qualquer linha do log
  if (ASCII_PUNCTUATION.test(snippet.replace(/\s+/g, ''))) return 'símbolo solto';
  return '';
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

export const isVisual = (label: string) => label === 'Print' || label === 'Vídeo';

/** Por que o Trecho ou a Falha não têm textos para conferir. */
export function notApplicable(field: 'excerpt' | 'failure', label: string): string {
  if (isVisual(label)) {
    return field === 'excerpt'
      ? 'Prints e vídeos não têm texto para procurar: confira olhando, como diz "Como ler".'
      : 'Prints e vídeos não têm texto para procurar: a falha é o que "Como ler" descreve não aparecer.';
  }
  if (field === 'excerpt') return 'O planejamento não definiu um texto esperado para esta evidência.';
  if (label === 'Diff') return 'Num diff, a falha é a mudança esperada não aparecer, e o Trecho esperado já confere isso.';
  return 'O planejamento não definiu sinais de falha para esta evidência.';
}
