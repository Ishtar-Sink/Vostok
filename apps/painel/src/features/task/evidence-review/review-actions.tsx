import { useState } from 'react';
import { acceptEvidence, rejectEvidence, withdrawReview, type ReviewResult } from '../../../shared/api';
import { Button } from '../../../ui/button/button';
import { ConfirmPopover, type ConfirmPopoverTone } from '../../../ui/confirm-popover/confirm-popover';
import type { EvidenceEntry } from '../evidence-list/evidence-items';
import styles from './evidence-review.module.css';

export type ReviewAction = 'aceitar' | 'recusar' | 'retirar';

const ASK: Record<
  ReviewAction,
  { title: string; description: string; fieldLabel: string; placeholder: string; confirmLabel: string; tone: ConfirmPopoverTone; required: boolean }
> = {
  aceitar: {
    title: 'Aceitar a evidência',
    description: 'Observação opcional: fica registrada como o seu aval.',
    fieldLabel: 'Observação',
    placeholder: 'Ex.: a tela bate com o layout aprovado',
    confirmLabel: 'Aceitar',
    tone: 'primary',
    required: false,
  },
  recusar: {
    title: 'Recusar a evidência',
    description: 'O motivo é obrigatório: é o que o agente vai corrigir.',
    fieldLabel: 'Motivo da recusa',
    placeholder: 'O que está errado ou falta',
    confirmLabel: 'Recusar',
    tone: 'danger',
    required: true,
  },
  retirar: {
    title: 'Retirar a decisão',
    description: 'O motivo é obrigatório e fica no registro; a evidência volta para A conferir.',
    fieldLabel: 'Motivo da retirada',
    placeholder: 'Por que retirar',
    confirmLabel: 'Retirar',
    tone: 'neutral',
    required: true,
  },
};

const SEND: Record<ReviewAction, (spec: string, task: string, id: string, sha256: string, texto: string) => Promise<ReviewResult>> = {
  aceitar: acceptEvidence,
  recusar: rejectEvidence,
  retirar: withdrawReview,
};

/** HTTP 409: o arquivo mudou depois de aberto ou a tarefa saiu da conferência (CB-06). */
const CONFLICT = 409;

export interface ReviewActionsProps {
  spec: string;
  task: string;
  entry: EvidenceEntry;
  /** Chamado depois de gravar a decisão. */
  onDone: (action: ReviewAction) => void;
  /** Chamado quando o servidor recusa porque a evidência mudou; a caixa fecha. */
  onChanged: (message: string) => void;
}

/** Botões do rodapé do modal: Recusar e Aceitar na evidência a conferir, Retirar na já decidida. */
export function ReviewActions({ spec, task, entry, onDone, onChanged }: ReviewActionsProps) {
  const [asking, setAsking] = useState<ReviewAction | null>(null);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState('');
  const status = entry.review?.status;

  if (!entry.id || (status !== 'a conferir' && status !== 'conferida' && status !== 'recusada')) return null;

  function ask(action: ReviewAction) {
    setError('');
    setAsking(action);
  }

  async function send(texto: string) {
    if (!asking) return;
    const action = asking;
    setSending(true);
    setError('');
    const result = await SEND[action](spec, task, entry.id, entry.review?.sha256 ?? '', texto);
    setSending(false);
    if (result.ok) {
      setAsking(null);
      onDone(action);
    } else if (result.status === CONFLICT) {
      setAsking(null);
      onChanged(result.message);
    } else {
      setError(result.message);
    }
  }

  return (
    <div className={styles.actions}>
      {status === 'a conferir' ? (
        <>
          <Button variant="secondary" className={styles.reject} onClick={() => ask('recusar')}>
            Recusar
          </Button>
          <Button variant="primary" onClick={() => ask('aceitar')}>
            Aceitar
          </Button>
        </>
      ) : (
        <Button variant="ghost" onClick={() => ask('retirar')}>
          {status === 'conferida' ? 'Retirar conferência' : 'Retirar recusa'}
        </Button>
      )}
      {asking ? (
        <ConfirmPopover open {...ASK[asking]} loading={sending} error={error} onCancel={() => setAsking(null)} onConfirm={send} />
      ) : null}
    </div>
  );
}
