import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { NumberField } from './number-field';

afterEach(cleanup);

const LABEL = 'Tempo-limite no baseline';

describe('NumberField', () => {
  it('liga o rótulo ao campo numérico', () => {
    render(<NumberField label={LABEL} value={1} min={0} max={3600} onChange={() => undefined} />);
    const field = screen.getByRole('spinbutton', { name: LABEL });
    expect(field.getAttribute('min')).toBe('0');
    expect(field.getAttribute('max')).toBe('3600');
  });

  it('mostra o valor e o vazio com null', () => {
    const { unmount } = render(<NumberField label={LABEL} value={120} onChange={() => undefined} />);
    expect((screen.getByRole('spinbutton') as HTMLInputElement).value).toBe('120');
    unmount();
    render(<NumberField label={LABEL} value={null} onChange={() => undefined} />);
    expect((screen.getByRole('spinbutton') as HTMLInputElement).value).toBe('');
  });

  it('mostra a unidade e a liga ao campo', () => {
    render(<NumberField label={LABEL} value={1} unit="s" onChange={() => undefined} />);
    const unit = screen.getByText('s');
    const describedBy = screen.getByRole('spinbutton').getAttribute('aria-describedby') ?? '';
    expect(describedBy.split(' ')).toContain(unit.id);
  });

  it('chama onChange com o número digitado', () => {
    const onChange = vi.fn();
    render(<NumberField label={LABEL} value={1} onChange={onChange} />);
    fireEvent.input(screen.getByRole('spinbutton'), { target: { value: '300' } });
    expect(onChange).toHaveBeenCalledWith(300);
  });

  it('aceita número fora da faixa para o erro aparecer', () => {
    const onChange = vi.fn();
    render(<NumberField label={LABEL} value={1} min={0} max={3600} onChange={onChange} />);
    fireEvent.input(screen.getByRole('spinbutton'), { target: { value: '-1' } });
    expect(onChange).toHaveBeenCalledWith(-1);
  });

  it('chama onChange com null no campo vazio', () => {
    const onChange = vi.fn();
    render(<NumberField label={LABEL} value={1} onChange={onChange} />);
    fireEvent.input(screen.getByRole('spinbutton'), { target: { value: '' } });
    expect(onChange).toHaveBeenCalledWith(null);
  });

  it('chama onChange com null em texto que não é número', () => {
    const onChange = vi.fn();
    render(<NumberField label={LABEL} value={1} onChange={onChange} />);
    fireEvent.input(screen.getByRole('spinbutton'), { target: { value: 'abc' } });
    expect(onChange).toHaveBeenCalledWith(null);
  });

  it('liga a ajuda por aria-describedby', () => {
    render(<NumberField label={LABEL} value={1} help="0 sem limite." onChange={() => undefined} />);
    const id = screen.getByRole('spinbutton').getAttribute('aria-describedby') ?? '';
    expect(document.getElementById(id)?.textContent).toBe('0 sem limite.');
  });

  it('mostra o erro com role alert e marca o campo inválido', () => {
    const error = 'Use um valor de 0 a 3600 segundos.';
    const { unmount } = render(
      <NumberField label={LABEL} value={-1} error={error} onChange={() => undefined} />,
    );
    const alert = screen.getByRole('alert');
    expect(alert.textContent).toBe(error);
    const field = screen.getByRole('spinbutton');
    expect(field.getAttribute('aria-invalid')).toBe('true');
    expect((field.getAttribute('aria-describedby') ?? '').split(' ')).toContain(alert.id);
    unmount();
    render(<NumberField label={LABEL} value={1} onChange={() => undefined} />);
    expect(screen.getByRole('spinbutton').hasAttribute('aria-invalid')).toBe(false);
  });

  it('passa className e atributos para a raiz', () => {
    render(
      <NumberField
        label={LABEL}
        value={1}
        onChange={() => undefined}
        className="extra"
        data-testid="campo"
      />,
    );
    const root = screen.getByTestId('campo');
    expect(root.classList.contains('extra')).toBe(true);
    expect(root.contains(screen.getByRole('spinbutton'))).toBe(true);
  });
});
