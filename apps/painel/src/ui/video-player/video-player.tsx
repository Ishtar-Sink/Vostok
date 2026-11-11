import type { ComponentPropsWithRef, KeyboardEvent as ReactKeyboardEvent, ReactNode } from 'react';
import { useEffect, useRef, useState } from 'react';
import styles from './video-player.module.css';

const SPEEDS = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2];
const FRAME_SECONDS = 1 / 30;

export interface VideoPlayerProps extends Omit<ComponentPropsWithRef<'div'>, 'children'> {
  src: string;
  /** Nome acessível do player. */
  label: string;
}

/** Formata segundos como `m:ss`, ou `h:mm:ss` a partir de uma hora. */
export function formatTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return '0:00';
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const rest = String(total % 60).padStart(2, '0');
  return hours > 0 ? `${hours}:${String(minutes).padStart(2, '0')}:${rest}` : `${minutes}:${rest}`;
}

const speedLabel = (speed: number) => `${String(speed).replace('.', ',')}×`;

function Icon({ children }: { children: ReactNode }) {
  return (
    <span aria-hidden="true" className={styles.icon}>
      <svg viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        {children}
      </svg>
    </span>
  );
}

export function VideoPlayer({ src, label, className, ...rest }: VideoPlayerProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);
  const [current, setCurrent] = useState(0);
  const [duration, setDuration] = useState(0);
  const [fullscreen, setFullscreen] = useState(false);
  const [speed, setSpeed] = useState(1);

  useEffect(() => {
    const sync = () => setFullscreen(document.fullscreenElement === rootRef.current);
    document.addEventListener('fullscreenchange', sync);
    return () => document.removeEventListener('fullscreenchange', sync);
  }, []);

  function readDuration() {
    const value = videoRef.current?.duration ?? 0;
    setDuration(Number.isFinite(value) ? value : 0);
  }

  function toggle() {
    const video = videoRef.current;
    if (!video) return;
    if (playing) video.pause();
    else void video.play();
  }

  function stepFrame(direction: 1 | -1) {
    const video = videoRef.current;
    if (!video) return;
    video.pause();
    const limit = Number.isFinite(video.duration) ? video.duration : 0;
    video.currentTime = Math.min(limit, Math.max(0, video.currentTime + direction * FRAME_SECONDS));
    setCurrent(video.currentTime);
  }

  function seek(value: string) {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = Number(value);
    setCurrent(video.currentTime);
  }

  function toggleFullscreen() {
    if (fullscreen) void document.exitFullscreen();
    else void rootRef.current?.requestFullscreen();
  }

  function cycleSpeed() {
    const next = SPEEDS[(SPEEDS.indexOf(speed) + 1) % SPEEDS.length];
    setSpeed(next);
    if (videoRef.current) videoRef.current.playbackRate = next;
  }

  function onKeyDown(event: ReactKeyboardEvent) {
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT') return;
    if (event.key === ' ' && target.tagName !== 'BUTTON') toggle();
    else if (event.key === 'ArrowLeft') stepFrame(-1);
    else if (event.key === 'ArrowRight') stepFrame(1);
    else return;
    event.preventDefault();
  }

  return (
    <div {...rest} ref={rootRef} role="group" aria-label={label} tabIndex={0} data-playing={playing} className={[styles.root, className].filter(Boolean).join(' ')} onKeyDown={onKeyDown}>
      <video
        ref={videoRef}
        src={src}
        preload="metadata"
        playsInline
        className={styles.video}
        onClick={toggle}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onTimeUpdate={() => setCurrent(videoRef.current?.currentTime ?? 0)}
        onLoadedMetadata={readDuration}
        onDurationChange={readDuration}
      />
      <div data-theme="dark" className={styles.controls}>
        <button type="button" aria-label={playing ? 'Pausar' : 'Tocar'} className={styles.action} onClick={toggle}>
          <Icon>{playing ? <path d="M5 3v10M11 3v10" /> : <path d="M5 3l8 5-8 5z" />}</Icon>
        </button>
        <button type="button" aria-label="Quadro anterior" className={styles.action} onClick={() => stepFrame(-1)}>
          <Icon><path d="M10 3L5 8l5 5" /></Icon>
        </button>
        <button type="button" aria-label="Próximo quadro" className={styles.action} onClick={() => stepFrame(1)}>
          <Icon><path d="M6 3l5 5-5 5" /></Icon>
        </button>
        <input
          type="range"
          aria-label="Progresso"
          className={styles.range}
          min={0}
          max={duration || 0}
          step="any"
          value={current}
          aria-valuetext={`${formatTime(current)} de ${formatTime(duration)}`}
          onInput={(event) => seek(event.currentTarget.value)}
        />
        <span className={styles.time}>{formatTime(current)} / {formatTime(duration)}</span>
        <button type="button" aria-label={`Velocidade ${speedLabel(speed)}`} className={`${styles.action} ${styles.speed}`} onClick={cycleSpeed}>
          {speedLabel(speed)}
        </button>
        <button type="button" aria-label={fullscreen ? 'Sair da tela cheia' : 'Tela cheia'} className={styles.action} onClick={toggleFullscreen}>
          <Icon>{fullscreen ? <path d="M6 2.5V6H2.5M10 2.5V6h3.5M6 13.5V10H2.5M10 13.5V10h3.5" /> : <path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10" />}</Icon>
        </button>
      </div>
    </div>
  );
}
