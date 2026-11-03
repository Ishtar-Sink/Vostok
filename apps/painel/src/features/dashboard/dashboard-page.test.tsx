import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { fetchActivities, type Activity } from '../../shared/api';
import { DashboardPage } from './dashboard-page';

vi.mock('../../shared/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../shared/api')>()),
  fetchActivities: vi.fn(),
}));

const task = (id: string, status: string, wave: string) => ({ id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '' });
const meta = { id: 'SPEC-001', title: 'Spec em andamento', status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' };
const overview = (over: object = {}) => ({
  project: 'proj',
  specs: [{ meta, plan: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'in_progress', '2'), task('TASK-03', 'pending', '2')] }],
  archived: [{ meta: { ...meta, id: 'SPEC-000' }, tasks_total: 2 }],
  questions: [],
  ...over,
});
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

const activityAt = (index: number): Activity => {
  const task = `TASK-${String(index + 1).padStart(2, '0')}`;
  return { date: '2026-11-02T10:00:00Z', type: 'tarefa', spec: 'SPEC-001', task, target: task, step: 'finish', result: 'done', detail: `detalhe ${index + 1}` };
};
const ALL_ACTIVITIES = Array.from({ length: 15 }, (_, index) => activityAt(index));

beforeEach(() => {
  vi.mocked(fetchActivities).mockImplementation(async ({ limite = 5 } = {}) => ALL_ACTIVITIES.slice(0, limite));
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
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
    expect(await screen.findByText(/detalhe 1$/)).toBeTruthy();
  });

  it('lists what waits for the user', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByRole('button', { name: 'Aprovar tarefa' })).toBeTruthy();
  });

  it('shows the empty messages when there is nothing yet', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs: [], archived: [] })))));
    vi.mocked(fetchActivities).mockResolvedValue([]);
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Nenhuma spec com plano ou tarefas.')).toBeTruthy();
    expect(await screen.findByText('Sem registros ainda.')).toBeTruthy();
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

  it('mostra cinco atividades por padrão', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="" />);
    const section = await screen.findByRole('region', { name: 'Atividade recente' });
    expect(await within(section).findAllByRole('listitem')).toHaveLength(5);
    expect(fetchActivities).toHaveBeenCalledWith({ limite: 5 });
  });

  it('troca a quantidade pelo seletor', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="" />);
    const section = await screen.findByRole('region', { name: 'Atividade recente' });
    await within(section).findAllByRole('listitem');
    const group = screen.getByRole('group', { name: 'Quantidade de atividades' });
    expect(within(group).getByRole('button', { name: '5' }).getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(within(group).getByRole('button', { name: '10' }));
    expect(fetchActivities).toHaveBeenLastCalledWith({ limite: 10 });
    await waitFor(() => expect(within(section).getAllByRole('listitem')).toHaveLength(10));
    expect(within(group).getByRole('button', { name: '10' }).getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(within(group).getByRole('button', { name: '15' }));
    expect(fetchActivities).toHaveBeenLastCalledWith({ limite: 15 });
    await waitFor(() => expect(within(section).getAllByRole('listitem')).toHaveLength(15));
  });

  it('leva a todas as atividades', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="" />);
    const link = await screen.findByRole('link', { name: 'Ver todas' });
    expect(link.getAttribute('href')).toBe('/atividades');
  });
});
