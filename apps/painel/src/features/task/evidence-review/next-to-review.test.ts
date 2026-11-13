import { describe, expect, it } from 'vitest';
import type { EvidenceCategory, EvidenceEntry } from '../evidence-list/evidence-items';
import { nextToReview } from './next-to-review';

const item = (name: string, category: EvidenceCategory) => ({ name, category }) as EvidenceEntry;
const LIST = [item('a', 'a-conferir'), item('b', 'conferidas'), item('c', 'a-conferir')];

describe('nextToReview', () => {
  it('escolhe a próxima a conferir depois da aberta', () => {
    expect(nextToReview(LIST, 'a')?.name).toBe('c');
  });

  it('volta ao começo quando não há depois', () => {
    expect(nextToReview(LIST, 'c')?.name).toBe('a');
  });

  it('ignora a aberta e as que não são a conferir', () => {
    expect(nextToReview([item('a', 'a-conferir'), item('b', 'recusadas')], 'a')).toBeUndefined();
  });

  it('começa pela primeira quando a aberta saiu da lista', () => {
    expect(nextToReview(LIST, 'z')?.name).toBe('a');
  });

  it('devolve undefined sem nenhuma a conferir', () => {
    expect(nextToReview([item('b', 'conferidas'), item('d', 'recusadas')], 'b')).toBeUndefined();
  });
});
