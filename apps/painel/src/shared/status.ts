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

const ACTIVITY_LABELS: Record<string, string> = {
  início: 'Início',
  baseline: 'Medição inicial',
  testes: 'Testes',
  regressão: 'Regressão',
  evidências: 'Evidências',
  conclusão: 'Conclusão',
  check: 'Conferência',
  conferência: 'Conferência',
  ajuste: 'Ajuste',
  retomada: 'Retomada',
  status: 'Mudança de status',
  criado: 'Pedido criado',
  estado: 'Mudança de estado',
  aprovar: 'Aprovação',
  'aprovar-plano': 'Aprovar o plano',
  'planejar-spec': 'Planejar a spec',
  'ajustar-spec': 'Ajustar a spec',
  'ajustar-plano': 'Ajustar o plano',
  'ajustar-tarefa': 'Ajustar a tarefa',
  'executar-tarefa': 'Executar a tarefa',
  'finalizar-tarefa': 'Finalizar a tarefa',
  'desbloquear-tarefa': 'Desbloquear a tarefa',
  'finalizar-spec': 'Finalizar a spec',
  'responder-pergunta': 'Responder pergunta',
  'fazer-commit': 'Fazer commit',
  'criar-spec': 'Criar spec',
  'refinar-spec': 'Refinar a spec',
  'executar-onda': 'Executar a onda',
  'liberar-commits': 'Liberar commits',
  'desfazer-commits': 'Desfazer commits',
  'perguntar-agente': 'Perguntar ao agente',
};

const withoutSeparators = (value: string): string => value.replace(/[-_]+/g, ' ');

export const statusTone = (status: string): BadgeTone => TONES[status] ?? 'neutral';
export const statusLabel = (status: string): string => LABELS[status] ?? (withoutSeparators(status) || '—');

export function activityLabel(value: string): string {
  const known = ACTIVITY_LABELS[value];
  if (known) return known;
  const plain = withoutSeparators(value).trim();
  return plain.charAt(0).toUpperCase() + plain.slice(1);
}

/** Seções do quadro, na ordem do fluxo de uma tarefa; `approved` e `ready` são os dois lados do status `approved`. */
export const BOARD_SECTIONS: { id: string; label: string; statuses: string[] }[] = [
  { id: 'pending', label: 'Para aprovar', statuses: ['pending'] },
  { id: 'approved', label: 'Aprovadas', statuses: ['approved'] },
  { id: 'ready', label: 'Prontas para execução', statuses: ['approved'] },
  { id: 'running', label: 'Em execução', statuses: ['in_progress'] },
  { id: 'validating', label: 'Em validação', statuses: ['validating'] },
  { id: 'blocked', label: 'Bloqueadas', statuses: ['blocked'] },
  { id: 'done', label: 'Concluídas', statuses: ['done'] },
];
