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
