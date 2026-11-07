import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import type { ChangeEvent, ClipboardEvent, DragEvent } from 'react';
import type { AttachmentLimits } from '../../shared/api';
import { Markdown } from '../../shared/markdown';
import { Button } from '../../ui/button/button';
import { ProgressBar } from '../../ui/progress-bar/progress-bar';
import { Tabs } from '../../ui/tabs/tabs';
import styles from './question-composer.module.css';

const BYTES_PER_KB = 1024;
const BYTES_PER_MB = BYTES_PER_KB * 1024;
const THUMBNAIL_TYPES = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];
const UNNAMED_PASTE = 'image.png';
const FOCUS_ATTACH = Symbol('anexar');
const TEXT_REQUIRED = 'Escreva a pergunta: o texto continua obrigatório, mesmo com anexos.';

export interface PendingAttachment {
  id: string;
  file: File;
  name: string;
  label: string;
  preview: string | null;
}

export interface AttachmentCandidate {
  file: File;
  isDirectory: boolean;
}

type FileTransfer = Pick<DataTransfer, 'files'> & { items?: DataTransferItemList };

let lastAttachmentId = 0;

/** A mesma regra da CLI, que confere de novo no envio: a lista vem da API e nunca é copiada aqui. */
export function isRefusedName(name: string, limits: AttachmentLimits): boolean {
  const normalized = (name.split(/[/\\]/).pop() ?? name)
    .replace(/\p{Cc}/gu, '')
    .replace(/[.\s]+$/u, '')
    .toLowerCase();
  if (limits.nomes_aceitos.includes(normalized)) return false;
  return limits.nomes_recusados.some((pattern) => {
    if (pattern.startsWith('*')) return normalized.endsWith(pattern.slice(1));
    if (pattern.endsWith('*')) return normalized.startsWith(pattern.slice(0, -1));
    return normalized === pattern;
  });
}

export function formatSize(bytes: number): string {
  if (bytes < BYTES_PER_KB) return `${bytes} B`;
  const [value, unit] = bytes < BYTES_PER_MB ? [bytes / BYTES_PER_KB, 'KB'] : [bytes / BYTES_PER_MB, 'MB'];
  return `${value.toFixed(1).replace('.', ',')} ${unit}`;
}

export function isThumbnailType(type: string): boolean {
  return THUMBNAIL_TYPES.includes(type);
}

/** `webkitGetAsEntry` só vale durante o evento: esta leitura precisa ser síncrona. */
function transferCandidates(transfer: FileTransfer): AttachmentCandidate[] {
  const entries = Array.from(transfer.items ?? [])
    .filter((item) => item.kind === 'file')
    .map((item) => item.webkitGetAsEntry?.() ?? null);
  return Array.from(transfer.files).map((file, index) => ({ file, isDirectory: entries[index]?.isDirectory === true }));
}

/** Alguns gerenciadores entregam o nome ou o caminho do arquivo como texto; a planilha entrega a imagem da
 *  célula junto do texto. Por isso, só o texto feito dos nomes dos arquivos não impede o anexo. */
export function pastedFiles(clipboard: FileTransfer & Pick<DataTransfer, 'getData'>): AttachmentCandidate[] {
  const candidates = transferCandidates(clipboard);
  const names = candidates.map(({ file }) => file.name).filter(Boolean);
  const lines = clipboard
    .getData('text/plain')
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  return lines.every((line) => names.some((name) => line.endsWith(name))) ? candidates : [];
}

function uniqueLabel(name: string, attachments: PendingAttachment[]): string {
  const taken = new Set(attachments.map((item) => item.label));
  if (!taken.has(name)) return name;
  let copy = 2;
  while (taken.has(`${name} (${copy})`)) copy += 1;
  return `${name} (${copy})`;
}

function refusalReason({ file, isDirectory }: AttachmentCandidate, name: string, current: PendingAttachment[], limits: AttachmentLimits | null): string | null {
  if (isDirectory) return `${name} é uma pasta: anexe os arquivos dela, um a um.`;
  if (limits && isRefusedName(name, limits)) return `${name} parece um arquivo de segredo. Para o agente ler, peça na própria pergunta, citando o arquivo.`;
  if (file.size === 0) return `${name} está vazio e não foi anexado.`;
  if (!limits) return null;
  if (current.length >= limits.max_arquivos) return `${name} não foi anexado: a pergunta aceita até ${limits.max_arquivos} anexos.`;
  const total = current.reduce((sum, item) => sum + item.file.size, file.size);
  if (total > limits.max_mb * BYTES_PER_MB) return `${name} não foi anexado: a soma dos anexos passaria de ${limits.max_mb} MB.`;
  return null;
}

