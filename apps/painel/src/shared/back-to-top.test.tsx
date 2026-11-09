import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { BackToTop } from './back-to-top';

function defineWindow(name: string, value: number) {
  Object.defineProperty(window, name, { configurable: true, value });
}

function scrollTo(y: number) {
  defineWindow('scrollY', y);
  fireEvent.scroll(window);
}

function mockMotion(reduced: boolean) {
  window.matchMedia = vi.fn().mockReturnValue({ matches: reduced });
}

const buttonName = { name: 'Voltar ao topo' };

describe('BackToTop', () => {
  beforeEach(() => {
    defineWindow('innerHeight', 600);
    defineWindow('scrollY', 0);
    window.scrollTo = vi.fn();
    mockMotion(false);
  });

  afterEach(() => {
    cleanup();
    document.body.innerHTML = '';
  });

  it('não aparece no topo da página', () => {
    render(<BackToTop />);
    expect(screen.queryByRole('button', buttonName)).toBeNull();
  });

  it('não aparece antes de rolar uma altura de tela', () => {
    render(<BackToTop />);
    scrollTo(600);
    expect(screen.queryByRole('button', buttonName)).toBeNull();
  });

  it('aparece depois de rolar uma altura de tela', () => {
    render(<BackToTop />);
    scrollTo(900);
    expect(screen.getByRole('button', buttonName)).toBeTruthy();
  });

  it('some de novo ao voltar para o topo', () => {
    render(<BackToTop />);
    scrollTo(900);
    scrollTo(0);
    expect(screen.queryByRole('button', buttonName)).toBeNull();
  });

  it('leva ao topo e põe o foco no conteúdo', () => {
    const main = document.createElement('main');
    main.id = 'conteudo';
    main.tabIndex = -1;
    document.body.appendChild(main);
    const focus = vi.spyOn(main, 'focus');
    render(<BackToTop />);
    scrollTo(900);
    fireEvent.click(screen.getByRole('button', buttonName));
    expect(window.scrollTo).toHaveBeenCalledWith(expect.objectContaining({ top: 0 }));
    expect(focus).toHaveBeenCalledWith({ preventScroll: true });
  });

  it('funciona pelo teclado', () => {
    render(<BackToTop />);
    scrollTo(900);
    const button = screen.getByRole('button', buttonName);
    button.focus();
    expect(document.activeElement).toBe(button);
    expect(button.tagName).toBe('BUTTON');
    expect(button.getAttribute('type')).toBe('button');
  });

  it('respeita o movimento reduzido', () => {
    mockMotion(true);
    render(<BackToTop />);
    scrollTo(900);
    fireEvent.click(screen.getByRole('button', buttonName));
    expect(window.scrollTo).toHaveBeenCalledWith(expect.objectContaining({ behavior: 'auto' }));
  });
});
