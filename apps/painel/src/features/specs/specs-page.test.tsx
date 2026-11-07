import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { SpecsPage } from './specs-page';

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

const working = (acao: string, over: object = {}) => ({ id: 'PED-001', acao, spec: 'SPEC-001', tarefa: '', onda: '', estado: 'em_andamento', atualizado_em: '2026-11-03T10:00:00Z', ...over });

const overview = (over: object = {}) => ({
  project: 'p',
  specs: [{ meta: meta('SPEC-001'), plan: false, tasks: [task('TASK-01', 'done'), task('TASK-02', 'pending')] }],
  archived: [{ meta: meta('SPEC-000', 'archived'), tasks_total: 3 }],
  activity: [],
  questions: [],
  ...over,
});

describe('SpecsPage', () => {
  it('mostra o trabalho do agente no status da spec', async () => {
    mockFetch(() => json(overview({ requests: [working('aprovar-plano')] })));
    render(<SpecsPage version={0} />);
    expect(await screen.findByText('Detalhando as tarefas')).toBeTruthy();
    expect(screen.getByText('planejada')).toBeTruthy();
  });

  it('mostra a tarefa com o agente no status da spec', async () => {
    mockFetch(() => json(overview({ requests: [working('executar-tarefa', { tarefa: 'TASK-02' })] })));
    render(<SpecsPage version={0} />);
    expect(await screen.findByText('Executando a TASK-02')).toBeTruthy();
  });

  it('mostra o pedido na fila do agente', async () => {
    mockFetch(() => json(overview({ requests: [working('ajustar-spec', { estado: 'pendente' })] })));
    render(<SpecsPage version={0} />);
    expect(await screen.findByText('Aguardando o agente')).toBeTruthy();
  });

  it('não mostra trabalho do agente sem pedido', async () => {
    mockFetch(() => json(overview({ requests: [] })));
    render(<SpecsPage version={0} />);
    await screen.findByText('Spec SPEC-001');
    expect(screen.queryByText('Aguardando o agente')).toBeNull();
    expect(screen.queryByText('Detalhando as tarefas')).toBeNull();
  });

  it('lists the open specs first and filters the archived ones', async () => {
    mockFetch(() => json(overview()));
    render(<SpecsPage version={0} />);
    expect(await screen.findByText('Spec SPEC-001')).toBeTruthy();
    expect(screen.queryByText('Spec SPEC-000')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: /Arquivadas/ }));
    expect(screen.getByText('Spec SPEC-000')).toBeTruthy();
    expect(screen.queryByText('Spec SPEC-001')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: /Todas/ }));
    expect(screen.getByText('Spec SPEC-000')).toBeTruthy();
    expect(screen.getByText('Spec SPEC-001')).toBeTruthy();
  });

  it('shows the empty state when no spec matches the filter', async () => {
    mockFetch(() => json(overview({ specs: [], archived: [] })));
    render(<SpecsPage version={0} />);
    expect(await screen.findByText('Nenhuma spec aqui')).toBeTruthy();
  });

  it('shows the error state and retries', async () => {
    const fetchMock = mockFetch(() => new Response('x', { status: 500 }));
    render(<SpecsPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Tentar de novo' }));
    await waitFor(() => expect(fetchMock.mock.calls.length).toBeGreaterThan(1));
  });
  it('grava o pedido de nova spec com o assunto', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-7' }) : json(overview())));
    render(<SpecsPage version={0} />);
    await screen.findByText('Spec SPEC-001');
    fireEvent.click(screen.getByRole('button', { name: 'Nova spec' }));
    const submit = () => screen.getAllByRole('button', { name: 'Nova spec' }).at(-1) as HTMLButtonElement;
    expect(submit().disabled).toBe(true);
    fireEvent.input(screen.getByRole('textbox', { name: 'Assunto da spec' }), { target: { value: 'Tela de login' } });
    fireEvent.click(submit());
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=criar-spec', expect.objectContaining({ method: 'POST', body: 'Tela de login' })));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('mostra a recusa do pedido de nova spec', async () => {
    mockFetch((url) => (url.startsWith('/api/pedidos') ? new Response('ação desconhecida', { status: 400 }) : json(overview())));
    render(<SpecsPage version={0} />);
    await screen.findByText('Spec SPEC-001');
    fireEvent.click(screen.getByRole('button', { name: 'Nova spec' }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Assunto da spec' }), { target: { value: 'Tela de login' } });
    fireEvent.click(screen.getAllByRole('button', { name: 'Nova spec' }).at(-1) as HTMLButtonElement);
    expect((await screen.findByRole('alert')).textContent).toContain('ação desconhecida');
  });
});
