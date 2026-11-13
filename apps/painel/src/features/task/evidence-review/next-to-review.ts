import type { EvidenceEntry } from '../evidence-list/evidence-items';

/** Próxima evidência a conferir entre as filtradas, depois da aberta e dando a volta; sem outra, `undefined` (RF-18, CB-04). */
export function nextToReview(entries: EvidenceEntry[], currentName: string): EvidenceEntry | undefined {
  const index = entries.findIndex((entry) => entry.name === currentName);
  const ordered = index === -1 ? entries : [...entries.slice(index + 1), ...entries.slice(0, index)];
  return ordered.find((entry) => entry.category === 'a-conferir' && entry.name !== currentName);
}
