import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { Board } from './board';
import { TasksPage } from './tasks-page';

const task = (id: string, status: string, over: object = {}) => ({ id, title: `Título ${id}`, status, wave: '1', profile: 'padrao', persona: 'dev', model: '', depends_on: '', covers: '', type: 'tarefa', description: '', ...over });
const meta = (id: string, status = 'planned') => ({ id, title: `Spec ${id}`, status, type: 'feature', author: 'ana', created_at: '2026-01-02', updated_at: '2026-01-03', refined_at: '', approved_at: '2026-01-04', adjustments: '' });
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
function mockFetch(handler: (url: string, init?: RequestInit) => Response | Promise<Response>) {
  const fn = vi.fn((url: string, init?: RequestInit) => Promise.resolve(handler(url, init)));
  vi.stubGlobal('fetch', fn);
  return fn;
}
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
  globalThis.IntersectionObserver = class { observe() {} disconnect() {} unobserve() {} takeRecords() { return []; } } as never;
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('Board', () => {
  it('groups the cards by status column and flags waiting dependencies', () => {
    const tasks = [
      { ...task('TASK-01', 'done'), spec: 'SPEC-001' },
      { ...task('TASK-02', 'approved', { depends_on: '[TASK-01]' }), spec: 'SPEC-001' },
      { ...task('TASK-03', 'approved', { depends_on: '[TASK-09]' }), spec: 'SPEC-001' },
    ];
    render(<Board tasks={tasks} />);
    expect(screen.getByText('Pronta para executar')).toBeTruthy();
    expect(screen.getByText('Espera TASK-09')).toBeTruthy();
    expect(screen.getByRole('link', { name: /TASK-01/ }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
  });
});

describe('TasksPage', () => {
  const overview = { project: 'p', archived: [], activity: [], questions: [], specs: [
    { meta: meta('SPEC-001'), plan: false, tasks: [task('TASK-01', 'pending')] },
    { meta: meta('SPEC-002'), plan: false, tasks: [task('TASK-07', 'pending')] },
  ] };

  it('filters the board by spec', async () => {
    mockFetch(() => json(overview));
    render(<TasksPage version={0} />);
    expect(await screen.findByText('Título TASK-07')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /SPEC-001/ }));
    expect(screen.queryByText('Título TASK-07')).toBeNull();
    expect(screen.getByText('Título TASK-01')).toBeTruthy();
  });

  it('shows the error state', async () => {
    mockFetch(() => new Response('x', { status: 500 }));
    render(<TasksPage version={0} />);
    expect(await screen.findByRole('button', { name: 'Tentar de novo' })).toBeTruthy();
  });
});
