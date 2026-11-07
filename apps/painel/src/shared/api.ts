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
  check_result?: string;
  waiting_on?: string;
  waiting_reason?: string;
  last_outcome: RequestOutcome | null;
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
  plan_outdated: boolean;
  tasks: TaskSummary[];
  last_outcome: RequestOutcome | null;
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

export interface PanelRequest {
  id: string;
  acao: string;
  spec: string;
  tarefa: string;
  /** N do "Onda N." no início do texto do `executar-onda`; vazio nos demais. */
  onda: string;
  estado: 'pendente' | 'em_andamento';
  atualizado_em: string;
  /** Última nota do agente e a hora dela; ausentes sem nota. */
  nota?: string;
  nota_em?: string;
}

export interface RequestNote {
  em: string;
  texto: string;
  detalhe: string;
}

export interface RequestDetail {
  id: string;
  acao: string;
  spec: string;
  tarefa: string;
  estado: string;
  texto: string;
  notas: RequestNote[];
  resultado: string;
  lida_em: string;
}

export interface RequestOutcome {
  id: string;
  acao: string;
  desfecho: 'concluido' | 'aguardando-pergunta' | 'pausado';
  pergunta: string;
  saidas: string[];
  resultado: string;
}

/** Última rodada do loop de pedidos, lida de `work/pedidos/.loop`. */
export interface LoopRound {
  ultima_rodada: string;
  intervalo: number;
  agente: string;
  /** Pedido em andamento sem nota nova e desde quando; ausentes sem aviso. */
  sem_nota_pedido?: string;
  sem_nota_desde?: string;
}

/** Pergunta livre ao agente (`/api/perguntas-livres`); `resultado` é a resposta ou o motivo da recusa. */
export interface FreeQuestion {
  id: string;
  spec: string;
  estado: 'pendente' | 'em_andamento' | 'feito' | 'recusado';
  texto: string;
  resultado: string;
  atualizado_em: string;
  lida_em: string;
  /** Os arquivos que estão na pasta de anexos do pedido; `caminho` é relativo à raiz do projeto. */
  anexos: { nome: string; caminho: string; tamanho: number }[];
}

export interface UnreadAnswer {
  id: string;
  spec: string;
  texto: string;
}

export interface Overview {
  project: string;
  specs: OpenSpec[];
  archived: ArchivedSpec[];
  activity: Activity[];
  questions: Question[];
  requests: PanelRequest[];
  loop: LoopRound | null;
  respostas_nao_lidas?: UnreadAnswer[];
  notificacoes_nao_lidas?: number;
  toast_dismiss_seconds?: number;
}

export interface NotificationItem {
  id: string;
  acao: string;
  spec: string;
  tarefa: string;
  estado: string;
  desfecho: string;
  pergunta: string;
  atualizado_em: string;
  lida: boolean;
}

export interface NotificationList {
  nao_lidas: number;
  itens: NotificationItem[];
}

