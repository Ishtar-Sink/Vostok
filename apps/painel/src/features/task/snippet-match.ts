/** As quatro primeiras linhas de um log gravado pelo harness: comando, data, commit e código de saída (RN-07). */
const LOG_HEADER = [/^\$ /, /^# data:/, /^# commit:/, /^# código de saída:/];

const ANSI = /\u001b\[[\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e]/g;
const WILDCARD = /\.{3}|…/g;
const BEFORE = '(?<![\\p{L}\\p{N}])';
const AFTER = '(?![\\p{L}\\p{N}])';

const escapeRegExp = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** Texto sem os códigos de cor ANSI, para a busca e para a exibição do log (CB-11). */
export function stripAnsi(text: string): string {
  return text.replace(ANSI, '');
}

const normalize = (text: string) => stripAnsi(text).replace(/\s+/g, ' ').trim();

function headerLength(lines: string[]): number {
  let length = 0;
  while (length < LOG_HEADER.length && LOG_HEADER[length].test(lines[length] ?? '')) length += 1;
  return length;
}

/** Índices das linhas do log que o texto do Trecho ou da Falha aponta, pelas regras da spec (RF-22): diferencia maiúsculas; junta espaços; ignora o cabeçalho e os códigos ANSI; a linha igual vence e, sem ela, vale a que contém o texto como palavra inteira; `...` e `…` valem qualquer trecho da mesma linha, com as partes na ordem e sem limite de palavra junto do curinga. */
export function findLines(log: string, snippet: string): number[] {
  const parts = snippet.split(WILDCARD).map(normalize);
  const wanted = parts.filter(Boolean);
  if (wanted.length === 0) return [];

  const lines = log.split('\n');
  const body = lines
    .map((line, index) => ({ text: normalize(line), index }))
    .slice(headerLength(lines));
  const before = parts[0] ? BEFORE : '';
  const after = parts[parts.length - 1] ? AFTER : '';
  const pattern = new RegExp(before + wanted.map(escapeRegExp).join('.*') + after, 'u');

  const equal = parts.length === 1 ? body.filter((line) => line.text === wanted[0]) : [];
  const found = equal.length > 0 ? equal : body.filter((line) => pattern.test(line.text));
  return found.map((line) => line.index);
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

export const FAILURE_IN_EXCERPT = 'está no Trecho';
export const FAILURE_CONDITION = 'condição sobre o valor';

const CONDITION_WORDS = ['diferente de', 'maior que', 'menor que', 'acima de', 'abaixo de', 'outro', 'outra', 'sem'];
const CLAUSE_BREAK = /[;.,`]/;
const NEIGHBOR = /[\p{L}\p{N}]/u;

function hasConditionWord(clause: string): boolean {
  const lower = clause.toLowerCase();
  return CONDITION_WORDS.some((word) => {
    let at = lower.indexOf(word);
    while (at >= 0) {
      const before = lower[at - 1];
      const after = lower[at + word.length];
      if (!(before && NEIGHBOR.test(before)) && !(after && NEIGHBOR.test(after))) return true;
      at = lower.indexOf(word, at + 1);
    }
    return false;
  });
}

function conditionNear(failure: string, text: string): boolean {
  const quoted = `\`${text}\``;
  for (let start = failure.indexOf(quoted); start >= 0; start = failure.indexOf(quoted, start + 1)) {
    const before = failure.slice(0, start);
    const after = failure.slice(start + quoted.length);
    const clauseBefore = before.slice(Math.max(before.lastIndexOf(';'), before.lastIndexOf('.'), before.lastIndexOf(','), before.lastIndexOf('`')) + 1);
    const breakAfter = after.search(CLAUSE_BREAK);
    const clauseAfter = breakAfter < 0 ? after : after.slice(0, breakAfter);
    const lastWords = clauseBefore.split(/\s+/).filter(Boolean).slice(-3).join(' ');
    const firstWords = clauseAfter.split(/\s+/).filter(Boolean).slice(0, 3).join(' ');
    if (hasConditionWord(lastWords) || hasConditionWord(firstWords)) return true;
  }
  return false;
}

/** Motivo de um texto da Falha ser suspeito: já está no Trecho ou vem com condição sobre o valor; vazio quando dá para buscar (RF-03). */
export function failureReason(excerpt: string | undefined, failure: string, text: string): string {
  if (snippetsOf(excerpt).some((held) => findLines(held, text).length > 0)) return FAILURE_IN_EXCERPT;
  return conditionNear(failure, text) ? FAILURE_CONDITION : '';
}
