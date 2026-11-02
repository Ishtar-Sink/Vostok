import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { InboxPage } from './inbox-page';

const task = (id: string, status: string) => ({ id, title: `Título ${id}`, status, wave: '1', profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '' });
const meta = { id: 'SPEC-001', title: 'Spec de teste', status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' };
const question = { meta: { id: 'Q-1', spec: 'SPEC-001', task: '', status: 'open', created_at: '', author: '' }, path: 'q.md', markdown: '## Pergunta\nQual caminho seguir?\n\n## Opções\n- A\n- B\n' };
const overview = (over: object = {}) => ({ project: 'p', specs: [{ meta, plan: false, tasks: [task('TASK-01', 'pending')] }], archived: [], activity: [], questions: [question], ...over });

function mockFetch(handler: (url: string, init?: RequestInit) => Response | Promise<Response>) {
  const fn = vi.fn((url: string, init?: RequestInit) => Promise.resolve(handler(url, init)));
  vi.stubGlobal('fetch', fn);
  return fn;
}
const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() {
    this.setAttribute('open', '');
  };
  HTMLDialogElement.prototype.close = function close() {
    this.removeAttribute('open');
  };
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('InboxPage', () => {
  it('lists a pending task and an open question with their actions', async () => {
    mockFetch(() => json(overview()));
    render(<InboxPage version={0} />);
    expect(await screen.findByRole('button', { name: 'Aprovar tarefa' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Responder' })).toBeTruthy();
    expect(screen.getByText('Qual caminho seguir?')).toBeTruthy();
  });

  it('approves the task through POST /api/aprovacoes', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/aprovacoes') ? json({ aprovadas: ['TASK-01'] }) : json(overview())));
    render(<InboxPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar tarefa' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Aprovar tarefa' }).at(-1)!);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/aprovacoes?spec=SPEC-001&tarefas=TASK-01', expect.objectContaining({ method: 'POST' })));
    expect(await screen.findByRole('status')).toBeTruthy();
  });

  it('shows the failure message when the approval is refused', async () => {
    mockFetch((url) => (url.startsWith('/api/aprovacoes') ? new Response('recusada', { status: 409 }) : json(overview())));
    render(<InboxPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar tarefa' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Aprovar tarefa' }).at(-1)!);
    expect((await screen.findByRole('alert')).textContent).toContain('recusada');
  });

  it('shows the empty state when nothing waits for the user', async () => {
    mockFetch(() => json(overview({ specs: [], questions: [] })));
    render(<InboxPage version={0} />);
    expect(await screen.findByText('Nada esperando você')).toBeTruthy();
  });

  it('shows the error state with a retry when the API fails', async () => {
    mockFetch(() => new Response('', { status: 500 }));
    render(<InboxPage version={0} />);
    expect(await screen.findByText('Não foi possível ler o harness')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Tentar de novo' })).toBeTruthy();
  });

  it('shows the loading state first', () => {
    mockFetch(() => new Promise<Response>(() => {}) as unknown as Response);
    render(<InboxPage version={0} />);
    expect(screen.getByText('Carregando as pendências')).toBeTruthy();
  });
});