export interface SpecDetail {
  meta: SpecMeta;
  path: string;
  plan: boolean;
  plan_outdated: boolean;
  markdown: string;
  tasks: TaskSummary[];
  last_outcome: RequestOutcome | null;
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

/** Evidência manual pedida pelo harness; `path` e `size` valem só com `attached`. */
export interface ManualEvidence extends FileRef {
  description: string;
  extension: string;
  attached: boolean;
}

export interface TaskDetail {
  spec: string;
  state: Record<string, string>;
  files: Record<string, string>;
  evidence: FileRef[];
  manual_evidence: ManualEvidence[];
  attach_limit_mb: number;
  last_outcome: RequestOutcome | null;
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
  deslocamento?: number;
  tipo?: ActivityType;
  spec?: string;
}

export async function fetchActivities(filter: ActivityFilter = {}): Promise<Activity[]> {
  const query = new URLSearchParams();
  if (filter.limite !== undefined) query.set('limite', String(filter.limite));
  if (filter.deslocamento !== undefined) query.set('deslocamento', String(filter.deslocamento));
  if (filter.tipo) query.set('tipo', filter.tipo);
  if (filter.spec) query.set('spec', filter.spec);
  const response = await fetch(`/api/atividades?${query.toString()}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

export function fileUrl(path: string): string {
  return `/arquivos/${path.split('/').map(encodeURIComponent).join('/')}`;
}

export const TEXT_PREVIEW_BYTES = 1024 * 1024;

export interface TextPreview {
  text: string;
  total: number | null;
}

export async function readTextPreview(path: string): Promise<TextPreview> {
  const response = await fetch(fileUrl(path), { headers: { Range: `bytes=0-${TEXT_PREVIEW_BYTES - 1}` } });
  if (!response.ok) throw new Error(`Falha ${response.status} ao ler o arquivo`);
  const text = new TextDecoder().decode(await response.arrayBuffer());
  const total = Number((response.headers.get('Content-Range') ?? '').split('/')[1]);
  return { text, total: Number.isFinite(total) && total > TEXT_PREVIEW_BYTES ? total : null };
}

export function listOf(value: string): string[] {
  return value.replace(/[[\]]/g, '').split(',').map((item) => item.trim()).filter(Boolean);
}

export function formatDate(value: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2})/.exec(value);
  return match ? `${match[3]}/${match[2]}/${match[1]}` : value || '—';
}

const ISO_DATETIME = /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(?::\d{2})?Z/g;

function sameDay(a: Date, b: Date): boolean {
  return a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
}

/** Data, e hora quando houver, no fuso local: "hoje, 14:32", "ontem, 14:32" ou "03/11/2026, 14:32". */
export function formatDateTime(value: string, now: Date = new Date()): string {
  if (!value) return '—';
  const dateOnly = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  const date = dateOnly ? new Date(Number(dateOnly[1]), Number(dateOnly[2]) - 1, Number(dateOnly[3])) : new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  const yesterday = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1);
  const pad = (number: number) => String(number).padStart(2, '0');
  const day = sameDay(date, now) ? 'hoje' : sameDay(date, yesterday) ? 'ontem' : `${pad(date.getDate())}/${pad(date.getMonth() + 1)}/${date.getFullYear()}`;
  return dateOnly ? day : `${day}, ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** Troca as datas ISO com hora de um texto pelo formato de `formatDateTime`. */
export function formatDatesIn(text: string, now: Date = new Date()): string {
  return text.replace(ISO_DATETIME, (match) => formatDateTime(match, now));
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

/** Marca notificações como lidas: as dos `ids` ou todas (`todas=sim`), inclusive as fora da lista. */
export function markNotificationsRead(target: { ids: string[] } | { todas: true }): Promise<ActionResult> {
  return act('POST', 'ids' in target ? `/api/notificacoes/lidas?ids=${target.ids.join(',')}` : '/api/notificacoes/lidas?todas=sim');
}

/** Envia o arquivo de uma evidência manual com o progresso (0 a 100); a rede que cai resolve com `status` 0. */
export function uploadEvidence(spec: string, task: string, evidence: string, file: File, onProgress: (percent: number) => void): Promise<ActionResult> {
  return new Promise((resolve) => {
    const dot = file.name.lastIndexOf('.');
    const extension = dot >= 0 ? file.name.slice(dot + 1) : '';
    const query = new URLSearchParams({ spec, tarefa: task, evidencia: evidence, extensao: extension });
    const xhr = new XMLHttpRequest();
    xhr.open('PUT', `/api/evidencias?${query.toString()}`);
    xhr.setRequestHeader('X-Painel-Token', sessionToken());
    xhr.upload.onprogress = (event) => {
      if (event.lengthComputable) onProgress(Math.round((event.loaded / event.total) * 100));
    };
    xhr.onload = () => resolve({ status: xhr.status, ok: xhr.status >= 200 && xhr.status <= 299, text: xhr.responseText });
    xhr.onerror = () => resolve({ status: 0, ok: false, text: 'falha de rede' });
    xhr.onabort = () => resolve({ status: 0, ok: false, text: 'falha de rede' });
    xhr.send(file);
  });
}

/** Envia um anexo do rascunho de uma pergunta com o progresso (0 a 100); a rede que cai resolve com `status` 0. */
export function uploadAttachment(draft: string, file: File, name: string, onProgress: (percent: number) => void): Promise<ActionResult> {
  return new Promise((resolve) => {
    const xhr = new XMLHttpRequest();
    xhr.open('PUT', `/api/pedidos/anexos?rascunho=${encodeURIComponent(draft)}&nome=${encodeURIComponent(name)}`);
    xhr.setRequestHeader('X-Painel-Token', sessionToken());
    xhr.upload.addEventListener('progress', (event) => {
      if (event.lengthComputable) onProgress(Math.round((event.loaded / event.total) * 100));
    });
    xhr.addEventListener('load', () => resolve({ status: xhr.status, ok: xhr.status >= 200 && xhr.status <= 299, text: xhr.responseText }));
    xhr.addEventListener('error', () => resolve({ status: 0, ok: false, text: 'falha de rede' }));
    xhr.addEventListener('abort', () => resolve({ status: 0, ok: false, text: 'falha de rede' }));
    xhr.send(file);
  });
}

export interface AttachmentLimits {
  max_mb: number;
  max_arquivos: number;
  nomes_recusados: string[];
  nomes_aceitos: string[];
}
