import { cleanup, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Topbar } from './topbar';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('Topbar', () => {
  it('o sino vem antes de Ao vivo', () => {
    vi.stubGlobal('fetch', vi.fn(() => new Promise<Response>(() => {})));
    render(<Topbar route={{ page: 'dashboard' }} connected unread={2} version={0} drawer={false} open={false} onToggle={() => {}} />);
    const bell = screen.getByRole('button', { name: 'Notificações: 2 não lidas' });
    const live = screen.getByRole('status');
    expect(live.textContent).toContain('Ao vivo');
    expect(bell.compareDocumentPosition(live) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });
});
