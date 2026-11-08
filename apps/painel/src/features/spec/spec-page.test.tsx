import { cleanup, render, screen } from '@testing-library/preact';
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

  it('não oferece editar o arquivo', async () => {
    mockFetch(route('/api/specs/', detail('vostok/work/specs/refined/SPEC-001.md', 'refined')));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    await screen.findByRole('navigation', { name: 'Nesta página' });
    expect(screen.queryByRole('button', { name: /Editar/ })).toBeNull();
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

  const outdatedBody = { ...detail('vostok/work/specs/approved/SPEC-001.md', 'approved'), plan: true, plan_outdated: true, tasks: [] };

  it('mostra o aviso de plano desatualizado na spec', async () => {
    mockFetch(route('/api/specs/', outdatedBody));
    render(<SpecPage id="SPEC-001" view="documento" version={0} />);
    expect(await screen.findByText('Plano desatualizado')).toBeTruthy();
    expect(screen.getByText('Desatualizado: a spec mudou depois do plano.')).toBeTruthy();
  });

  it('mostra o aviso de plano desatualizado na visão do plano', async () => {
    mockFetch((url) => (url.startsWith('/api/planos') ? json({ id: 'SPEC-001', path: 'p.md', markdown: '## Plano\nx' }) : json(outdatedBody)));
    render(<SpecPage id="SPEC-001" view="plano" version={0} />);
    expect(await screen.findByText(/A spec mudou depois deste plano/)).toBeTruthy();
    expect(screen.queryByText('Revisão')).toBeNull();
  });

  it('não mostra botão de ação em nenhum status', async () => {
    for (const status of ['draft', 'refined', 'approved', 'planned']) {
      for (const withPlan of [true, false]) {
        const body = { ...detail(`vostok/work/specs/${status}/SPEC-001.md`, status), plan: withPlan };
        mockFetch(route('/api/specs/', body));
        const { unmount } = render(<SpecPage id="SPEC-001" view="documento" version={0} />);
        await screen.findByRole('navigation', { name: 'Nesta página' });
        for (const button of screen.queryAllByRole('button')) expect(button.textContent ?? '').not.toMatch(/Aprovar|Refinar|Planejar|Ajustar|Refazer|Executar|Finalizar|Editar/);
        unmount();
      }
    }
  });
});
