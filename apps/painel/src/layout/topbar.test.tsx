import { cleanup, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { Topbar } from './topbar';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('Topbar', () => {
  it('mostra Ao vivo sem o sino de notificações', () => {
    render(<Topbar route={{ page: 'dashboard' }} connected drawer={false} open={false} onToggle={() => {}} />);
    expect(screen.getByRole('status').textContent).toContain('Ao vivo');
    expect(screen.queryByRole('button', { name: /^Notificações/ })).toBeNull();
  });
});
