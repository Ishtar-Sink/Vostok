import { useEffect, useRef } from 'react';
import { fileUrl, formatDateTime } from '../../../shared/api';
import { CopyButton } from '../../../ui/copy-button/copy-button';
import { ImageViewer } from '../../../ui/image-viewer/image-viewer';
import { VideoPlayer } from '../../../ui/video-player/video-player';
import { IMAGE, VIDEO, type EvidenceEntry } from '../evidence-list/evidence-items';
import { stripAnsi } from '../snippet-match';
import styles from './evidence-review.module.css';

export const TEXT = /\.(md|txt|yml|log|out|diff|csv|json|css|js)$/i;

/** As quatro primeiras linhas de um log gravado pelo harness: comando, data, commit e código de saída. */
const LOG_HEADER = [/^\$ /, /^# data:/, /^# commit:/, /^# código de saída:/];
const EXIT_CODE = /^# código de saída: (\d+)/;

export interface TextState {
  value: string | null;
  total: number | null;
  failed: boolean;
}

export function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1).replace('.', ',')} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1).replace('.', ',')} MB`;
}

function diffKind(line: string): 'add' | 'del' | undefined {
  if (line.startsWith('+') && !line.startsWith('+++')) return 'add';
  if (line.startsWith('-') && !line.startsWith('---')) return 'del';
  return undefined;
}

function LogLines({ text, hits, diff }: { text: string; hits: Set<number>; diff: boolean }) {
  const preRef = useRef<HTMLPreElement>(null);
  const first = hits.size > 0 ? Math.min(...hits) : null;

  useEffect(() => {
    if (first === null) return;
    preRef.current?.querySelector(`[data-line="${first}"]`)?.scrollIntoView({ block: 'center' });
  }, [first, hits]);

  const lines = text.endsWith('\n') ? text.slice(0, -1).split('\n') : text.split('\n');
  return (
    <pre ref={preRef} className={styles.pre}>
      {lines.map((line, index) => {
        const head = index < LOG_HEADER.length && LOG_HEADER[index].test(line);
        return (
          <span
            key={index}
            data-line={index}
            data-head={head ? '' : undefined}
            className={[styles.line, hits.has(index) ? styles.hit : ''].filter(Boolean).join(' ')}
          >
            <span className={styles.lineNumber} aria-hidden="true">
              {index + 1}
            </span>
            <span className={styles.lineText} data-diff={diff && !head ? diffKind(line) : undefined}>
              {stripAnsi(line)}
            </span>
          </span>
        );
      })}
    </pre>
  );
}

/** A evidência aberta no lado direito do modal: imagem, vídeo, log numerado ou o link de baixar. */
export function EvidenceViewer({ entry, text, hits }: { entry: EvidenceEntry; text: TextState; hits: Set<number> }) {
  const { file, manual, name } = entry;
  const exitCode = text.value ? EXIT_CODE.exec(text.value.split('\n', 4)[3] ?? '')?.[1] : undefined;
  const reviewedAt = entry.review?.reviewed_at;

  function body() {
    if (manual && !manual.attached) return <p className={styles.missing}>Falta anexar: {manual.description}</p>;
    if (!file) return <p className={styles.missing}>O arquivo da evidência não foi encontrado.</p>;
    const url = fileUrl(file.path);
    if (IMAGE.test(name)) {
      return (
        <div className={styles.media}>
          <ImageViewer src={url} alt={`Evidência ${name}`} />
        </div>
      );
    }
    if (VIDEO.test(name)) {
      return (
        <div className={styles.media}>
          <VideoPlayer src={url} label={`Vídeo ${name}`} />
        </div>
      );
    }
    if (!TEXT.test(name)) {
      return (
        <div className={styles.media}>
          <a className={styles.download} href={url} download={name} aria-label={`Baixar ${name}`}>
            Baixar {name}
          </a>
        </div>
      );
    }
    return (
      <div className={styles.log}>
        {text.value !== null ? <CopyButton className={styles.copy} text={text.value} label={`Copiar ${name}`} /> : null}
        {text.failed ? (
          <p className={styles.missing}>Não foi possível ler o arquivo.</p>
        ) : text.value === null ? (
          <p className={styles.missing}>Carregando…</p>
        ) : (
          <LogLines text={text.value} hits={hits} diff={entry.kindLabel === 'Diff'} />
        )}
        {text.total !== null ? (
          <p className={styles.partial} role="status">
            Mostrando o primeiro 1 MB de {formatSize(text.total)}.{' '}
            <a className={styles.download} href={url} download={name}>
              Baixar o arquivo inteiro
            </a>
          </p>
        ) : null}
      </div>
    );
  }

  return (
    <div className={styles.viewer}>
      <div className={styles.bar}>
        <span className={styles.fileName}>{name}</span>
        {file && (!manual || manual.attached) ? <span className={styles.size}>{formatSize(file.size)}</span> : null}
        <span className={styles.barEnd}>
          {exitCode !== undefined ? <span>{`código de saída ${exitCode}`}</span> : null}
          {reviewedAt ? <span>{formatDateTime(reviewedAt)}</span> : null}
        </span>
      </div>
      {body()}
    </div>
  );
}
