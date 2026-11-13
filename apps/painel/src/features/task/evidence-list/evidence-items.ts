import type { EvidenceReview, FileRef, ManualEvidence, ReviewStatus, TaskDetail } from '../../../shared/api';
import type { BadgeTone } from '../../../ui/badge/badge';

export interface EvidenceNotes {
  title?: string;
  proof?: string;
  expect?: string;
  excerpt?: string;
  failure?: string;
}

const NOTE_FIELDS: Record<string, keyof EvidenceNotes> = { Título: 'title', Prova: 'proof', Espera: 'expect', Trecho: 'excerpt', Falha: 'failure' };

/** Lê os subitens de cada evidência na seção `### Evidências` da verificação; o painel só mostra o que a tarefa escreveu. */
export function evidenceNotes(verification: string): Map<string, EvidenceNotes> {
  const notes = new Map<string, EvidenceNotes>();
  let inSection = false;
  let current: EvidenceNotes | null = null;
  for (const line of verification.split('\n')) {
    if (line.startsWith('#')) {
      inSection = /^###\s+Evidências\s*$/.test(line);
      current = null;
      continue;
    }
    if (!inSection) continue;
    const opened = line.match(/^- `([^`]+)`/);
    if (opened) {
      current = {};
      notes.set(opened[1], current);
      continue;
    }
    const field = line.match(/^\s{2,}- (Título|Prova|Espera|Trecho|Falha):\s*(.*)$/);
    if (field && current) current[NOTE_FIELDS[field[1]]] = field[2];
  }
  return notes;
}

export type EvidenceCategory = 'a-conferir' | 'recusadas' | 'conferidas';

export const CATEGORIES: { id: EvidenceCategory; label: string; hint: string; tone: BadgeTone; one: string; many: string }[] = [
  { id: 'a-conferir', label: 'A conferir', hint: 'precisam do seu aval', tone: 'warning', one: 'a conferir', many: 'a conferir' },
  { id: 'recusadas', label: 'Recusadas', hint: 'voltaram para correção', tone: 'danger', one: 'recusada', many: 'recusadas' },
  { id: 'conferidas', label: 'Conferidas', hint: '', tone: 'success', one: 'conferida', many: 'conferidas' },
];

export const IMAGE = /\.(png|jpe?g|gif|webp)$/i;
export const VIDEO = /\.(mp4|webm|mov)$/i;

/** Rótulo curto do tipo da evidência, como o usuário a confere. */
export function kindLabel(kind: string, name: string): string {
  if (VIDEO.test(name)) return 'Vídeo';
  if (kind === 'diff') return 'Diff';
  if (kind === 'print') return 'Print';
  if (kind === 'manual') return 'Manual';
  if (kind.startsWith('automática')) return 'Log';
  return IMAGE.test(name) ? 'Print' : 'Log';
}

export function categoryOf(status?: ReviewStatus): EvidenceCategory {
  if (status === 'a conferir') return 'a-conferir';
  if (status === 'recusada') return 'recusadas';
  return 'conferidas';
}

export interface EvidenceEntry {
  name: string;
  id: string;
  title: string;
  kind: string;
  kindLabel: string;
  category: EvidenceCategory;
  review?: EvidenceReview;
  file?: FileRef;
  manual?: ManualEvidence;
  notes: EvidenceNotes;
}

/** Evidências da tarefa na ordem da verificação, com as sem conferência no fim e sem repetir nomes. */
export function evidenceEntries(detail: TaskDetail): EvidenceEntry[] {
  const notes = evidenceNotes(detail.files.verificacao ?? '');
  const manuals = detail.manual_evidence ?? [];
  const seen = new Set<string>();
  const entries: EvidenceEntry[] = [];
  const add = (name: string, review?: EvidenceReview) => {
    if (seen.has(name)) return;
    seen.add(name);
    const manual = manuals.find((item) => item.name === name);
    const file = detail.evidence.find((item) => item.name === name);
    const kind = review?.kind ?? (manual ? 'manual' : '');
    const entryNotes = notes.get(name) ?? {};
    entries.push({
      name,
      id: review?.id ?? '',
      title: entryNotes.title ?? name,
      kind,
      kindLabel: kindLabel(kind, name),
      category: categoryOf(review?.status),
      review,
      file,
      manual,
      notes: entryNotes,
    });
  };
  for (const review of detail.reviews ?? []) add(review.name, review);
  for (const file of detail.evidence) add(file.name);
  for (const manual of manuals) add(manual.name);
  return entries;
}