/** O único código de entrada: colar, soltar e Anexar passam por aqui. */
export function addAttachments(current: PendingAttachment[], candidates: AttachmentCandidate[], limits: AttachmentLimits | null): { next: PendingAttachment[]; refused: string[] } {
  const next = [...current];
  const refused: string[] = [];
  for (const candidate of candidates) {
    const name = candidate.file.name || UNNAMED_PASTE;
    const reason = refusalReason(candidate, name, next, limits);
    if (reason) {
      refused.push(reason);
      continue;
    }
    lastAttachmentId += 1;
    next.push({
      id: `anexo-${lastAttachmentId}`,
      file: candidate.file,
      name,
      label: uniqueLabel(name, next),
      preview: isThumbnailType(candidate.file.type) ? URL.createObjectURL(candidate.file) : null,
    });
  }
  return { next, refused };
}

function revokePreview(item: PendingAttachment) {
  if (item.preview) URL.revokeObjectURL(item.preview);
}

interface AttachmentListProps {
  items: PendingAttachment[];
  itemNodes: Map<string, HTMLLIElement>;
  onRemove: (item: PendingAttachment) => void;
  /** Durante o envio: Remover fica indisponível e cada item mostra a sua barra. */
  progress: Record<string, number> | null;
}

function AttachmentList({ items, itemNodes, onRemove, progress }: AttachmentListProps) {
  if (items.length === 0) return null;
  return (
    <ul aria-label="Anexos" className={styles.list}>
      {items.map((item) => (
        <li
          key={item.id}
          className={styles.item}
          ref={(node) => {
            if (node) itemNodes.set(item.id, node);
            else itemNodes.delete(item.id);
          }}
        >
          <span className={styles.thumbnail}>{item.preview ? <img src={item.preview} alt="" /> : null}</span>
          <span className={styles.description}>
            <span className={styles.name}>{item.label}</span>
            <span className={styles.size}>{formatSize(item.file.size)}</span>
          </span>
          <Button size="small" aria-label={`Remover ${item.label}`} disabled={progress !== null} onClick={() => onRemove(item)}>Remover</Button>
          {progress ? (
            <span className={styles.progress}>
              <ProgressBar label={`Enviando ${item.label}`} value={progress[item.id] ?? 0} />
            </span>
          ) : null}
        </li>
      ))}
    </ul>
  );
}

export interface QuestionComposerProps {
  limits: AttachmentLimits | null;
  /** Devolve `null` quando a pergunta foi aceita, ou o motivo da falha; `report` informa o progresso de cada arquivo. */
  onSubmit: (text: string, attachments: PendingAttachment[], report: (id: string, percent: number) => void) => Promise<string | null>;
}

