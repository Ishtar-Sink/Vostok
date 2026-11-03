import { describe, expect, it } from 'vitest';
import { formatDatesIn, formatDateTime } from './api';

const now = new Date(2026, 10, 3, 18, 0);
const at = (day: number, hours: number, minutes: number) => new Date(2026, 10, day, hours, minutes).toISOString();

describe('formatDateTime', () => {
  it('mostra hoje e ontem com a hora local', () => {
    expect(formatDateTime(at(3, 14, 32), now)).toBe('hoje, 14:32');
    expect(formatDateTime(at(2, 9, 5), now)).toBe('ontem, 09:05');
  });

  it('mostra a data completa nos outros dias', () => {
    expect(formatDateTime(at(1, 8, 0), now)).toBe('01/11/2026, 08:00');
  });

  it('mostra só o dia quando o valor não tem hora', () => {
    expect(formatDateTime('2026-11-03', now)).toBe('hoje');
    expect(formatDateTime('2026-10-20', now)).toBe('20/10/2026');
  });

  it('mostra um traço quando não há data', () => {
    expect(formatDateTime('', now)).toBe('—');
  });
});

describe('formatDatesIn', () => {
  it('formata as datas ISO dentro de um texto', () => {
    const iso = at(3, 12, 46).replace(/\.\d{3}Z$/, 'Z');
    expect(formatDatesIn(`aprovada em ${iso}; modelo: sonnet`, now)).toBe('aprovada em hoje, 12:46; modelo: sonnet');
  });
});
