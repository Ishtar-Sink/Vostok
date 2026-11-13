import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Checkbox } from './checkbox';

afterEach(cleanup);

describe('Checkbox', () => {
  it('liga o rótulo à caixa', () => {
    const onChange = vi.fn();
    render(<Checkbox label="Diff" checked={false} onChange={onChange} />);
    expect(screen.getByRole('checkbox', { name: 'Diff' })).toBeTruthy();
    fireEvent.click(screen.getByText('Diff'));
    expect(onChange).toHaveBeenCalled();
  });

  it('chama onChange com o novo estado no clique', () => {
    const onChange = vi.fn();
    const { unmount } = render(<Checkbox label="Diff" checked={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole('checkbox'));
    expect(onChange).toHaveBeenLastCalledWith(true);
    unmount();
    render(<Checkbox label="Diff" checked onChange={onChange} />);
    fireEvent.click(screen.getByRole('checkbox'));
    expect(onChange).toHaveBeenLastCalledWith(false);
  });

  it('liga a ajuda por aria-describedby', () => {
    render(
      <Checkbox
        label="Diff"
        help="Pede conferência das evidências de diff."
        checked={false}
        onChange={() => undefined}
      />,
    );
    const id = screen.getByRole('checkbox').getAttribute('aria-describedby');
    expect(document.getElementById(id as string)?.textContent).toBe(
      'Pede conferência das evidências de diff.',
    );
  });

  it('não chama onChange desabilitada', () => {
    const onChange = vi.fn();
    render(<Checkbox label="Diff" checked={false} disabled onChange={onChange} />);
    const box = screen.getByRole('checkbox') as HTMLInputElement;
    expect(box.disabled).toBe(true);
    fireEvent.click(box);
    expect(onChange).not.toHaveBeenCalled();
  });

  it('passa className e atributos para a raiz', () => {
    render(
      <Checkbox
        label="Diff"
        checked={false}
        onChange={() => undefined}
        className="extra"
        data-testid="caixa"
      />,
    );
    const root = screen.getByTestId('caixa');
    expect(root.classList.contains('extra')).toBe(true);
    expect(root.contains(screen.getByRole('checkbox'))).toBe(true);
  });
});
