import { useCallback, useEffect, useMemo, useState } from 'react';

export interface TaskSummary {
  id: string;
  title: string;
  status: string;
  wave: string;
  profile: string;
  persona: string;
  model: string;
  depends_on: string;
  covers: string;
  type: string;
  description: string;
}

export interface SpecMeta {
  id: string;
  title: string;
  status: string;
  type: string;
  author: string;
  created_at: string;
  updated_at: string;
  refined_at: string;
  approved_at: string;
  adjustments: string;
}

export interface OpenSpec {
  meta: SpecMeta;
  plan: boolean;
  tasks: TaskSummary[];
}

export interface ArchivedSpec {
  meta: SpecMeta;
  tasks_total: number;
}

export type ActivityType = 'tarefa' | 'spec' | 'plano' | 'pedido' | 'aval';

export interface Activity {
  date: string;
  type: ActivityType;
  spec: string;
  task: string;
  target: string;
  step: string;
  result: string;
  detail: string;
}

export interface Question {
  meta: { id: string; spec: string; task: string; status: string; created_at: string; author: string };
  path: string;
  markdown: string;
}

export interface Overview {
  project: string;
  specs: OpenSpec[];
  archived: ArchivedSpec[];
  activity: Activity[];
  questions: Question[];
}

export interface SpecDetail {
  meta: SpecMeta;
  path: string;
  plan: boolean;
  markdown: string;
  tasks: TaskSummary[];
}

export interface PlanDetail {
  id: string;
  path: string;
  markdown: string;
}

export interface FileRef {
  name: string;
  path: string;
  size: number;
}

export interface TaskDetail {
  spec: string;
  state: Record<string, string>;
  files: Record<string, string>;
  evidence: FileRef[];
}

export interface DocumentGroup {
  meta: SpecMeta;
  docs: FileRef[];
}

export interface Brand {
  brand: { name: string; icon: string; primary: string; secondary: string };
  icon_data: string;
  light: Record<string, string>;
  dark: Record<string, string>;
}

export type Resource<T> = (
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; data: T }
) & { retry: () => void };

/** Busca um recurso da API e busca de novo a cada aviso do WebSocket; mantém o dado anterior
 *  enquanto a nova busca não chega, para a tela não piscar. */
export function useResource<T>(url: string, version: number): Resource<T> {
  const [state, setState] = useState<{ kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; data: T }>({ kind: 'loading' });
  const [attempt, setAttempt] = useState(0);
  const retry = useCallback(() => setAttempt((value) => value + 1), []);

  useEffect(() => {
    let active = true;
    fetch(url)
      .then((response) => {
        if (!response.ok) throw new Error(response.status === 404 ? 'Não encontrado.' : `Erro ${response.status}.`);
        return response.json() as Promise<T>;
      })
      .then((data) => active && setState({ kind: 'ready', data }))
      .catch((error: Error) => active && setState({ kind: 'error', message: error.message }));
    return () => {
      active = false;
    };
  }, [url, version, attempt]);

  return useMemo(() => ({ ...state, retry }), [state, retry]);
}

/** Espera antes de reconectar o WebSocket depois de uma queda. */
const RECONNECT_MS = 2000;

/** Número que muda a cada aviso `mudou` do WebSocket; as telas o usam para buscar de novo. */
/** `connected` é `null` até a primeira conexão, para a tela distinguir "conectando" de uma queda. */
export function useLiveVersion(): { version: number; connected: boolean | null } {
  const [version, setVersion] = useState(0);
  const [connected, setConnected] = useState<boolean | null>(null);

  useEffect(() => {
    let socket: WebSocket | null = null;
    let timer = 0;
    let active = true;
    const connect = () => {
      socket = new WebSocket(`ws://${window.location.host}/api/ao-vivo`);
      socket.onopen = () => setConnected(true);
      socket.onmessage = (event) => {
        if (event.data === 'mudou') setVersion((value) => value + 1);
      };
      socket.onclose = () => {
        setConnected(false);
        if (active) timer = window.setTimeout(connect, RECONNECT_MS);
      };
    };
    connect();
    return () => {
      active = false;
      window.clearTimeout(timer);
      socket?.close();
    };
  }, []);

  return { version, connected };
}

export interface ActivityFilter {
  limite?: number;
  tipo?: ActivityType;
  spec?: string;
}

export async function fetchActivities(filter: ActivityFilter = {}): Promise<Activity[]> {
  const query = new URLSearchParams();
  if (filter.limite !== undefined) query.set('limite', String(filter.limite));
  if (filter.tipo) query.set('tipo', filter.tipo);
  if (filter.spec) query.set('spec', filter.spec);
  const response = await fetch(`/api/atividades?${query.toString()}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export function fileUrl(path: string): string {
  return `/arquivos/${path.split('/').map(encodeURIComponent).join('/')}`;
}

export function listOf(value: string): string[] {
  return value.replace(/[[\]]/g, '').split(',').map((item) => item.trim()).filter(Boolean);
}

export function formatDate(value: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  return match ? `${match[3]}/${match[2]}/${match[1]}` : value || '—';
}

/** Token da sessão, que o servidor põe na página; sem ele, as ações são recusadas. */
function sessionToken(): string {
  return document.querySelector<HTMLMetaElement>('meta[name="painel-token"]')?.content ?? '';
}

export interface ActionResult {
  status: number;
  ok: boolean;
  text: string;
}

/** Ação do usuário (aprovar, gravar, pedir), com o token da sessão. */
export async function act(method: 'POST' | 'PUT', url: string, body = ''): Promise<ActionResult> {
  try {
    const response = await fetch(url, { method, body, headers: { 'X-Painel-Token': sessionToken(), 'Content-Type': 'text/plain; charset=utf-8' } });
    return { status: response.status, ok: response.ok, text: await response.text() };
  } catch (error) {
    return { status: 0, ok: false, text: error instanceof Error ? error.message : 'falha de rede' };
  }
}
