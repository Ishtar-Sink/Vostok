import { act, cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { CopyButton } from './copy-button';

function stubClipboard(value: unknown) {
  Object.defineProperty(navigator, 'clipboard', { value, configurable: true });
}

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe('CopyButton', () => {
  it('copia o texto para a área de transferência', () => {
    const writeText = vi.fn(() => Promise.resolve());
    stubClipboard({ writeText });
    const text = '$ cmd\n# data: d\n# commit: c\n# código de saída: 0\nsaída';
    render(<CopyButton text={text} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copiar' }));
    expect(writeText).toHaveBeenCalledWith(text);
  });

  it('aceita uma função que devolve o texto na hora do clique', () => {
    const writeText = vi.fn(() => Promise.resolve());
    stubClipboard({ writeText });
    render(<CopyButton text={() => 'agora'} />);
    fireEvent.click(screen.getByRole('button', { name: 'Copiar' }));
    expect(writeText).toHaveBeenCalledWith('agora');
  });

  it('troca o ícone por um check e volta depois de 2 s', async () => {
    vi.useFakeTimers();
    stubClipboard({ writeText: vi.fn(() => Promise.resolve()) });
    render(<CopyButton text="x" />);
    const button = screen.getByRole('button', { name: 'Copiar' });
    expect(button.textContent).toBe('');
    expect(button.querySelector('[data-icon="copy"]')).toBeTruthy();
    await act(async () => {
      fireEvent.click(button);
    });
    expect(button.querySelector('[data-icon="check"]')).toBeTruthy();
    expect(button.querySelector('[data-icon="copy"]')).toBeNull();
    expect(button.textContent).toBe('');
    expect(screen.getByRole('status').textContent).toBe('Copiado');
    await act(async () => {
      vi.advanceTimersByTime(2000);
    });
    expect(button.querySelector('[data-icon="copy"]')).toBeTruthy();
    expect(screen.getByRole('status').textContent).toBe('');
  });

  it('avisa quando a cópia falha', async () => {
    stubClipboard({ writeText: vi.fn(() => Promise.reject(new Error('negado'))) });
    render(<CopyButton text="x" />);
    fireEvent.click(screen.getByRole('button', { name: 'Copiar' }));
    expect((await screen.findByRole('status')).textContent).toBe('Não foi possível copiar');
  });

  it('avisa quando o navegador não oferece a área de transferência', async () => {
    stubClipboard(undefined);
    render(<CopyButton text="x" />);
    fireEvent.click(screen.getByRole('button', { name: 'Copiar' }));
    expect((await screen.findByRole('status')).textContent).toBe('Não foi possível copiar');
  });

  it('usa o rótulo acessível recebido e funciona pelo teclado', () => {
    const writeText = vi.fn(() => Promise.resolve());
    stubClipboard({ writeText });
    render(<CopyButton text="x" label="Copiar saida.txt" />);
    const button = screen.getByRole('button', { name: 'Copiar saida.txt' });
    expect(button.getAttribute('type')).toBe('button');
    button.focus();
    expect(document.activeElement).toBe(button);
    fireEvent.click(button);
    expect(writeText).toHaveBeenCalledWith('x');
  });
});
