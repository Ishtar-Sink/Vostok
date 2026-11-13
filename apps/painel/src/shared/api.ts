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
  closes_after?: string;
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
}

export interface ArchivedSpec {
  meta: SpecMeta;
  tasks_total: number;
}

export interface Overview {
  project: string;
  specs: OpenSpec[];
  archived: ArchivedSpec[];
}

export interface SpecDetail {
  meta: SpecMeta;
  path: string;
  plan: boolean;
  plan_outdated: boolean;
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

/** Evidência manual pedida pelo harness; `path` e `size` valem só com `attached`. */
export interface ManualEvidence extends FileRef {
  description: string;
  extension: string;
  attached: boolean;
}

/** Estado da conferência de uma evidência, calculado pela CLI (campo `reviews`). */
export type ReviewStatus = 'conferida' | 'recusada' | 'a conferir' | 'sem aval';

export interface EvidenceReview {
  /** `EV-NN` pela ordem dos itens de "Evidências"; ausente numa CLI anterior à 0.6.0. */
  id?: string;
  name: string;
  kind: string;
  status: ReviewStatus;
  /** Palavras do usuário: o motivo da recusa ou o aval da conferência; vazio sem elas. */
  reason: string;
  /** Data (AAAA-MM-DD) da linha de conferência ou rejeição válida; vazio a conferir; ausente numa CLI anterior à 0.6.0. */
  reviewed_at?: string;
  /** Origem da conferência: `chat` (vostok task evidence) ou `painel`; vazio sem conferência. */
  origin?: string;
  /** Hash atual da evidência, mandado de volta no POST para o servidor recusar com 409 o arquivo que mudou (CB-06). */
  sha256?: string;
  /** Comando da evidência; no diff, o caminho; na manual, a descrição. */
  command?: string;
  /** Ferramenta que gerou a evidência (Playwright, git diff, o primeiro programa do comando); vazia quando o comando não tem programa reconhecível. */
  tool?: string;
  /** Comando para reproduzir a evidência; vazio na manual. */
  reproduce?: string;
  /** Frase do subitem `Como:`; vazia sem ele. */
  how?: string;
}

export interface TaskDetail {
  spec: string;
  state: Record<string, string>;
  files: Record<string, string>;
  evidence: FileRef[];
  manual_evidence?: ManualEvidence[];
  reviews?: EvidenceReview[];
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

export type ReviewAction = 'aceitar' | 'recusar' | 'retirar';

export type ReviewResult = { ok: true } | { ok: false; status: number; message: string };

const REVIEW_ERRORS: Record<number, string> = {
  400: 'Escreva o motivo.',
  403: 'O painel recusou a gravação: recarregue a página.',
  404: 'Evidência não encontrada.',
  409: 'A evidência mudou ou a tarefa não está em conferência: recarregue a página.',
};

/** Token da sessão que o servidor põe na página; vazio fora do painel servido pela CLI. */
export function sessionToken(): string {
  return document.querySelector('meta[name="vostok-token"]')?.getAttribute('content') ?? '';
}

async function postReview(
  action: ReviewAction,
  spec: string,
  task: string,
  id: string,
  sha256: string,
  texto: string,
): Promise<ReviewResult> {
  const url = `/api/tarefas/${encodeURIComponent(spec)}/${encodeURIComponent(task)}/evidencias/${encodeURIComponent(id)}/${action}`;
  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Vostok-Token': sessionToken() },
      body: JSON.stringify(sha256 ? { texto, sha256 } : { texto }),
    });
    if (response.ok) return { ok: true };
    return { ok: false, status: response.status, message: REVIEW_ERRORS[response.status] ?? `Erro ${response.status}.` };
  } catch {
    return { ok: false, status: 0, message: 'Sem conexão com o painel.' };
  }
}

export const acceptEvidence = (spec: string, task: string, id: string, sha256: string, texto = '') =>
  postReview('aceitar', spec, task, id, sha256, texto);

export const rejectEvidence = (spec: string, task: string, id: string, sha256: string, texto: string) =>
  postReview('recusar', spec, task, id, sha256, texto);

export const withdrawReview = (spec: string, task: string, id: string, sha256: string, texto: string) =>
  postReview('retirar', spec, task, id, sha256, texto);

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
