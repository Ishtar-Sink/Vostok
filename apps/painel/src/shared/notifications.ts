import { useCallback, useEffect, useRef, useState } from 'react';
import { markNotificationsRead, type Overview, type PanelRequest, type RequestOutcome } from './api';
import { navigate, onLinkClick } from './router';

export interface ReturnNotice {
  id: string;
  tone: 'success' | 'info' | 'warning';
  title: string;
  message: string;
  href: string;
  linkLabel: string;
}

export const MAX_NOTICES = 3;
export const DEFAULT_TOAST_SECONDS = 5;

/** Desfecho gravado para o pedido, na spec ou na tarefa dele; `null` quando não é o dele. */
function outcomeOf(request: PanelRequest, overview: Overview): RequestOutcome | null {
  const spec = overview.specs.find((item) => item.meta.id === request.spec);
  if (!spec) return null;
  const last = request.tarefa ? spec.tasks.find((task) => task.id === request.tarefa)?.last_outcome : spec.last_outcome;
  return last && last.id === request.id ? last : null;
}

/** O que o texto do aviso precisa saber de um pedido que voltou; a notificação do sino usa a mesma fonte. */
export interface NoticeSource {
  id: string;
  acao: string;
  spec: string;
  tarefa: string;
  desfecho: string;
  pergunta: string;
}

export function noticeFor(source: NoticeSource): ReturnNotice {
  const where = [source.spec, source.tarefa].filter(Boolean).join(' · ');
  const page = source.tarefa ? `/tarefas/${source.spec}/${source.tarefa}` : `/specs/${source.spec}`;
  const label = where ? `${source.id} (${where})` : source.id;
  const base = { id: source.id };
  const question = source.acao === 'perguntar-agente' ? { href: '/perguntas', linkLabel: 'Ver a resposta' } : null;
  if (source.desfecho === 'aguardando-pergunta') {
    return { ...base, tone: 'warning', title: 'O agente tem uma pergunta', message: `${label} espera a sua resposta à ${source.pergunta}.`, href: '/pendencias', linkLabel: 'Responder', ...question };
  }
  if (source.desfecho === 'pausado') {
    return { ...base, tone: 'warning', title: 'Pedido pausado', message: `${label} parou e espera a sua decisão.`, href: page, linkLabel: 'Ver o motivo', ...question };
  }
  if (source.desfecho === 'concluido') {
    return { ...base, tone: 'success', title: 'Pedido concluído', message: `${label} terminou. Veja o próximo passo.`, href: page, linkLabel: source.tarefa ? 'Ver a tarefa' : 'Ver a spec', ...question };
  }
  return { ...base, tone: 'info', title: 'Pedido encerrado', message: `${label} saiu da fila. Veja o resultado em Atividades.`, href: '/atividades', linkLabel: 'Ver em Atividades', ...question };
}

export function returnNotice(request: PanelRequest, overview: Overview): ReturnNotice {
  const outcome = outcomeOf(request, overview);
  return noticeFor({ id: request.id, acao: request.acao, spec: request.spec, tarefa: request.tarefa, desfecho: outcome?.desfecho ?? '', pergunta: outcome?.pergunta ?? '' });
}

/** Os pedidos da leitura anterior que não estão mais na atual. */
export function returnedRequests(previous: PanelRequest[], current: PanelRequest[]): PanelRequest[] {
  return previous.filter((item) => !current.some((other) => other.id === item.id));
}

function browserPermission(): NotificationPermission | 'unsupported' {
  return typeof Notification === 'undefined' ? 'unsupported' : Notification.permission;
}

function notifyBrowser(notice: ReturnNotice) {
  const notification = new Notification(notice.title, { body: notice.message, tag: notice.id });
  notification.onclick = () => {
    window.focus();
    navigate(notice.href);
    notification.close();
  };
}

/** Avisos de volta: um pedido que sai da fila entre duas leituras da visão geral vira um aviso na tela e, se permitido, uma notificação do navegador. */
export function useReturnNotices(overview: Overview | null) {
  const [notices, setNotices] = useState<ReturnNotice[]>([]);
  const [permission, setPermission] = useState(browserPermission);
  const [declined, setDeclined] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const previous = useRef<PanelRequest[] | null>(null);
  const timers = useRef(new Map<string, ReturnType<typeof setTimeout>>());
  const seconds = overview?.toast_dismiss_seconds ?? DEFAULT_TOAST_SECONDS;

  useEffect(() => {
    if (!overview) return;
    const before = previous.current;
    previous.current = overview.requests;
    if (!before) return;
    const fresh = returnedRequests(before, overview.requests).map((item) => returnNotice(item, overview));
    if (fresh.length === 0) return;
    setNotices((list) => [...fresh, ...list].slice(0, MAX_NOTICES));
    if (browserPermission() === 'granted' && !document.hasFocus()) fresh.forEach(notifyBrowser);
  }, [overview]);

  const dismiss = useCallback((id: string) => setNotices((list) => list.filter((notice) => notice.id !== id)), []);

  // Cada aviso tem o seu tempo; o ponteiro ou o foco na região param todos, e ao sair cada um recomeça inteiro.
  useEffect(() => {
    const paused = hovered || focused;
    const live = new Set(notices.map((notice) => notice.id));
    for (const [id, timer] of timers.current) {
      if (paused || seconds <= 0 || !live.has(id)) {
        clearTimeout(timer);
        timers.current.delete(id);
      }
    }
    if (paused || seconds <= 0) return;
    for (const notice of notices) {
      if (timers.current.has(notice.id)) continue;
      timers.current.set(notice.id, setTimeout(() => {
        timers.current.delete(notice.id);
        dismiss(notice.id);
      }, seconds * 1000));
    }
  }, [notices, hovered, focused, seconds, dismiss]);

  useEffect(() => () => {
    timers.current.forEach((timer) => clearTimeout(timer));
    timers.current.clear();
  }, []);

  useEffect(() => {
    if (notices.length === 0) {
      setHovered(false);
      setFocused(false);
    }
  }, [notices.length]);

  const regionProps = {
    onMouseEnter: () => setHovered(true),
    onMouseLeave: () => setHovered(false),
    onFocus: () => setFocused(true),
    onBlur: () => setFocused(false),
  };
  const followNotice = useCallback((event: React.MouseEvent<HTMLAnchorElement>, notice: ReturnNotice) => {
    void markNotificationsRead({ ids: [notice.id] });
    onLinkClick(event);
    dismiss(notice.id);
  }, [dismiss]);
  const enableBrowser = useCallback(async () => {
    if (typeof Notification === 'undefined') return;
    setPermission(await Notification.requestPermission());
  }, []);
  const declineBrowser = useCallback(() => setDeclined(true), []);

  const askPermission = permission === 'default' && !declined && (overview?.requests.length ?? 0) > 0;
  return { notices, dismiss, askPermission, enableBrowser, declineBrowser, regionProps, followNotice };
}
