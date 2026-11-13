import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import styles from './confirm-popover.module.css';
import { ConfirmPopover, type ConfirmPopoverProps } from './confirm-popover';

afterEach(cleanup);

const parentKeyDown = vi.fn();
const onConfirm = vi.fn();
const onCancel = vi.fn();

function Harness(props: Partial<ConfirmPopoverProps>) {
  return (
    <div onKeyDown={parentKeyDown}>
      <ConfirmPopover
        open
        title="Recusar a evidência"
        description="O motivo é obrigatório: é o que o agente vai corrigir."
        fieldLabel="Motivo da recusa"
        placeholder="O que está errado ou falta"
        confirmLabel="Recusar"
        onConfirm={onConfirm}
        onCancel={onCancel}
        {...props}
      />
    </div>
  );
}

function field() {
  return screen.getByRole('textbox', { name: 'Motivo da recusa' });
}

afterEach(() => {
  parentKeyDown.mockClear();
  onConfirm.mockClear();
  onCancel.mockClear();
});

describe('ConfirmPopover', () => {
  it('não renderiza fechado', () => {
    render(<Harness open={false} />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('abre com o título, a explicação e o foco no campo', () => {
    render(<Harness />);
    const dialog = screen.getByRole('dialog', { name: 'Recusar a evidência' });
    const describedBy = dialog.getAttribute('aria-describedby') ?? '';
    expect(document.getElementById(describedBy)?.textContent).toBe(
      'O motivo é obrigatório: é o que o agente vai corrigir.',
    );
    expect(document.activeElement).toBe(field());
  });

  it('confirma com o texto sem espaços nas pontas', () => {
    render(<Harness />);
    fireEvent.input(field(), { target: { value: '  falta o total  ' } });
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    expect(onConfirm).toHaveBeenCalledWith('falta o total');
  });

  it('aceita confirmar vazio quando o texto é opcional', () => {
    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    expect(onConfirm).toHaveBeenCalledWith('');
  });

  it('recusa confirmar vazio quando o texto é obrigatório', () => {
    render(<Harness required />);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    expect(onConfirm).not.toHaveBeenCalled();
    expect(field().getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByRole('alert').textContent).toBe('Obrigatório');
    expect(document.activeElement).toBe(field());
    fireEvent.input(field(), { target: { value: 'x' } });
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('cancela pelo botão e pelo Esc sem fechar o que está atrás', () => {
    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
    fireEvent.keyDown(field(), { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(2);
    expect(parentKeyDown).not.toHaveBeenCalled();
  });

  it('mostra o erro e o botão ocupado', () => {
    render(<Harness error="O arquivo mudou." loading />);
    expect(screen.getByRole('alert').textContent).toBe('O arquivo mudou.');
    expect(screen.getByRole('button', { name: 'Recusar' }).getAttribute('aria-busy')).toBe('true');
  });

  it('pinta o confirmar no tom de recusa', () => {
    render(<Harness tone="danger" />);
    expect(screen.getByRole('button', { name: 'Recusar' }).classList.contains(styles.danger as string)).toBe(true);
  });

  it('limpa o campo ao abrir de novo', () => {
    const { rerender } = render(<Harness />);
    fireEvent.input(field(), { target: { value: 'algo' } });
    rerender(<Harness open={false} />);
    rerender(<Harness open />);
    expect((field() as HTMLTextAreaElement).value).toBe('');
  });

  it('passa className e atributos para a raiz', () => {
    render(<Harness className="extra" data-testid="caixa" />);
    const root = screen.getByTestId('caixa');
    expect(root.className).toContain('extra');
    expect(root.getAttribute('role')).toBe('dialog');
  });
});
