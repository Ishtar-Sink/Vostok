import type { ComponentPropsWithRef, CSSProperties, KeyboardEvent as ReactKeyboardEvent, MouseEvent as ReactMouseEvent, PointerEvent as ReactPointerEvent, WheelEvent as ReactWheelEvent } from 'react';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import styles from './image-viewer.module.css';

const ZOOM_MIN = 1;
const ZOOM_MAX = 8;
const ZOOM_STEP = 1.25;
const ZOOM_CLICK = 2;
const PAN_STEP = 40;
const DRAG_THRESHOLD = 3;
const CLOSE_FALLBACK_MS = 600;

export interface ImageViewerProps extends Omit<ComponentPropsWithRef<'button'>, 'children' | 'onClick'> {
  src: string;
  /** Texto da imagem e nome do diálogo. */
  alt: string;
}

interface Point {
  x: number;
  y: number;
}

/** Posição e escala da miniatura em relação à imagem centralizada, de onde ela cresce e para onde volta. */
interface Origin extends Point {
  scale: number;
}

function reducedMotion(): boolean {
  return typeof window.matchMedia !== 'function' || window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

const ORIGIN: Point = { x: 0, y: 0 };
const ARROW_PAN: Record<string, Point> = {
  ArrowLeft: { x: PAN_STEP, y: 0 },
  ArrowRight: { x: -PAN_STEP, y: 0 },
  ArrowUp: { x: 0, y: PAN_STEP },
  ArrowDown: { x: 0, y: -PAN_STEP },
};

function clampZoom(value: number): number {
  return Math.round(Math.min(ZOOM_MAX, Math.max(ZOOM_MIN, value)) * 100) / 100;
}

export function ImageViewer({ src, alt, className, ...rest }: ImageViewerProps) {
  const [open, setOpen] = useState(false);
  const [zoom, setZoom] = useState(ZOOM_MIN);
  const [offset, setOffset] = useState<Point>(ORIGIN);
  const [dragging, setDragging] = useState(false);
  const [origin, setOrigin] = useState<Origin | null>(null);
  const [phase, setPhase] = useState<'in' | 'out'>('in');
  const triggerRef = useRef<HTMLButtonElement>(null);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const closeRef = useRef<HTMLButtonElement>(null);
  const stageRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<HTMLDivElement>(null);
  const dragFrom = useRef<Point | null>(null);
  const dragged = useRef(0);

  useEffect(() => {
    if (!open) return undefined;
    dialogRef.current?.showModal();
    closeRef.current?.focus();
    return () => {
      dialogRef.current?.close();
      triggerRef.current?.focus();
    };
  }, [open]);

  useLayoutEffect(() => {
    if (open && !reducedMotion()) setOrigin(measureOrigin());
  }, [open]);

  useEffect(() => {
    if (phase !== 'out') return undefined;
    const timer = window.setTimeout(finish, CLOSE_FALLBACK_MS);
    return () => window.clearTimeout(timer);
  }, [phase]);

  function measureOrigin(): Origin | null {
    const thumb = triggerRef.current?.getBoundingClientRect();
    const frame = frameRef.current?.getBoundingClientRect();
    if (!thumb || !frame || frame.width === 0) return null;
    return {
      x: thumb.left + thumb.width / 2 - (frame.left + frame.width / 2),
      y: thumb.top + thumb.height / 2 - (frame.top + frame.height / 2),
      scale: thumb.width / frame.width,
    };
  }

  function finish() {
    setOpen(false);
    setOrigin(null);
    setPhase('in');
    setZoom(ZOOM_MIN);
    setOffset(ORIGIN);
  }

  /** Com animação, o diálogo só sai do documento quando o quadro termina de voltar à miniatura. */
  function close() {
    if (phase === 'out') return;
    const target = reducedMotion() ? null : measureOrigin();
    if (!target) {
      finish();
      return;
    }
    setOrigin(target);
    setPhase('out');
    setZoom(ZOOM_MIN);
    setOffset(ORIGIN);
  }

  /** Ponto do cursor medido a partir do centro do palco, onde a imagem está centrada. */
  function fromCenter(clientX: number, clientY: number): Point {
    const box = stageRef.current?.getBoundingClientRect();
    return { x: clientX - (box ? box.left + box.width / 2 : 0), y: clientY - (box ? box.top + box.height / 2 : 0) };
  }

  function zoomTo(target: number, at: Point = ORIGIN) {
    const next = clampZoom(target);
    if (next === ZOOM_MIN) {
      setOffset(ORIGIN);
    } else {
      const ratio = next / zoom;
      setOffset({ x: at.x - (at.x - offset.x) * ratio, y: at.y - (at.y - offset.y) * ratio });
    }
    setZoom(next);
  }

  function pan(delta: Point) {
    setOffset((current) => ({ x: current.x + delta.x, y: current.y + delta.y }));
  }

  function onKeyDown(event: ReactKeyboardEvent) {
    const arrow = ARROW_PAN[event.key];
    if (event.key === 'Escape') close();
    else if (event.key === '+' || event.key === '=') zoomTo(zoom * ZOOM_STEP);
    else if (event.key === '-') zoomTo(zoom / ZOOM_STEP);
    else if (event.key === '0') zoomTo(ZOOM_MIN);
    else if (arrow && zoom > ZOOM_MIN) pan(arrow);
    else return;
    event.preventDefault();
  }

  function onPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    dragged.current = 0;
    if (zoom <= ZOOM_MIN) return;
    dragFrom.current = { x: event.clientX, y: event.clientY };
    setDragging(true);
    event.currentTarget.setPointerCapture?.(event.pointerId);
  }

  function onPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    const from = dragFrom.current;
    if (!from) return;
    const delta = { x: event.clientX - from.x, y: event.clientY - from.y };
    dragged.current += Math.abs(delta.x) + Math.abs(delta.y);
    pan(delta);
    dragFrom.current = { x: event.clientX, y: event.clientY };
  }

  function onPointerUp() {
    dragFrom.current = null;
    setDragging(false);
  }

  function onWheel(event: ReactWheelEvent<HTMLDivElement>) {
    event.preventDefault();
    zoomTo(event.deltaY < 0 ? zoom * ZOOM_STEP : zoom / ZOOM_STEP, fromCenter(event.clientX, event.clientY));
  }

  function onImageClick(event: ReactMouseEvent<HTMLImageElement>) {
    event.stopPropagation();
    if (dragged.current > DRAG_THRESHOLD) return;
    if (zoom === ZOOM_MIN) zoomTo(ZOOM_CLICK, fromCenter(event.clientX, event.clientY));
    else zoomTo(ZOOM_MIN);
  }

  function onStageClick(event: ReactMouseEvent<HTMLDivElement>) {
    if (event.target === event.currentTarget && dragged.current <= DRAG_THRESHOLD) close();
  }

  const classes = [styles.trigger, className].filter(Boolean).join(' ');
  return (
    <>
      <button {...rest} ref={triggerRef} type="button" aria-label={`Ampliar ${alt}`} className={classes} onClick={() => setOpen(true)}>
        <img src={src} alt={alt} className={styles.thumb} />
      </button>
      {open ? (
        <dialog
          ref={dialogRef}
          aria-label={alt}
          data-theme="dark"
          data-phase={origin ? phase : undefined}
          className={styles.dialog}
          onKeyDown={onKeyDown}
          onCancel={(event) => {
            event.preventDefault();
            close();
          }}
        >
          <div
            ref={stageRef}
            className={[styles.stage, dragging ? styles.dragging : ''].filter(Boolean).join(' ')}
            onClick={onStageClick}
            onPointerDown={onPointerDown}
            onPointerMove={onPointerMove}
            onPointerUp={onPointerUp}
            onWheel={onWheel}
          >
            <div
              ref={frameRef}
              className={styles.frame}
              data-phase={origin ? phase : undefined}
              style={origin ? ({ '--from-x': `${origin.x}px`, '--from-y': `${origin.y}px`, '--from-scale': origin.scale } as CSSProperties) : undefined}
              onAnimationEnd={(event) => {
                if (phase === 'out' && event.target === event.currentTarget) finish();
              }}
            >
              <img
                src={src}
                alt={alt}
                draggable={false}
                data-zoomed={zoom > ZOOM_MIN ? '' : undefined}
                className={styles.image}
                style={{ transform: `translate(${offset.x}px, ${offset.y}px) scale(${zoom})` }}
                onClick={onImageClick}
              />
            </div>
          </div>
          <button ref={closeRef} type="button" aria-label="Fechar" className={styles.close} onClick={close}>
            <span aria-hidden="true" className={styles.icon}>
              <svg viewBox="0 0 16 16" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                <path d="M3.5 3.5l9 9M12.5 3.5l-9 9" />
              </svg>
            </span>
          </button>
          <output aria-live="polite" className={styles.visuallyHidden}>{Math.round(zoom * 100)}%</output>
        </dialog>
      ) : null}
    </>
  );
}
