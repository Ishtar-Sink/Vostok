import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { VideoPlayer } from './video-player';

const play = vi.fn(() => Promise.resolve());
const pause = vi.fn();

beforeAll(() => {
  HTMLMediaElement.prototype.play = play;
  HTMLMediaElement.prototype.pause = pause;
});
afterEach(() => {
  cleanup();
  play.mockClear();
  pause.mockClear();
});

function setup() {
  const { container } = render(<VideoPlayer src="/x/a.mp4" label="Vídeo a.mp4" />);
  const video = container.querySelector('video') as HTMLVideoElement;
  const group = screen.getByRole('group', { name: 'Vídeo a.mp4' });
  return { video, group };
}
function load(video: HTMLVideoElement, duration: number) {
  Object.defineProperty(video, 'duration', { value: duration, configurable: true });
  fireEvent(video, new Event('loadedmetadata'));
}

describe('VideoPlayer', () => {
  it('toca e pausa pelo botão e pela barra de espaço', () => {
    const { video, group } = setup();
    fireEvent.click(screen.getByRole('button', { name: 'Tocar' }));
    expect(play).toHaveBeenCalled();
    fireEvent(video, new Event('play'));
    expect(screen.getByRole('button', { name: 'Pausar' })).toBeTruthy();
    const notCancelled = fireEvent.keyDown(group, { key: ' ' });
    expect(pause).toHaveBeenCalled();
    expect(notCancelled).toBe(false);
  });

  it('mostra o progresso com o tempo atual e a duração', () => {
    const { video } = setup();
    load(video, 80);
    video.currentTime = 5;
    fireEvent(video, new Event('timeupdate'));
    expect(screen.getByText('0:05 / 1:20')).toBeTruthy();
    const slider = screen.getByRole('slider', { name: 'Progresso' });
    expect(slider.getAttribute('aria-valuetext')).toBe('0:05 de 1:20');
    fireEvent.input(slider, { target: { value: '40' } });
    expect(video.currentTime).toBe(40);
  });

  it('alterna a velocidade entre 0,25× e 2×', () => {
    const { video } = setup();
    const button = () => screen.getByRole('button', { name: /^Velocidade/ });
    expect(button().textContent).toBe('1×');
    expect(button().getAttribute('aria-label')).toBe('Velocidade 1×');
    for (const [label, rate] of [['1,25×', 1.25], ['1,5×', 1.5], ['2×', 2], ['0,25×', 0.25]] as const) {
      fireEvent.click(button());
      expect(button().textContent).toBe(label);
      expect(video.playbackRate).toBe(rate);
    }
  });

  it('avança e volta um quadro pelos botões e pelas setas', () => {
    const { video, group } = setup();
    load(video, 80);
    video.currentTime = 1;
    fireEvent.click(screen.getByRole('button', { name: 'Próximo quadro' }));
    expect(video.currentTime).toBeCloseTo(1 + 1 / 30);
    expect(pause).toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: 'Quadro anterior' }));
    expect(video.currentTime).toBeCloseTo(1);
    expect(fireEvent.keyDown(group, { key: 'ArrowRight' })).toBe(false);
    expect(video.currentTime).toBeCloseTo(1 + 1 / 30);
    expect(fireEvent.keyDown(group, { key: 'ArrowLeft' })).toBe(false);
    expect(video.currentTime).toBeCloseTo(1);
  });

  it('não passa do início nem do fim ao andar quadro a quadro', () => {
    const { video } = setup();
    load(video, 10);
    video.currentTime = 0;
    fireEvent.click(screen.getByRole('button', { name: 'Quadro anterior' }));
    expect(video.currentTime).toBe(0);
    video.currentTime = 10;
    fireEvent.click(screen.getByRole('button', { name: 'Próximo quadro' }));
    expect(video.currentTime).toBe(10);
  });

  it('não usa as setas do grupo dentro do slider nem a barra de espaço nos botões', () => {
    const { video } = setup();
    load(video, 80);
    video.currentTime = 1;
    const notCancelled = fireEvent.keyDown(screen.getByRole('slider', { name: 'Progresso' }), { key: 'ArrowRight' });
    expect(video.currentTime).toBe(1);
    expect(notCancelled).toBe(true);
    expect(fireEvent.keyDown(screen.getByRole('button', { name: 'Tocar' }), { key: ' ' })).toBe(true);
    expect(play).not.toHaveBeenCalled();
  });

  it('entra e sai da tela cheia', () => {
    const { group } = setup();
    const request = vi.fn(() => Promise.resolve());
    const exit = vi.fn(() => Promise.resolve());
    (group as unknown as { requestFullscreen: unknown }).requestFullscreen = request;
    Object.defineProperty(document, 'exitFullscreen', { value: exit, configurable: true });
    fireEvent.click(screen.getByRole('button', { name: 'Tela cheia' }));
    expect(request).toHaveBeenCalled();
    Object.defineProperty(document, 'fullscreenElement', { value: group, configurable: true });
    fireEvent(document, new Event('fullscreenchange'));
    fireEvent.click(screen.getByRole('button', { name: 'Sair da tela cheia' }));
    expect(exit).toHaveBeenCalled();
    Object.defineProperty(document, 'fullscreenElement', { value: null, configurable: true });
  });

  it('tem nome acessível no grupo e em cada controle', () => {
    const { video } = setup();
    expect(video.hasAttribute('controls')).toBe(false);
    for (const name of ['Tocar', 'Quadro anterior', 'Próximo quadro', 'Velocidade 1×', 'Tela cheia']) {
      expect(screen.getByRole('button', { name })).toBeTruthy();
    }
    expect(screen.getByRole('slider', { name: 'Progresso' })).toBeTruthy();
    expect(screen.queryByRole('combobox')).toBeNull();
  });
});
