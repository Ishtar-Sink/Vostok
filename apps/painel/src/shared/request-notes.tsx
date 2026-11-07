import { useContext } from 'react';
import { LiveVersion } from './action-bar';
import { formatDateTime, useResource, type RequestDetail } from './api';
import { Button } from '../ui/button/button';
import { Dialog } from '../ui/dialog/dialog';
import styles from './request-notes.module.css';

/** Janela com as notas de um pedido, na ordem em que foram gravadas. O texto vem do agente e fica como texto puro. */
export function RequestNotes({ id, onClose }: { id: string; onClose: () => void }) {
  const detail = useResource<RequestDetail>(`/api/pedidos/${encodeURIComponent(id)}`, useContext(LiveVersion));
  return (
    <Dialog open title={`Notas do pedido ${id}`} size="content" onClose={onClose} actions={<Button variant="secondary" onClick={onClose}>Fechar</Button>}>
      {detail.kind === 'loading' ? <p className={styles.state}>Carregando as notas…</p> : null}
      {detail.kind === 'error' ? (
        <div className={styles.state} role="alert">
          <p>Não foi possível ler as notas: {detail.message}</p>
          <Button variant="secondary" onClick={detail.retry}>Tentar de novo</Button>
        </div>
      ) : null}
      {detail.kind === 'ready' && detail.data.notas.length === 0 ? <p className={styles.state}>O agente ainda não gravou notas neste pedido.</p> : null}
      {detail.kind === 'ready' && detail.data.notas.length > 0 ? (
        <ol className={styles.list} aria-label="Notas" aria-live="polite" tabIndex={0}>
          {detail.data.notas.map((note, index) => (
            <li key={`${note.em}-${index}`} className={styles.note}>
              <time dateTime={note.em} className={styles.time}>{formatDateTime(note.em)}</time>
              <p className={styles.text}>{note.texto}</p>
              {note.detalhe ? <pre className={styles.detail}>{note.detalhe}</pre> : null}
            </li>
          ))}
        </ol>
      ) : null}
    </Dialog>
  );
}
