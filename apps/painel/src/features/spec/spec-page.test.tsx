import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { SpecPage } from './spec-page';

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

const detail = (path: string, status = 'planned') => ({
  meta: meta('SPEC-001', status),
  path,
  plan: true,
  markdown: '# Título\n\n## Objetivo\nTexto.\n\n## Escopo\nMais texto.\n',
  tasks: [task('TASK-01', 'done'), task('TASK-02', 'pending')],
});
const route = (path: string, body: unknown) => (url: string) => (url.startsWith(path) ? json(body) : new Response('', { status: 404 }));

describe('SpecPage', () => {
  it('shows the index, the metadata and the tasks', async () => {
    mockFetch(route('/api/specs/', detail('vostok/work/specs/approved/SPEC-001.md')));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    expect(await screen.findByRole('navigation', { name: 'Nesta página' })).toBeTruthy();
    expect(screen.getByText('1 de 2 concluídas')).toBeTruthy();
    expect(screen.getByRole('link', { name: /TASK-02/ }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-02');
  });

  it('opens the edit dialog with the content of /api/fonte', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/fonte') ? json({ conteudo: 'conteúdo da fonte', hash: 'abc' }) : json(detail('vostok/work/specs/approved/SPEC-001.md'))));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Editar o arquivo' }));
    expect(await screen.findByDisplayValue('conteúdo da fonte')).toBeTruthy();
    expect(fetchMock).toHaveBeenCalledWith('/api/fonte?caminho=vostok%2Fwork%2Fspecs%2Fapproved%2FSPEC-001.md');
  });

  it('does not offer to edit an archived spec', async () => {
    mockFetch(route('/api/specs/', detail('vostok/work/.archive/specs/SPEC-001.md', 'archived')));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    await screen.findByRole('navigation', { name: 'Nesta página' });
    expect(screen.queryByRole('button', { name: 'Editar o arquivo' })).toBeNull();
  });

  it('shows the board view grouped by status', async () => {
    mockFetch(route('/api/specs/', detail('vostok/work/specs/approved/SPEC-001.md')));
    render(<SpecPage id="SPEC-001" view="quadro" version={0} />);
    expect(await screen.findByText('Título TASK-02')).toBeTruthy();
  });

  it('shows the plan view and its empty state when there is no plan', async () => {
    mockFetch((url) => (url.startsWith('/api/planos') ? json({ id: 'SPEC-001', path: 'p.md', markdown: '## Plano\nx' }) : json(detail('vostok/work/specs/approved/SPEC-001.md'))));
    render(<SpecPage id="SPEC-001" view="plano" version={0} />);
    expect(await screen.findByText('Revisão')).toBeTruthy();
  });

  it('shows the plan error state', async () => {
    mockFetch((url) => (url.startsWith('/api/planos') ? new Response('', { status: 404 }) : json(detail('vostok/work/specs/approved/SPEC-001.md'))));
    render(<SpecPage id="SPEC-001" view="plano" version={0} />);
    expect(await screen.findByText('Nenhum plano aberto')).toBeTruthy();
  });

  it('shows the error state when the spec cannot be read', async () => {
    mockFetch(() => new Response('', { status: 404 }));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    expect(await screen.findByText('Não foi possível abrir SPEC-001')).toBeTruthy();
  });
  const requests = (body: unknown) => (url: string) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-8' }) : url.startsWith('/api/specs/') ? json(body) : new Response('', { status: 404 }));
  const lastButton = (name: string) => screen.getAllByRole('button', { name }).at(-1) as HTMLButtonElement;

  it('pede o refinamento da spec em rascunho', async () => {
    const body = { ...detail('vostok/work/specs/draft/SPEC-001.md', 'draft'), plan: false, tasks: [] };
    const fetchMock = mockFetch(requests(body));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Refinar' }));
    fireEvent.click(lastButton('Refinar'));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=refinar-spec&spec=SPEC-001', expect.objectContaining({ method: 'POST' })));
  });

  it('pede a execução da próxima onda', async () => {
    const tasks = [task('TASK-01', 'done'), task('TASK-02', 'approved', { wave: '2', depends_on: '[TASK-01]' })];
    const body = { ...detail('vostok/work/specs/refined/SPEC-001.md'), plan: false, tasks };
    const fetchMock = mockFetch(requests(body));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Executar onda' }));
    expect(screen.getByText('Onda 2: TASK-02')).toBeTruthy();
    fireEvent.click(lastButton('Executar onda'));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=executar-onda&spec=SPEC-001&onda=2', expect.objectContaining({ method: 'POST' })));
  });

  it('não oferece refinar fora do rascunho', async () => {
    const body = { ...detail('vostok/work/specs/refined/SPEC-001.md'), plan: false };
    mockFetch(requests(body));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    await screen.findByRole('navigation', { name: 'Nesta página' });
    expect(screen.queryByRole('button', { name: 'Refinar' })).toBeNull();
  });

  const outdatedBody = { ...detail('vostok/work/specs/approved/SPEC-001.md', 'approved'), plan: true, plan_outdated: true, tasks: [] };

  it('mostra o aviso de plano desatualizado na spec', async () => {
    mockFetch(requests(outdatedBody));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    expect(await screen.findByText('Plano desatualizado')).toBeTruthy();
    expect(screen.getByText('Desatualizado: a spec mudou depois do plano.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Refazer o plano' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Aprovar plano' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Ajustar plano' })).toBeNull();
  });

  it('mostra o aviso de plano desatualizado na visão do plano', async () => {
    mockFetch((url) => (url.startsWith('/api/planos') ? json({ id: 'SPEC-001', path: 'p.md', markdown: '## Plano\nx' }) : json(outdatedBody)));
    render(<SpecPage id="SPEC-001" view="plano" version={0} />);
    expect(await screen.findByText(/A spec mudou depois deste plano/)).toBeTruthy();
    expect(screen.queryByText('Revisão')).toBeNull();
  });

  it('pede refazer o plano desatualizado', async () => {
    const fetchMock = mockFetch(requests(outdatedBody));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Refazer o plano' }));
    fireEvent.click(lastButton('Refazer o plano'));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=planejar-spec&spec=SPEC-001', expect.objectContaining({ method: 'POST' })));
  });

  it('pede o ajuste do plano em dia com texto', async () => {
    const body = { ...detail('vostok/work/specs/approved/SPEC-001.md', 'approved'), plan: true, plan_outdated: false, tasks: [] };
    const fetchMock = mockFetch(requests(body));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    await screen.findByRole('navigation', { name: 'Nesta página' });
    expect(screen.queryByText('Plano desatualizado')).toBeNull();
    fireEvent.click(screen.getByRole('button', { name: 'Ajustar plano' }));
    expect(lastButton('Ajustar plano').disabled).toBe(true);
    fireEvent.input(screen.getByRole('textbox', { name: 'O que ajustar' }), { target: { value: 'Dividir a TASK-02' } });
    fireEvent.click(lastButton('Ajustar plano'));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=ajustar-plano&spec=SPEC-001', expect.objectContaining({ method: 'POST', body: 'Dividir a TASK-02' })));
  });

  it('cita o log de avais da spec ao aprovar', async () => {
    const body = { ...detail('vostok/work/specs/refined/SPEC-001.md', 'refined'), plan: false, tasks: [] };
    mockFetch((url) => (url.startsWith('/api/aprovacoes') ? json({ aprovadas: ['SPEC-001'] }) : requests(body)(url)));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar spec' }));
    fireEvent.click(lastButton('Aprovar spec'));
    expect((await screen.findByRole('status')).textContent).toContain('vostok/work/avais/SPEC-001.log');
  });
});
