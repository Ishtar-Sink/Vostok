import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { ImageViewer } from './image-viewer';

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const THUMB = 'Ampliar Evidência tela.png';

function openViewer() {
  render(<ImageViewer src="/x/tela.png" alt="Evidência tela.png" />);
  fireEvent.click(screen.getByRole('button', { name: THUMB }));
  return screen.getByRole('dialog', { name: 'Evidência tela.png' });
}
const zoomText = () => screen.getByRole('status').textContent;
const image = (dialog: HTMLElement) => dialog.querySelector('img') as HTMLImageElement;
const frameOf = (dialog: HTMLElement) => image(dialog).parentElement as HTMLElement;
const stageOf = (dialog: HTMLElement) => frameOf(dialog).parentElement as HTMLElement;
const transformOf = (dialog: HTMLElement) => image(dialog).style.transform;

describe('ImageViewer', () => {
  it('mostra a miniatura com o nome acessível de ampliar', () => {
    render(<ImageViewer src="/x/tela.png" alt="Evidência tela.png" />);
    const button = screen.getByRole('button', { name: THUMB });
    expect(button.contains(screen.getByAltText('Evidência tela.png'))).toBe(true);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('abre só a imagem, sem barra de ferramentas', () => {
    const dialog = openViewer();
    expect(dialog.hasAttribute('open')).toBe(true);
    expect(dialog.getAttribute('data-theme')).toBe('dark');
    expect(image(dialog).getAttribute('src')).toBe('/x/tela.png');
    expect(zoomText()).toBe('100%');
    const buttons = Array.from(dialog.querySelectorAll('button'));
    expect(buttons.map((button) => button.getAttribute('aria-label'))).toEqual(['Fechar']);
  });

  it('dá zoom no ponto clicado e volta ao tamanho original no segundo clique', () => {
    const dialog = openViewer();
    expect(image(dialog).hasAttribute('data-zoomed')).toBe(false);
    fireEvent.click(image(dialog), { clientX: 10, clientY: 20 });
    expect(zoomText()).toBe('200%');
    expect(image(dialog).hasAttribute('data-zoomed')).toBe(true);
    expect(transformOf(dialog)).toBe('translate(-10px, -20px) scale(2)');
    fireEvent.click(image(dialog));
    expect(zoomText()).toBe('100%');
    expect(image(dialog).hasAttribute('data-zoomed')).toBe(false);
    expect(transformOf(dialog)).toBe('translate(0px, 0px) scale(1)');
  });

  it('aumenta, diminui e ajusta o zoom pelo teclado, dentro dos limites', () => {
    const dialog = openViewer();
    fireEvent.keyDown(dialog, { key: '+' });
    expect(zoomText()).toBe('125%');
    fireEvent.keyDown(dialog, { key: '-' });
    expect(zoomText()).toBe('100%');
    fireEvent.keyDown(dialog, { key: '-' });
    expect(zoomText()).toBe('100%');
    fireEvent.keyDown(dialog, { key: '+' });
    fireEvent.keyDown(dialog, { key: '+' });
    fireEvent.keyDown(dialog, { key: '0' });
    expect(zoomText()).toBe('100%');
    for (let i = 0; i < 12; i += 1) fireEvent.keyDown(dialog, { key: '+' });
    expect(zoomText()).toBe('800%');
  });

  it('arrasta a imagem ampliada e não confunde o arraste com clique', () => {
    const dialog = openViewer();
    const stage = stageOf(dialog);
    fireEvent.pointerDown(stage, { clientX: 10, clientY: 10, pointerId: 1 });
    fireEvent.pointerMove(stage, { clientX: 40, clientY: 30, pointerId: 1 });
    fireEvent.pointerUp(stage, { pointerId: 1 });
    expect(transformOf(dialog)).toContain('translate(0px, 0px)');
    fireEvent.click(image(dialog));
    expect(zoomText()).toBe('200%');
    fireEvent.pointerDown(stage, { clientX: 10, clientY: 10, pointerId: 1 });
    fireEvent.pointerMove(stage, { clientX: 40, clientY: 30, pointerId: 1 });
    fireEvent.pointerUp(stage, { pointerId: 1 });
    expect(transformOf(dialog)).toContain('translate(30px, 20px)');
    fireEvent.click(stage);
    fireEvent.click(image(dialog));
    expect(zoomText()).toBe('200%');
    expect(screen.getByRole('dialog')).toBeTruthy();
  });

  it('move a imagem ampliada com as setas', () => {
    const dialog = openViewer();
    fireEvent.click(image(dialog));
    const notCancelled = fireEvent.keyDown(dialog, { key: 'ArrowRight' });
    expect(transformOf(dialog)).toContain('translate(-40px, 0px)');
    expect(notCancelled).toBe(false);
  });

  it('fecha com um clique no fundo fora da imagem', () => {
    const dialog = openViewer();
    fireEvent.click(image(dialog));
    expect(screen.getByRole('dialog')).toBeTruthy();
    fireEvent.click(image(dialog));
    fireEvent.click(stageOf(dialog));
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('fecha com Esc e devolve o foco à miniatura', () => {
    const dialog = openViewer();
    fireEvent.click(image(dialog));
    fireEvent.keyDown(dialog, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: THUMB }));
    fireEvent.click(screen.getByRole('button', { name: THUMB }));
    expect(zoomText()).toBe('100%');
  });

  it('fecha pelo botão Fechar', () => {
    openViewer();
    fireEvent.click(screen.getByRole('button', { name: 'Fechar' }));
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  describe('animação', () => {
    function animate(reduced = false) {
      vi.stubGlobal('matchMedia', () => ({ matches: reduced }));
      vi.spyOn(Element.prototype, 'getBoundingClientRect').mockImplementation(function rect(this: Element) {
        const thumb = this.tagName === 'BUTTON' && this.getAttribute('aria-label')?.startsWith('Ampliar');
        return (thumb ? { left: 0, top: 0, width: 100, height: 50 } : { left: 200, top: 100, width: 400, height: 200 }) as DOMRect;
      });
    }

    it('abre crescendo a partir da miniatura', () => {
      animate();
      const dialog = openViewer();
      const frame = frameOf(dialog);
      expect(frame.getAttribute('data-phase')).toBe('in');
      expect(frame.style.getPropertyValue('--from-scale')).toBe('0.25');
      expect(frame.style.getPropertyValue('--from-x')).toBe('-350px');
      expect(frame.style.getPropertyValue('--from-y')).toBe('-175px');
    });

    it('espera a animação terminar antes de fechar', () => {
      animate();
      const dialog = openViewer();
      fireEvent.keyDown(dialog, { key: 'Escape' });
      expect(screen.getByRole('dialog')).toBeTruthy();
      expect(frameOf(dialog).getAttribute('data-phase')).toBe('out');
      expect(document.activeElement).not.toBe(screen.getByRole('button', { name: THUMB }));
      fireEvent.animationEnd(frameOf(dialog));
      expect(screen.queryByRole('dialog')).toBeNull();
      expect(document.activeElement).toBe(screen.getByRole('button', { name: THUMB }));
    });

    it('não anima com prefers-reduced-motion', () => {
      animate(true);
      const dialog = openViewer();
      expect(frameOf(dialog).hasAttribute('data-phase')).toBe(false);
      fireEvent.keyDown(dialog, { key: 'Escape' });
      expect(screen.queryByRole('dialog')).toBeNull();
    });
  });
});