export function QuestionComposer({ limits, onSubmit }: QuestionComposerProps) {
  const [tab, setTab] = useState('escrever');
  const [text, setText] = useState('');
  const [items, setItems] = useState<PendingAttachment[]>([]);
  const [messages, setMessages] = useState<string[]>([]);
  const [sending, setSending] = useState(false);
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [failure, setFailure] = useState<string | null>(null);
  const textarea = useRef<HTMLTextAreaElement>(null);
  const submitRow = useRef<HTMLDivElement>(null);
  const afterSend = useRef<'text' | 'send' | null>(null);
  const announced = useRef('');
  const reasonId = useId();
  const fileInput = useRef<HTMLInputElement>(null);
  const attachRow = useRef<HTMLDivElement>(null);
  const itemNodes = useRef(new Map<string, HTMLLIElement>());
  const pendingFocus = useRef<string | typeof FOCUS_ATTACH | null>(null);
  const latestItems = useRef(items);
  latestItems.current = items;
  const empty = text.trim() === '';

  useEffect(() => {
    const keepPage = (event: Event) => event.preventDefault();
    window.addEventListener('dragover', keepPage);
    window.addEventListener('drop', keepPage);
    return () => {
      window.removeEventListener('dragover', keepPage);
      window.removeEventListener('drop', keepPage);
    };
  }, []);

  useEffect(() => () => latestItems.current.forEach(revokePreview), []);

  useLayoutEffect(() => {
    const target = pendingFocus.current;
    if (target === null) return;
    pendingFocus.current = null;
    const holder = target === FOCUS_ATTACH ? attachRow.current : itemNodes.current.get(target);
    holder?.querySelector('button')?.focus();
  }, [items]);

  useLayoutEffect(() => {
    const target = afterSend.current;
    if (sending || target === null) return;
    afterSend.current = null;
    (target === 'text' ? textarea.current : submitRow.current?.querySelector('button'))?.focus();
  }, [sending]);

  function add(candidates: AttachmentCandidate[]) {
    if (sending || candidates.length === 0) return;
    const { next, refused } = addAttachments(items, candidates, limits);
    const added = next.slice(items.length).map((item) => `${item.label} anexado.`);
    setItems(next);
    setMessages([...added, ...refused]);
  }

  function report(id: string, percent: number) {
    setProgress((current) => ({ ...current, [id]: percent }));
    if (announced.current === id) return;
    announced.current = id;
    const item = latestItems.current.find((other) => other.id === id);
    if (item) setMessages([`Enviando ${item.label}.`]);
  }

  async function submit() {
    if (sending) return;
    const sent = items;
    announced.current = '';
    setFailure(null);
    setMessages([]);
    setProgress({});
    setSending(true);
    let reason: string | null;
    try {
      reason = (await onSubmit(text, sent, report)) ?? null;
    } catch (error) {
      reason = error instanceof Error ? error.message : 'Não foi possível enviar a pergunta.';
    }
    setProgress({});
    if (reason === null) {
      sent.forEach(revokePreview);
      setItems([]);
      setText('');
      setTab('escrever');
      setMessages(['Pergunta enviada.']);
    } else {
      setFailure(reason);
      setMessages([]);
    }
    afterSend.current = reason === null ? 'text' : 'send';
    setSending(false);
  }

  function remove(item: PendingAttachment) {
    const index = items.indexOf(item);
    pendingFocus.current = (items[index + 1] ?? items[index - 1])?.id ?? FOCUS_ATTACH;
    revokePreview(item);
    setItems(items.filter((other) => other !== item));
    setMessages([`${item.label} removido.`]);
  }

  function onPaste(event: ClipboardEvent<HTMLTextAreaElement>) {
    const candidates = event.clipboardData ? pastedFiles(event.clipboardData) : [];
    if (candidates.length === 0) return;
    event.preventDefault();
    add(candidates);
  }

  function onDrop(event: DragEvent<HTMLDivElement>) {
    if (event.dataTransfer) add(transferCandidates(event.dataTransfer));
  }

  function onFilesChosen(event: ChangeEvent<HTMLInputElement>) {
    const input = event.target;
    add(Array.from(input.files ?? []).map((file) => ({ file, isDirectory: false })));
    input.value = '';
  }

  const writePanel = (
    <label className={styles.field}>
      <span className={styles.label}>Sua pergunta</span>
      <textarea ref={textarea} rows={10} disabled={sending} className={styles.control} value={text} onInput={(event) => setText((event.target as HTMLTextAreaElement).value)} onPaste={onPaste} />
    </label>
  );
  const previewPanel = <div className={styles.preview}>{empty ? <p className={styles.previewEmpty}>Nada para mostrar ainda.</p> : <Markdown source={text} untrusted />}</div>;

  return (
    <div className={styles.composer} onDragOver={(event) => event.preventDefault()} onDrop={onDrop}>
      {/* O fieldset desabilitado tira as abas e o campo da interação enquanto os arquivos sobem. */}
      <fieldset disabled={sending} className={styles.group}>
        <Tabs
          label="Campo da pergunta"
          value={tab}
          onChange={setTab}
          tabs={[
            { id: 'escrever', label: 'Escrever', panel: writePanel },
            { id: 'previa', label: 'Prévia', panel: previewPanel },
          ]}
        />
      </fieldset>
      <div ref={attachRow} className={styles.attachRow}>
        <Button disabled={sending} onClick={() => fileInput.current?.click()}>Anexar</Button>
        <input ref={fileInput} type="file" multiple hidden onChange={onFilesChosen} />
      </div>
      <AttachmentList items={items} itemNodes={itemNodes.current} onRemove={remove} progress={sending ? progress : null} />
      <p role="status" aria-live="polite" className={styles.status}>{messages.join(' ')}</p>
      {failure ? <p role="alert" className={styles.failure}>{failure}</p> : null}
      <div ref={submitRow} className={styles.submitRow}>
        {/* O nome falado diz o que vai: o texto visível sozinho não cita a pergunta. */}
        <Button variant="primary" aria-label="Enviar pergunta" disabled={empty} loading={sending} aria-describedby={empty ? reasonId : undefined} onClick={() => void submit()}>Enviar</Button>
        {empty ? <span id={reasonId} className={styles.reason}>{TEXT_REQUIRED}</span> : null}
      </div>
    </div>
  );
}
