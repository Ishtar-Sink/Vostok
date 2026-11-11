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

  it('leva a migalha da tarefa ao quadro filtrado pela spec', () => {
    render(<Topbar route={{ page: 'task', spec: 'SPEC-001', task: 'TASK-02' }} connected drawer={false} open={false} onToggle={() => {}} />);
    expect(screen.getByRole('link', { name: 'SPEC-001' }).getAttribute('href')).toBe('/tarefas?spec=SPEC-001');
    expect(screen.getByRole('link', { name: 'Tarefas' }).getAttribute('href')).toBe('/tarefas');
  });
});
