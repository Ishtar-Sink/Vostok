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

  it('mostra o botão Pedir commit', async () => {
    mockFetch(() => json(overview({ specs: [], questions: [] })));
    render(<InboxPage version={0} />);
    expect(await screen.findByRole('button', { name: 'Pedir commit' })).toBeTruthy();
  });

  it('grava o pedido de commit com o texto', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-1' }) : json(overview())));
    render(<InboxPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Pedir commit' }));
    fireEvent.input(screen.getByRole('textbox'), { target: { value: 'Commit da TASK-03' } });
    fireEvent.click(screen.getAllByRole('button', { name: 'Pedir commit' }).at(-1)!);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=fazer-commit', expect.objectContaining({ method: 'POST', body: 'Commit da TASK-03' })));
    expect((await screen.findByRole('status')).textContent).toContain('PED-1');
  });
  describe('commits sem liberação', () => {
    const LINES = ['Commits novos:', 'abc1234 feat: x'];
    const withCommits = (lines: string[], requests: (url: string) => Response = () => json({ id: 'PED-3' })) => (url: string) =>
      url.startsWith('/api/commits') ? json({ linhas: lines }) : url.startsWith('/api/pedidos') ? requests(url) : json(overview({ specs: [], questions: [] }));
    const requestCalls = (fn: ReturnType<typeof vi.fn>) => fn.mock.calls.filter(([url]) => String(url).startsWith('/api/pedidos'));

    it('mostra os commits apontados com liberar e desfazer', async () => {
      mockFetch(withCommits(LINES));
      render(<InboxPage version={0} />);
      const region = await screen.findByRole('region', { name: 'Commits sem liberação' });
      expect(region.textContent).toContain('abc1234 feat: x');
      expect(screen.getByRole('button', { name: 'Liberar commits' })).toBeTruthy();
      expect(screen.getByRole('button', { name: 'Desfazer commits' })).toBeTruthy();
    });

    it('libera os commits pelo pedido', async () => {
      const fn = mockFetch(withCommits(LINES));
      render(<InboxPage version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Liberar commits' }));
      fireEvent.click(screen.getAllByRole('button', { name: 'Liberar commits' }).at(-1)!);
      await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=liberar-commits', expect.objectContaining({ method: 'POST', body: LINES.join('\n') })));
    });

    it('desfaz os commits só depois de confirmar', async () => {
      const fn = mockFetch(withCommits(LINES));
      render(<InboxPage version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Desfazer commits' }));
      expect(screen.getByRole('alertdialog')).toBeTruthy();
      fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
      expect(screen.queryByRole('alertdialog')).toBeNull();
      expect(requestCalls(fn)).toHaveLength(0);
      fireEvent.click(screen.getByRole('button', { name: 'Desfazer commits' }));
      fireEvent.click(screen.getAllByRole('button', { name: 'Desfazer commits' }).at(-1)!);
      await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=desfazer-commits', expect.objectContaining({ method: 'POST', body: LINES.join('\n') })));
      expect((await screen.findByRole('status')).textContent).toContain('PED-3');
    });

    it('esconde os commits quando não há nenhum', async () => {
      mockFetch(withCommits([]));
      render(<InboxPage version={0} />);
      await screen.findByText('Nada esperando você');
      expect(screen.queryByRole('region', { name: 'Commits sem liberação' })).toBeNull();
    });
  });

  it('responde a pergunta com o id dela', async () => {
    let answers = 0;
    const fetchMock = mockFetch((url) => {
      if (!url.startsWith('/api/pedidos')) return json(overview());
      answers += 1;
      return answers === 1 ? json({ id: 'PED-2' }) : new Response('a pergunta já foi respondida', { status: 409 });
    });
    const reply = async () => {
      fireEvent.click(await screen.findByRole('button', { name: 'Responder' }));
      fireEvent.click(screen.getByRole('radio', { name: 'A' }));
      fireEvent.click(screen.getAllByRole('button', { name: 'Responder' }).at(-1)!);
    };
    const { unmount } = render(<InboxPage version={0} />);
    await reply();
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=responder-pergunta&spec=SPEC-001&pergunta=Q-1', expect.objectContaining({ method: 'POST', body: 'A' })));
    unmount();
    render(<InboxPage version={0} />);
    await reply();
    expect((await screen.findByRole('alert')).textContent).toContain('a pergunta já foi respondida');
  });
});
