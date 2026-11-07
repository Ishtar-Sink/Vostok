import { useState } from 'react';
import { formatDateTime, type PanelRequest, type RequestOutcome } from './api';
import { ActionDialog } from './action-bar';
import { ACTION_LABELS, requestLabel, type ActionKind, type AttentionItem } from './attention';
import { Markdown } from './markdown';
import { onLinkClick } from './router';
import { Button } from '../ui/button/button';
import { Card } from '../ui/card/card';
import styles from './request-state.module.css';

/** Saídas de uma parada que o painel sabe pedir; `executar-onda` e `responder-pergunta` precisam de dados que a saída não traz. */
const OUTPUT_ACTIONS: Record<string, ActionKind> = {
  'criar-spec': 'create-spec',
  'refinar-spec': 'refine-spec',
  'ajustar-spec': 'adjust-spec',
  'planejar-spec': 'plan-spec',
  'aprovar-plano': 'approve-plan',
  'ajustar-plano': 'adjust-plan',
  'finalizar-spec': 'finalize-spec',
  'executar-tarefa': 'execute-task',
  'ajustar-tarefa': 'adjust-task',
  'finalizar-tarefa': 'finish-task',
  'desbloquear-tarefa': 'unblock-task',
  'fazer-commit': 'fazer-commit',
};
const TASK_KINDS: ActionKind[] = ['execute-task', 'adjust-task', 'finish-task', 'unblock-task'];

function outputItems(outcome: RequestOutcome, spec: string, task?: string): AttentionItem[] {
  const items: AttentionItem[] = [];
  for (const name of outcome.saidas) {
    const kind = OUTPUT_ACTIONS[name];
    if (!kind) continue;
    const forTask = TASK_KINDS.includes(kind);
    if (forTask && !task) continue;
    items.push({
      key: `saida-${name}`,
      kind,
      spec: kind === 'create-spec' ? '' : spec,
      task: forTask ? task : undefined,
      title: `Saída do pedido ${outcome.id}`,
      reason: '',
      href: task ? `/tarefas/${spec}/${task}` : `/specs/${spec}`,
    });
  }
  return items;
}

/** Estado visível do pedido da spec ou da tarefa: ativo (fila ou andamento) ou o desfecho da última parada. */
export function RequestState({ spec, task, request, outcome, queued = 1 }: { spec: string; task?: string; request: PanelRequest | undefined; outcome: RequestOutcome | null; queued?: number }) {
  const [selected, setSelected] = useState<AttentionItem | null>(null);

  if (request) {
    return (
      <Card as="section" tone="info" aria-label="Estado do pedido" aria-live="polite" className={styles.root}>
        <p className={styles.title}>{requestLabel(request)}</p>
        <p className={styles.line}>
          {request.estado === 'pendente'
            ? `Pedido ${request.id} na fila desde ${formatDateTime(request.atualizado_em)}.`
            : `Pedido ${request.id} em andamento desde ${formatDateTime(request.atualizado_em)}.`}
        </p>
        {queued > 1 ? <p className={styles.line}>{`${queued} pedidos na fila; este é o mais antigo.`}</p> : null}
        <p className={styles.line}>As ações voltam quando o agente terminar o pedido.</p>
      </Card>
    );
  }
  if (outcome?.desfecho === 'aguardando-pergunta') {
    return (
      <Card as="section" tone="accent" aria-label="Estado do pedido" aria-live="polite" className={styles.root}>
        <p className={styles.title}>{`Aguardando sua resposta à ${outcome.pergunta}`}</p>
        {outcome.resultado ? <Markdown untrusted source={outcome.resultado} /> : null}
        <a href="/pendencias" onClick={onLinkClick} className={styles.link}>Responder em Pendências</a>
      </Card>
    );
  }
  if (outcome?.desfecho === 'pausado') {
    const items = outputItems(outcome, spec, task);
    return (
      <Card as="section" tone="warning" aria-label="Estado do pedido" aria-live="polite" className={styles.root}>
        <p className={styles.title}>Pedido pausado</p>
        <p className={styles.line}>{`O pedido ${outcome.id} parou e espera a sua decisão.`}</p>
        <Markdown untrusted source={outcome.resultado || 'O agente não deixou o motivo.'} />
        {items.length > 0 ? (
          <div className={styles.outputs} role="group" aria-label="Saídas do pedido">
            {items.map((item) => (
              <Button key={item.key} variant="secondary" onClick={() => setSelected(item)}>{ACTION_LABELS[item.kind].button}</Button>
            ))}
          </div>
        ) : null}
        <ActionDialog item={selected} onClose={() => setSelected(null)} />
      </Card>
    );
  }
  return null;
}
