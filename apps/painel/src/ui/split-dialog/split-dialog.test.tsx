import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { useState } from 'react';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { SplitDialog, type SplitDialogProps } from './split-dialog';

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute('open', '');
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute('open');
  };
});
afterEach(cleanup);

function Harness(props: Partial<SplitDialogProps>) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <button type="button" onClick={() => setOpen(true)}>
        Abrir
      </button>
      <SplitDialog
        title="Testes do quadro"
        eyebrow="EV-01"
        status={<span>A conferir</span>}
        side={<button type="button">Item da explicação</button>}
        main={<pre>linha do log</pre>}
        footer={<button type="button">Aceitar</button>}
        {...props}
        open={open}
        onClose={() => {
          props.onClose?.();
          setOpen(false);
        }}
      />
    </>
  );
}

function openDialog() {
  fireEvent.click(screen.getByRole('button', { name: 'Abrir' }));
}

describe('SplitDialog', () => {
  it('não renderiza nada fechado', () => {
    render(<Harness />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('abre como modal com o título como nome acessível', () => {
    render(<Harness />);
    openDialog();
    const dialog = screen.getByRole('dialog', { name: 'Testes do quadro' });
    expect(dialog.hasAttribute('open')).toBe(true);
    expect(within(dialog).getByRole('heading', { level: 2 }).textContent).toBe('Testes do quadro');
  });

  it('mostra o selo antes do título e o estado depois', () => {
    render(<Harness />);
    openDialog();
    const head = screen.getByRole('heading', { level: 2 }).parentElement as HTMLElement;
    const text = head.textContent ?? '';
    expect(text.indexOf('EV-01')).toBeLessThan(text.indexOf('Testes do quadro'));
    expect(text.indexOf('Testes do quadro')).toBeLessThan(text.indexOf('A conferir'));
  });

  it('mostra a explicação, a evidência e o rodapé', () => {
    render(<Harness />);
    openDialog();
    expect(within(screen.getByRole('region', { name: 'Explicação' })).getByText('Item da explicação')).toBeTruthy();
    expect(within(screen.getByRole('region', { name: 'Evidência' })).getByText('linha do log')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Aceitar' })).toBeTruthy();
  });

  it('fecha pelo botão Fechar (Esc) e pelo Esc', () => {
    const onClose = vi.fn();
    render(<Harness onClose={onClose} />);
    openDialog();
    fireEvent.click(screen.getByRole('button', { name: 'Fechar (Esc)' }));
    expect(onClose).toHaveBeenCalledTimes(1);
    openDialog();
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('não fecha no Esc já tratado por um filho', () => {
    const onClose = vi.fn();
    render(
      <Harness
        onClose={onClose}
        side={
          <textarea
            aria-label="campo"
            onKeyDown={(event) => {
              if (event.key === 'Escape') event.preventDefault();
            }}
          />
        }
      />,
    );
    openDialog();
    fireEvent.keyDown(screen.getByLabelText('campo'), { key: 'Escape' });
    expect(onClose).not.toHaveBeenCalled();
  });

  it('prende o foco entre o primeiro e o último elemento', () => {
    render(<Harness />);
    openDialog();
    const dialog = screen.getByRole('dialog');
    const buttons = within(dialog).getAllByRole('button');
    const first = buttons[0] as HTMLElement;
    const last = buttons[buttons.length - 1] as HTMLElement;
    last.focus();
    fireEvent.keyDown(last, { key: 'Tab' });
    expect(document.activeElement).toBe(first);
    first.focus();
    fireEvent.keyDown(first, { key: 'Tab', shiftKey: true });
    expect(document.activeElement).toBe(last);
  });

  it('devolve o foco ao gatilho ao fechar', () => {
    render(<Harness />);
    const trigger = screen.getByRole('button', { name: 'Abrir' });
    trigger.focus();
    openDialog();
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(document.activeElement).toBe(trigger);
  });

  it('troca o painel pelas abas Explicação e Evidência', () => {
    const onPaneChange = vi.fn();
    render(<Harness onPaneChange={onPaneChange} />);
    openDialog();
    const tabs = screen.getByRole('tablist', { name: 'Partes da conferência' });
    expect(within(tabs).getByRole('tab', { name: 'Explicação' }).getAttribute('aria-selected')).toBe('true');
    fireEvent.click(within(tabs).getByRole('tab', { name: 'Evidência' }));
    expect(screen.getByRole('region', { name: 'Explicação' }).parentElement?.getAttribute('data-pane')).toBe('main');
    expect(onPaneChange).toHaveBeenCalledWith('main');
  });

  it('segue o painel controlado pela prop pane', () => {
    render(<Harness pane="main" />);
    openDialog();
    expect(screen.getByRole('tab', { name: 'Evidência' }).getAttribute('aria-selected')).toBe('true');
  });

  it('passa className e atributos para o dialog', () => {
    render(<Harness className="extra" data-testid="modal" />);
    openDialog();
    const dialog = screen.getByTestId('modal');
    expect(dialog.tagName).toBe('DIALOG');
    expect(dialog.className).toContain('extra');
  });
});
