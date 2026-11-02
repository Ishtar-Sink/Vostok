import type { BadgeTone } from '../ui/badge/badge';

const TONES: Record<string, BadgeTone> = {
  draft: 'neutral',
  refined: 'info',
  approved: 'primary',
  planned: 'accent',
  implemented: 'success',
  archived: 'neutral',
  pending: 'neutral',
  in_progress: 'info',
  validating: 'info',
  blocked: 'warning',
  done: 'success',
  passed: 'success',
  failed: 'danger',
};

const LABELS: Record<string, string> = {
  draft: 'rascunho',
  refined: 'refinada',
  approved: 'aprovada',
  planned: 'planejada',
  implemented: 'implementada',
  archived: 'arquivada',
  pending: 'pendente',
  in_progress: 'em execução',
  validating: 'validando',
  blocked: 'bloqueada',
  done: 'concluída',
  passed: 'passou',
  failed: 'falhou',
};

export const statusTone = (status: string): BadgeTone => TONES[status] ?? 'neutral';
export const statusLabel = (status: string): string => LABELS[status] ?? (status || '—');

/** Colunas do quadro, na ordem do fluxo de uma tarefa. */
export const BOARD_COLUMNS: { id: string; label: string; statuses: string[] }[] = [
  { id: 'pending', label: 'Para aprovar', statuses: ['pending'] },
  { id: 'approved', label: 'Aprovadas', statuses: ['approved'] },
  { id: 'running', label: 'Em execução', statuses: ['in_progress', 'validating'] },
  { id: 'blocked', label: 'Bloqueadas', statuses: ['blocked'] },
  { id: 'done', label: 'Concluídas', statuses: ['done'] },
];
