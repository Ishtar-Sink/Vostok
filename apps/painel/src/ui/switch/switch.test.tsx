import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Switch } from './switch';

afterEach(cleanup);

const LABEL = 'Copiar as evidências para o projeto';

describe('Switch', () => {
  it('expõe role switch com aria-checked', () => {
    const { unmount } = render(<Switch label={LABEL} checked={false} onChange={() => undefined} />);
    expect(screen.getByRole('switch', { name: LABEL }).getAttribute('aria-checked')).toBe('false');
    unmount();
    render(<Switch label={LABEL} checked onChange={() => undefined} />);
    expect(screen.getByRole('switch', { name: LABEL }).getAttribute('aria-checked')).toBe('true');
  });

  it('alterna no clique chamando onChange com o oposto', () => {
    const onChange = vi.fn();
    const { unmount } = render(<Switch label={LABEL} checked={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenLastCalledWith(true);
    unmount();
    render(<Switch label={LABEL} checked onChange={onChange} />);
    fireEvent.click(screen.getByRole('switch'));
    expect(onChange).toHaveBeenLastCalledWith(false);
  });

  it('liga a ajuda por aria-describedby', () => {
    render(
      <Switch
        label={LABEL}
        help="Desligado, a pasta de destino fica vazia."
        checked
        onChange={() => undefined}
      />,
    );
    const id = screen.getByRole('switch').getAttribute('aria-describedby') ?? '';
    expect(document.getElementById(id)?.textContent).toBe(
      'Desligado, a pasta de destino fica vazia.',
    );
  });

  it('é um botão que não envia formulário', () => {
    render(<Switch label={LABEL} checked onChange={() => undefined} />);
    const control = screen.getByRole('switch');
    expect(control.tagName).toBe('BUTTON');
    expect(control.getAttribute('type')).toBe('button');
  });

  it('passa className e atributos para a raiz', () => {
    render(
      <Switch
        label={LABEL}
        checked
        onChange={() => undefined}
        className="extra"
        data-testid="interruptor"
      />,
    );
    const root = screen.getByTestId('interruptor');
    expect(root.classList.contains('extra')).toBe(true);
    expect(root.contains(screen.getByRole('switch'))).toBe(true);
  });
});
