import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { InboxPage } from './inbox-page';

const task = (id: string, status: string) => ({ id, title: `Título ${id}`, status, wave: '1', profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '' });
const meta = { id: 'SPEC-001', title: 'Spec de teste', status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' };
const question = { meta: { id: 'Q-1', spec: 'SPEC-002', task: '', status: 'open', created_at: '', author: '' }, path: 'q.md', markdown: '## Pergunta\nQual caminho seguir?\n\n## Opções\n- A\n- B\n' };
const HINT = 'Responda com o número e a letra (ex.: 1A, 2B). Caso nenhuma se encaixe, descreva como deseja prosseguir.';
const TEMPLATE = [
  '## Pergunta', '', '# 1. Stack do app', '', 'Escolha: única', '',
  'O repositório só tem o harness, e nada define a linguagem.', '',
  '| Opção | Descrição |', '|---|---|',
  '| A | Web com TypeScript, React e Vite (recomendada) |',
  '| B | Web com HTML e JavaScript puros |',
  '| C | Outra stack; informe a linguagem |', '', HINT, '',
  '# 2. A lista de tarefas com filtros ainda não existe', '', 'Escolha: várias', '',
  'A spec assume que a lista e os filtros já existem.', '',
  '| Opção | Descrição |', '|---|---|',
  '| A | Criar antes uma spec para a lista (recomendada) |',
  '| B | Ajustar a SPEC-001 para incluir uma lista mínima |',
  '| C | Aprovar agora, registrando o desvio |', '', HINT,
].join('\n');
const templateQuestion = { meta: { id: 'P-002', spec: 'SPEC-001', task: '', status: 'aberta', created_at: '', author: '' }, path: 'p.md', markdown: TEMPLATE };
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
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
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

  const unread = [
    { id: 'PED-001', spec: '', texto: 'Qual o formato do arquivo exportado?\nDetalhe' },
    { id: 'PED-002', spec: 'SPEC-001', texto: 'Pode apagar a pasta de testes?' },
  ];

  it('mostra um item para cada resposta não lida', async () => {
    mockFetch(() => json(overview({ respostas_nao_lidas: unread })));
    render(<InboxPage version={0} />);
    expect(await screen.findByText('Respostas às suas perguntas')).toBeTruthy();
    expect(screen.getAllByRole('button', { name: /Abrir a resposta de/ })).toHaveLength(2);
    expect(screen.getByText('Sem spec')).toBeTruthy();
    expect(screen.getByText('Qual o formato do arquivo exportado?')).toBeTruthy();
  });

  it('resposta não lida leva à pergunta', async () => {
    mockFetch(() => json(overview({ respostas_nao_lidas: unread })));
    render(<InboxPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Abrir a resposta de PED-001' }));
    expect(window.location.pathname + window.location.hash).toBe('/perguntas#PED-001');
  });

  it('resposta não lida tira o estado vazio', async () => {
    mockFetch(() => json(overview({ specs: [], questions: [], respostas_nao_lidas: unread })));
    render(<InboxPage version={0} />);
    expect(await screen.findByText('Respostas às suas perguntas')).toBeTruthy();
    expect(screen.queryByText('Nada esperando você')).toBeNull();
  });

  it('sem resposta não lida, nada muda', async () => {
    mockFetch(() => json(overview()));
    render(<InboxPage version={0} />);
    expect(await screen.findByRole('button', { name: 'Aprovar tarefa' })).toBeTruthy();
    expect(screen.queryByText('Respostas às suas perguntas')).toBeNull();
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
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
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

    const manyLines = (count: number) => ['Commits novos:', ...Array.from({ length: count }, (_, index) => `- abc${1000 + index} Commit ${index} (arquivos: a.txt)`)];

    it('deixa aberta nas Pendências a lista com 10 commits', async () => {
      mockFetch(withCommits(manyLines(10)));
      render(<InboxPage version={0} />);
      const region = await screen.findByRole('region', { name: 'Commits sem liberação' });
      expect(region.querySelector('details')).toBeNull();
      expect(within(region).getByText('- abc1009 Commit 9 (arquivos: a.txt)')).toBeTruthy();
    });

    it('recolhe nas Pendências a lista com 11 commits', async () => {
      mockFetch(withCommits(manyLines(11)));
      render(<InboxPage version={0} />);
      const region = await screen.findByRole('region', { name: 'Commits sem liberação' });
      expect(within(region).getByText('11 commits apontados')).toBeTruthy();
      expect(region.querySelector('details')!.hasAttribute('open')).toBe(false);
    });

    it('libera os 11 commits com a lista inteira', async () => {
      const fn = mockFetch(withCommits(manyLines(11)));
      render(<InboxPage version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Liberar commits' }));
      fireEvent.click(screen.getAllByRole('button', { name: 'Liberar commits' }).at(-1)!);
      await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=liberar-commits', expect.objectContaining({ method: 'POST', body: manyLines(11).join('\n') })));
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
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=responder-pergunta&spec=SPEC-002&pergunta=Q-1', expect.objectContaining({ method: 'POST', body: 'A' })));
    unmount();
    render(<InboxPage version={0} />);
    await reply();
    expect((await screen.findByRole('alert')).textContent).toContain('a pergunta já foi respondida');
  });

  it('mostra a fila guiada da spec', async () => {
    mockFetch(() => json(overview({ specs: [{ meta: { ...meta, status: 'approved' }, plan: true, plan_outdated: false, tasks: [task('TASK-01', 'pending'), task('TASK-02', 'pending')] }], questions: [templateQuestion] })));
    render(<InboxPage version={0} />);
    const region = await screen.findByRole('region', { name: /SPEC-001/ });
    const items = Array.from(region.querySelectorAll(':scope > ul > li')) as HTMLElement[];
    expect(items).toHaveLength(3);
    expect(items[0].textContent).toContain('Faça agora');
    for (const item of items.slice(1)) {
      expect(item.textContent).toContain('Depois de:');
      expect(within(item).queryAllByRole('button')).toHaveLength(0);
    }
    for (const item of items) expect(['decisao', 'resposta', 'conferencia', 'teste-manual']).toContain(item.dataset.effort);
  });

  it('responde a pergunta do template pelo formulário', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-2' }) : json(overview({ specs: [], questions: [templateQuestion] }))));
    render(<InboxPage version={0} />);
    const g1 = within(await screen.findByRole('group', { name: '1. Stack do app' }));
    fireEvent.click(g1.getByRole('radio', { name: /^A / }));
    fireEvent.click(within(screen.getByRole('group', { name: /^2\. / })).getByRole('checkbox', { name: /^B / }));
    fireEvent.click(screen.getByRole('button', { name: 'Enviar resposta' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=responder-pergunta&spec=SPEC-001&pergunta=P-002', expect.objectContaining({ method: 'POST', body: '1A, 2B' })));
    expect((await screen.findByRole('status')).textContent).toContain('PED-2');
  });

  it('mostra o título da primeira pergunta do template', async () => {
    mockFetch(() => json(overview({ specs: [], questions: [templateQuestion] })));
    render(<InboxPage version={0} />);
    expect(await screen.findByText('Stack do app e mais 1')).toBeTruthy();
    expect(screen.queryByText('Escolha: única')).toBeNull();
  });

  it('responde em texto livre a pergunta fora do template', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-3' }) : json(overview({ specs: [], questions: [{ ...question, markdown: '## Pergunta\nQual caminho seguir?\n' }] }))));
    render(<InboxPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Responder' }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Ou escreva a resposta' }), { target: { value: 'Seguir pelo B' } });
    fireEvent.click(screen.getAllByRole('button', { name: 'Responder' }).at(-1)!);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith(expect.stringContaining('responder-pergunta'), expect.objectContaining({ method: 'POST', body: 'Seguir pelo B' })));
    expect(screen.queryByRole('group', { name: /^1\. / })).toBeNull();
  });

  it('mostra o texto inicial da pergunta real', async () => {
    const markdown = ['## Pergunta', '', '## O que aconteceu', '', 'A exportação parou na cópia.', '', TEMPLATE.replace('## Pergunta\n\n', '')].join('\n');
    mockFetch(() => json(overview({ specs: [], questions: [{ ...templateQuestion, markdown }] })));
    render(<InboxPage version={0} />);
    expect(await screen.findByRole('group', { name: '1. Stack do app' })).toBeTruthy();
    expect(screen.getByText('A exportação parou na cópia.')).toBeTruthy();
    expect(screen.getByText('Stack do app e mais 1')).toBeTruthy();
  });

  it('Abrir as evidências leva à seção da tarefa em validação', async () => {
    mockFetch(() => json(overview({ questions: [], specs: [{ meta, plan: false, tasks: [task('TASK-01', 'validating'), task('TASK-02', 'blocked')] }] })));
    render(<InboxPage version={0} />);
    const link = await screen.findByRole('link', { name: 'Abrir as evidências' });
    expect(screen.getAllByRole('link', { name: 'Abrir as evidências' })).toHaveLength(1);
    fireEvent.click(link);
    expect(window.location.pathname + window.location.hash).toBe('/tarefas/SPEC-001/TASK-01#evidencias');
  });
});
