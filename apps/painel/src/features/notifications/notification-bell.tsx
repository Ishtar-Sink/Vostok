import { useEffect, useRef, useState } from 'react';
import { formatDateTime, markNotificationsRead, useResource, type NotificationItem, type NotificationList } from '../../shared/api';
import { noticeFor } from '../../shared/notifications';
import { onLinkClick } from '../../shared/router';
import { Button } from '../../ui/button/button';
import { Checkbox } from '../../ui/checkbox/checkbox';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { ErrorState } from '../../ui/error-state/error-state';
import { LoadingState } from '../../ui/loading-state/loading-state';
import styles from './notification-bell.module.css';

const BADGE_LIMIT = 99;

function bellLabel(unread: number): string {
  if (unread === 0) return 'Notificações: nenhuma não lida';
  return unread === 1 ? 'Notificações: 1 não lida' : `Notificações: ${unread} não lidas`;
}

function BellIcon() {
  return (
    <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M6 9a6 6 0 0 1 12 0c0 6 2 7 2 7H4s2-1 2-7" />
      <path d="M10 20a2 2 0 0 0 4 0" />
    </svg>
  );
}

interface Selection {
  selected: Set<string>;
  toggle: (id: string) => void;
}

function NotificationEntry({ item, onFollow, selection }: { item: NotificationItem; onFollow: () => void; selection?: Selection }) {
  const notice = noticeFor(item);
  return (
    <li className={styles.item} data-notification-id={item.id} data-tone={notice.tone} data-read={item.lida}>
      {selection && !item.lida ? <Checkbox className={styles.select} label={`Selecionar ${item.id}`} checked={selection.selected.has(item.id)} onChange={() => selection.toggle(item.id)} /> : null}
      <p className={styles.title}>{notice.title}</p>
      <p className={styles.message}>{notice.message}</p>
      <div className={styles.meta}>
        <time dateTime={item.atualizado_em}>{formatDateTime(item.atualizado_em)}</time>
        <a href={notice.href} onClick={(event) => { if (!item.lida) void markNotificationsRead({ ids: [item.id] }); onLinkClick(event); onFollow(); }}>{notice.linkLabel}</a>
      </div>
    </li>
  );
}

function Group({ title, items, onFollow, selection }: { title: string; items: NotificationItem[]; onFollow: () => void; selection?: Selection }) {
  if (items.length === 0) return null;
  return (
    <div className={styles.group}>
      <h3 className={styles.groupTitle}>{title}</h3>
      <ul className={styles.list}>
        {items.map((item) => <NotificationEntry key={item.id} item={item} onFollow={onFollow} selection={selection} />)}
      </ul>
    </div>
  );
}

function NotificationPanel({ version, onFollow }: { version: number; onFollow: () => void }) {
  const resource = useResource<NotificationList>('/api/notificacoes', version);
  const ref = useRef<HTMLElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => ref.current?.focus(), []);

  const items = resource.kind === 'ready' ? resource.data.itens : [];
  const unreadItems = items.filter((item) => !item.lida);
  const valid = unreadItems.filter((item) => selected.has(item.id)).map((item) => item.id);
  const toggle = (id: string) => setSelected((current) => {
    const next = new Set(current);
    if (!next.delete(id)) next.add(id);
    return next;
  });

  const mark = async (target: { ids: string[] } | { todas: true }) => {
    setBusy(true);
    const result = await markNotificationsRead(target);
    setBusy(false);
    if (!result.ok) {
      setError(result.text.trim() || `erro ${result.status}`);
      return;
    }
    setSelected(new Set());
    setError('');
    heading.current?.focus();
    if (resource.kind === 'ready') resource.retry();
  };

  return (
    <section id="notificacoes" role="dialog" aria-label="Notificações" tabIndex={-1} ref={ref} className={styles.panel}>
      <h2 className={styles.heading} tabIndex={-1} ref={heading}>Notificações</h2>
      {resource.kind === 'loading' ? <LoadingState label="Carregando as notificações" rows={3} /> : null}
      {resource.kind === 'error' ? <ErrorState title="Não foi possível carregar as notificações." description={resource.message} retryLabel="Tentar de novo" onRetry={resource.retry} /> : null}
      {resource.kind === 'ready' && items.length === 0 ? <EmptyState title="Nenhuma notificação" description="Os avisos de pedidos encerrados aparecem aqui." /> : null}
      {resource.kind === 'ready' && items.length > 0 ? (
        <>
          <div className={styles.actions}>
            <Button size="small" disabled={valid.length === 0 || busy} onClick={() => void mark({ ids: valid })}>Marcar selecionadas</Button>
            <Button size="small" disabled={resource.data.nao_lidas === 0 || busy} onClick={() => void mark({ todas: true })}>Marcar todas como lidas</Button>
          </div>
          {error ? <p role="alert" className={styles.error}>Não foi possível marcar: {error}</p> : null}
          <Group title="Não lidas" items={unreadItems} onFollow={onFollow} selection={{ selected, toggle }} />
          <Group title="Lidas" items={items.filter((item) => item.lida)} onFollow={onFollow} />
        </>
      ) : null}
    </section>
  );
}

export function NotificationBell({ unread, version }: { unread: number; version: number }) {
  const [open, setOpen] = useState(false);
  const wrap = useRef<HTMLDivElement>(null);
  const button = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      setOpen(false);
      button.current?.focus();
    };
    const onClick = (event: MouseEvent) => {
      if (!wrap.current?.contains(event.target as Node)) setOpen(false);
    };
    document.addEventListener('keydown', onKey);
    document.addEventListener('click', onClick);
    return () => {
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('click', onClick);
    };
  }, [open]);

  return (
    <div className={styles.wrap} ref={wrap}>
      <button type="button" ref={button} className={styles.bell} aria-label={bellLabel(unread)} aria-expanded={open} aria-haspopup="dialog" aria-controls="notificacoes" onClick={() => setOpen(!open)}>
        <BellIcon />
        {unread > 0 ? <span className={styles.badge} aria-hidden="true">{unread > BADGE_LIMIT ? `${BADGE_LIMIT}+` : unread}</span> : null}
      </button>
      {open ? <NotificationPanel version={version} onFollow={() => setOpen(false)} /> : null}
    </div>
  );
}
