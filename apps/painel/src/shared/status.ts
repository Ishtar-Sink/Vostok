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
  executed: 'success',
  validating: 'info',
  blocked: 'warning',
  done: 'success',
  passed: 'success',
  failed: 'danger',
  pendente: 'neutral',
  em_andamento: 'info',
  feito: 'success',
  recusado: 'danger',
  erro: 'danger',
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
  executed: 'pronta para finalizar',
  validating: 'validando',
  blocked: 'bloqueada',
  done: 'concluída',
  passed: 'passou',
  failed: 'falhou',
  pendente: 'pendente',
  em_andamento: 'em andamento',
  feito: 'feito',
  recusado: 'recusado',
  coletado: 'coletado',
  erro: 'erro',
  painel: 'pelo painel',
  chat: 'pelo chat',
  agente: 'pelo agente',
  usuário: 'pelo usuário',
};

const withoutSeparators = (value: string): string => value.replace(/[-_]+/g, ' ');

export const statusTone = (status: string): BadgeTone => TONES[status] ?? 'neutral';
export const statusLabel = (status: string): string => LABELS[status] ?? (withoutSeparators(status) || '—');

/** Seções do quadro, na ordem do fluxo de uma tarefa; `approved` e `ready` são os dois lados do status `approved`. */
export const BOARD_SECTIONS: { id: string; label: string; statuses: string[] }[] = [
  { id: 'pending', label: 'Para aprovar', statuses: ['pending'] },
  { id: 'approved', label: 'Aprovadas', statuses: ['approved'] },
  { id: 'ready', label: 'Prontas para execução', statuses: ['approved'] },
  { id: 'running', label: 'Em execução', statuses: ['in_progress'] },
  { id: 'executed', label: 'Prontas para finalizar', statuses: ['executed'] },
  { id: 'validating', label: 'Em validação', statuses: ['validating'] },
  { id: 'blocked', label: 'Bloqueadas', statuses: ['blocked'] },
  { id: 'done', label: 'Concluídas', statuses: ['done'] },
];
