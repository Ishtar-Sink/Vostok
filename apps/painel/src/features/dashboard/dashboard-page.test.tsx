import { cleanup, render, screen, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DashboardPage } from './dashboard-page';

const task = (id: string, status: string, wave: string) => ({ id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '' });
const meta = { id: 'SPEC-001', title: 'Spec em andamento', status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' };
const overview = (over: object = {}) => ({
  project: 'proj',
  specs: [{ meta, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'in_progress', '2'), task('TASK-03', 'pending', '2')] }],
  archived: [{ meta: { ...meta, id: 'SPEC-000' }, tasks_total: 2 }],
  ...over,
});
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('DashboardPage', () => {
  it('shows the indicators and the specs in progress with waves', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    render(<DashboardPage version={0} name="Meu projeto" />);
    expect(await screen.findByText('Meu projeto')).toBeTruthy();
    expect(screen.getByText('Specs abertas')).toBeTruthy();
    expect(screen.getByText('1 arquivadas')).toBeTruthy();
    expect(screen.getByText('Spec em andamento')).toBeTruthy();
    expect(screen.getByRole('img', { name: '1 pendente, 1 em execução, 1 concluída' })).toBeTruthy();
    expect(screen.getByText('Onda 2')).toBeTruthy();
  });

  it('conta a tarefa executed como em execução', async () => {
    const specs = [{ meta, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'executed', '2'), task('TASK-03', 'pending', '2')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    await screen.findByText('Spec em andamento');
    expect(within(screen.getByText('Tarefas em execução').parentElement as HTMLElement).getByText('1')).toBeTruthy();
    expect(screen.getByRole('img', { name: '1 pendente, 1 pronta para finalizar, 1 concluída' })).toBeTruthy();
  });

  it('não mostra onda sem número', async () => {
    const specs = [{ meta, plan: false, tasks: [task('TASK-01', 'pending', ''), task('TASK-02', 'in_progress', '')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    await screen.findByText('Spec em andamento');
    expect(screen.queryByRole('list', { name: 'Ondas' })).toBeNull();
    expect(screen.queryByText(/^Onda/)).toBeNull();
  });

  it('mostra só as ondas com número', async () => {
    const specs = [{ meta, plan: false, tasks: [task('TASK-01', 'pending', '2'), task('TASK-02', 'pending', '')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Onda 2')).toBeTruthy();
    expect(within(screen.getByRole('list', { name: 'Ondas' })).getAllByRole('listitem')).toHaveLength(1);
  });

  it('shows the empty messages when there is nothing yet', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs: [], archived: [] })))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Nenhuma spec com plano ou tarefas.')).toBeTruthy();
    expect(screen.getByText('Nenhuma tarefa faltando')).toBeTruthy();
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

  it('mostra as tarefas que faltam com o status e a onda', async () => {
    const specs = [{ meta, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'in_progress', '1'), task('TASK-03', 'approved', '2')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    const section = await screen.findByRole('region', { name: 'Tarefas que faltam' });
    const second = within(section).getByRole('link', { name: /TASK-02/ }).closest('li') as HTMLElement;
    expect(within(second).getByText('em execução')).toBeTruthy();
    expect(within(second).getByText('onda 1')).toBeTruthy();
    const third = within(section).getByRole('link', { name: /TASK-03/ }).closest('li') as HTMLElement;
    expect(within(third).getByText('aprovada')).toBeTruthy();
    expect(within(third).getByText('onda 2')).toBeTruthy();
  });

  it('não mostra tarefa concluída em Tarefas que faltam', async () => {
    const specs = [{ meta, plan: false, plan_outdated: false, tasks: [task('TASK-01', 'done', '1'), task('TASK-02', 'in_progress', '1')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    const section = await screen.findByRole('region', { name: 'Tarefas que faltam' });
    expect(within(section).queryByText(/TASK-01/)).toBeNull();
    expect(within(section).getByRole('link', { name: /TASK-02/ })).toBeTruthy();
  });

  it('mostra o plano desatualizado no cartão da spec', async () => {
    const specs = [{ meta, plan: true, plan_outdated: true, tasks: [task('TASK-01', 'pending', '1')] }];
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview({ specs })))));
    render(<DashboardPage version={0} name="" />);
    expect(await screen.findByText('Plano desatualizado')).toBeTruthy();
  });

  it('não mostra botão de ação nem Aguardando você', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json(overview()))));
    const { container } = render(<DashboardPage version={0} name="" />);
    await screen.findByText('Spec em andamento');
    expect(container.querySelectorAll('button')).toHaveLength(0);
    expect(screen.queryByText('Aguardando você')).toBeNull();
  });
});
