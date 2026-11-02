import { cleanup, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DashboardPage } from './dashboard-page';

const task = (id: string, status: string, wave: string) => ({ id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '' });
const meta = { id: 'SPEC-001', title: 'Spec em andamento', status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' };
const overview = (over: object = {}) => ({
  project: 'proj',
  specs: [{ meta, plan: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'in_progress', '2'), task('TASK-03', 'pending', '2')] }],
  archived: [{ meta: { ...meta, id: 'SPEC-000' }, tasks_total: 2 }],
  activity: [{ date: '2026-11-02T10:00:00Z', spec: 'SPEC-001', task: 'TASK-01', step: 'finish', result: 'done', detail: 'tudo certo' }],
  questions: [],
  ...over,
});
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('DashboardPage', () => {
  it('shows the indicators, the specs in progress with waves and the recent activity', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="Meu projeto" />);
    expect(await screen.findByText('Meu projeto')).toBeTruthy();
    expect(screen.getByText('Specs abertas')).toBeTruthy();
    expect(screen.getByText('1 arquivadas')).toBeTruthy();
    expect(screen.getByText('Spec em andamento')).toBeTruthy();
    expect(screen.getByRole('img', { name: '1 pendente, 1 em execução, 1 concluída' })).toBeTruthy();
    expect(screen.getByText('Onda 2')).toBeTruthy();
    expect(screen.getByText('Atividade recente')).toBeTruthy();
    expect(screen.getByText(/tudo certo/)).toBeTruthy();
  });

  it('lists what waits for the user', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByRole('button', { name: 'Aprovar tarefa' })).toBeTruthy();
  });

  it('shows the empty messages when there is nothing yet', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs: [], archived: [], activity: [] })))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Nenhuma spec com plano ou tarefas.')).toBeTruthy();
    expect(screen.getByText('Sem registros ainda.')).toBeTruthy();
    expect(screen.getByText('Nada esperando você')).toBeTruthy();
  });

  it('shows the error state when the API fails', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(new Response('', { status: 500 }))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Não foi possível ler o harness')).toBeTruthy();
  });

  it('shows the loading state first', () => {
    vi.stubGlobal('fetch', vi.fn(() => new Promise(() => {})));
    render(<DashboardPage version={0} name="" />);
    expect(screen.getByText('Carregando o painel')).toBeTruthy();
  });
});
