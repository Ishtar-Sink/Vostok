import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { Dialog } from './dialog';

beforeAll(() => {
  // O jsdom não implementa o <dialog> modal.
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute('open', '');
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute('open');
  };
});

afterEach(cleanup);

describe('Dialog', () => {
  it('shows the title, the body and the actions when open', () => {
    render(
      <Dialog open title="Aprovar tarefa" onClose={() => {}} actions={<button type="button">Confirmar</button>}>
        <p>Corpo</p>
      </Dialog>,
    );
    expect(screen.getByRole('dialog', { name: 'Aprovar tarefa' })).toBeTruthy();
    expect(screen.getByText('Corpo')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Confirmar' })).toBeTruthy();
  });

  it('renders nothing when closed', () => {
    render(
      <Dialog open={false} title="Fechado" onClose={() => {}} actions={null}>
        <p>Corpo</p>
      </Dialog>,
    );
    expect(screen.queryByText('Corpo')).toBeNull();
  });

  it('calls onClose on Escape', () => {
    const onClose = vi.fn();
    render(
      <Dialog open title="Esc" onClose={onClose} actions={null}>
        <p>Corpo</p>
      </Dialog>,
    );
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onClose).toHaveBeenCalledOnce();
  });
});
