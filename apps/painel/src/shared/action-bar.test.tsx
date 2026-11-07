import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { ActionBar, ActionDialog, CommitList, LiveVersion, parseRefusal, performAction } from './action-bar';
import { specActions, taskActions, type AttentionItem } from './attention';
import type { PanelRequest, TaskSummary } from './api';
import dialogStyles from '../ui/dialog/dialog.module.css';

const task = (id: string, status: string, wave = '1'): TaskSummary => ({ id, title: `Título ${id}`, status, wave, profile: 'padrao', persona: '', model: '', depends_on: '', covers: '', type: 'tarefa', description: '', last_outcome: null });
const meta = { id: 'SPEC-001', title: 'Spec', status: 'approved' };
const ids = ['TASK-01', 'TASK-02', 'TASK-03', 'TASK-04', 'TASK-05'];
const batch: AttentionItem = { key: 'approve-tasks', kind: 'approve-task', spec: 'SPEC-001', tasks: ids, title: '5 tarefa(s) pendente(s)', reason: '', href: '/specs/SPEC-001' };
const specBody = { meta: { ...meta, status: 'planned' }, path: 'p.md', plan: false, plan_outdated: false, markdown: '', tasks: ids.map((id, index) => task(id, 'pending', index < 3 ? '1' : '2')), last_outcome: null };
const refusal = 'as tarefas de SPEC-001 não podem ser aprovadas; nenhuma foi alterada:\nTASK-01: Ler sem a spec\nTASK-01: Baseline vazio\nTASK-02: Evidência sem Prova';
const bar = () => render(<ActionBar items={specActions(meta, true, [task('TASK-01', 'pending'), task('TASK-02', 'pending')])} />);

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

describe('ActionBar', () => {
  it('mostra só o primário em destaque e o resto em Mais ações', async () => {
    bar();
    expect(screen.getAllByRole('button').map((el) => el.textContent)).toEqual(['Aprovar plano', 'Mais ações']);
    const more = screen.getByRole('button', { name: 'Mais ações' });
    expect(more.getAttribute('aria-haspopup')).toBe('menu');
    expect(more.getAttribute('aria-expanded')).toBe('false');
    fireEvent.click(more);
    expect(screen.getByRole('button', { name: 'Mais ações' }).getAttribute('aria-expanded')).toBe('true');
    expect(screen.getAllByRole('menuitem').map((el) => el.textContent)).toEqual(['Ajustar o plano', 'Ajustar a spec', 'Aprovar 2 tarefas']);
  });

  it('não mostra Mais ações com uma ação só', () => {
    render(<ActionBar items={taskActions('SPEC-001', { id: 'TASK-01', title: 'T', status: 'approved' }, false)} />);
    expect(screen.getByRole('button', { name: 'Ajustar a tarefa' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Mais ações' })).toBeNull();
  });

  it('navega o menu pelo teclado e fecha com Esc', async () => {
    bar();
    fireEvent.click(screen.getByRole('button', { name: 'Mais ações' }));
    await waitFor(() => expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[0]));
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[1]);
    fireEvent.keyDown(document.activeElement!, { key: 'End' });
    expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[2]);
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[0]);
    fireEvent.keyDown(document.activeElement!, { key: 'ArrowUp' });
    expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[2]);
    fireEvent.keyDown(document.activeElement!, { key: 'Home' });
    expect(document.activeElement).toBe(screen.getAllByRole('menuitem')[0]);
    fireEvent.keyDown(document.activeElement!, { key: 'Escape' });
    expect(screen.queryByRole('menu')).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Mais ações' }));
  });

  it('fecha o menu com um clique fora', () => {
    bar();
    fireEvent.click(screen.getByRole('button', { name: 'Mais ações' }));
    fireEvent.mouseDown(document.body);
    expect(screen.queryByRole('menu')).toBeNull();
  });

  it('abre o diálogo da ação escolhida no menu', () => {
    bar();
    fireEvent.click(screen.getByRole('button', { name: 'Mais ações' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Ajustar a spec' }));
    expect(screen.queryByRole('menu')).toBeNull();
    expect(screen.getByText('Ajustar a spec: SPEC-001')).toBeTruthy();
    expect(screen.getByRole('textbox', { name: 'O que ajustar' })).toBeTruthy();
  });
});

describe('aprovação em lote', () => {
  it('lista as tarefas pendentes marcadas com título e onda', async () => {
    mockFetch((url) => (url.startsWith('/api/specs/') ? json(specBody) : new Response('', { status: 404 })));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    expect(await screen.findByRole('checkbox', { name: 'TASK-01 · Título TASK-01 · Onda 1' })).toBeTruthy();
    expect(screen.getByRole('checkbox', { name: 'TASK-05 · Título TASK-05 · Onda 2' })).toBeTruthy();
    expect(screen.getAllByRole('checkbox').every((box) => (box as HTMLInputElement).checked)).toBe(true);
    expect(screen.getByText('Aprovar 5 tarefas: SPEC-001')).toBeTruthy();
  });

  it('aprova só as marcadas', async () => {
    const fetchMock = mockFetch((url) => (url.startsWith('/api/aprovacoes') ? json({ aprovadas: ['TASK-01', 'TASK-03', 'TASK-05'] }) : json(specBody)));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    fireEvent.click(await screen.findByRole('checkbox', { name: 'TASK-02 · Título TASK-02 · Onda 1' }));
    fireEvent.click(screen.getByRole('checkbox', { name: 'TASK-04 · Título TASK-04 · Onda 2' }));
    fireEvent.click(screen.getByRole('button', { name: 'Aprovar 3 tarefas' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/aprovacoes?spec=SPEC-001&tarefas=TASK-01%2CTASK-03%2CTASK-05', expect.objectContaining({ method: 'POST' })));
  });

  it('desabilita a aprovação sem tarefa marcada', async () => {
    mockFetch((url) => (url.startsWith('/api/specs/') ? json(specBody) : new Response('', { status: 404 })));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    const boxes = await screen.findAllByRole('checkbox');
    boxes.forEach((box) => fireEvent.click(box));
    expect((screen.getAllByRole('button', { name: 'Aprovar tarefa' }).at(-1) as HTMLButtonElement).disabled).toBe(true);
    expect(screen.getByText('Marque ao menos uma tarefa para aprovar.')).toBeTruthy();
  });

  it('mostra só os IDs quando a spec não carrega', async () => {
    mockFetch(() => new Response('', { status: 404 }));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    const box = await screen.findByRole('checkbox', { name: 'TASK-01' });
    expect((box as HTMLInputElement).checked).toBe(true);
  });
});

describe('recusa legível', () => {
  it('resume a recusa e agrupa os detalhes por tarefa', async () => {
    mockFetch((url) => (url.startsWith('/api/aprovacoes') ? new Response(refusal, { status: 422 }) : json(specBody)));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar 5 tarefas' }));
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('as tarefas de SPEC-001 não podem ser aprovadas; nenhuma foi alterada.');
    expect(alert.querySelector('details')).toBeNull();
    expect(alert.textContent).toContain('TASK-01');
    expect(alert.textContent).toContain('Ler sem a spec');
    expect(alert.textContent).toContain('Baseline vazio');
    expect(alert.textContent).toContain('TASK-02');
    expect(alert.textContent).toContain('Evidência sem Prova');
  });

  it('mostra a recusa de uma linha sem detalhes', () => {
    expect(parseRefusal('recusada')).toEqual({ summary: 'recusada', groups: [], rest: [] });
  });

  it('junta as linhas sem tarefa em outros detalhes', () => {
    expect(parseRefusal('falhou:\n- linha a\nTASK-03: x')).toEqual({ summary: 'falhou.', groups: [{ task: 'TASK-03', lines: ['x'] }], rest: ['linha a'] });
  });
});

const two: AttentionItem = { ...batch, tasks: ['TASK-01', 'TASK-02'], title: '2 tarefa(s) pendente(s)' };
const manual = (id: string) => `${id}: edição manual sem registro; registre-a com: vostok task adjust record-manual SPEC-001 ${id}`;
const manualRefusal = `as tarefas de SPEC-001 não podem ser aprovadas; nenhuma foi alterada:\n${manual('TASK-01')}\n${manual('TASK-02')}`;
const request = (tarefa: string, acao = 'ajustar-tarefa'): PanelRequest => ({ id: 'PED-9', acao, spec: 'SPEC-001', tarefa, onda: '', estado: 'pendente', atualizado_em: '' });

interface World {
  refusal?: string;
  requests?: PanelRequest[];
  body?: () => string;
  pedido?: (url: string) => Response;
  spec?: () => unknown;
}
function world(w: World = {}) {
  return mockFetch((url) => {
    if (url.startsWith('/api/aprovacoes')) return new Response(w.refusal ?? manualRefusal, { status: 422 });
    if (url.startsWith('/api/pedidos')) return w.pedido ? w.pedido(url) : json({ id: 'PED-1' });
    if (url.startsWith('/api/visao')) return json({ requests: w.requests ?? [] });
    if (url.startsWith('/api/tarefas/')) return new Response(w.body ? w.body() : 'corpo', { status: 200 });
    if (url.startsWith('/api/specs/')) return json(w.spec ? w.spec() : specBody);
    return new Response('', { status: 404 });
  });
}
const refuse = async () => {
  fireEvent.click(await screen.findByRole('button', { name: 'Aprovar 2 tarefas' }));
  return screen.findByRole('alert');
};
const posts = (fn: ReturnType<typeof vi.fn>, prefix: string) => fn.mock.calls.filter(([url, init]) => String(url).startsWith(prefix) && (init as RequestInit | undefined)?.method === 'POST');

describe('recusa aberta', () => {
  it('mostra todos os problemas abertos em linguagem simples', async () => {
    world();
    render(<ActionDialog item={two} onClose={() => {}} />);
    const alert = await refuse();
    expect(alert.querySelector('details')).toBeNull();
    expect(alert.textContent).toContain('TASK-01');
    expect(alert.textContent).toContain('TASK-02');
    expect(alert.textContent).toContain('Esta tarefa foi editada, e a edição ainda não foi registrada.');
    expect(alert.textContent).not.toContain('vostok');
    expect(alert.textContent).not.toContain('record-manual');
  });

  it('traduz a recusa de status', async () => {
    world({ refusal: "TASK-01: status 'approved'; só tarefas com status 'pending' são aprovadas" });
    render(<ActionDialog item={two} onClose={() => {}} />);
    expect((await refuse()).textContent).toContain('Esta tarefa não está mais à espera de aprovação: agora está aprovada.');
  });

  it('traduz a recusa de status da spec', async () => {
    world({ refusal: "a spec SPEC-001 está com status 'draft'; só specs com status 'refined' são aprovadas" });
    render(<ActionDialog item={two} onClose={() => {}} />);
    expect((await refuse()).textContent).toContain('A spec SPEC-001 está rascunho; só uma spec refinada pode ser aprovada.');
  });

  it('mostra o motivo desconhecido como veio', async () => {
    world({ refusal: 'as tarefas de SPEC-001 não podem ser aprovadas:\nTASK-01: Baseline vazio' });
    render(<ActionDialog item={two} onClose={() => {}} />);
    const alert = await refuse();
    expect(alert.textContent).toContain('Baseline vazio');
    expect(alert.querySelector('details')).toBeNull();
  });
});

describe('pedido de registro', () => {
  const register = () => screen.getByRole('button', { name: 'Pedir o registro da edição' }) as HTMLButtonElement;

  it('grava um pedido de registro por item recusado', async () => {
    const fn = world();
    render(<ActionDialog item={two} onClose={() => {}} />);
    await refuse();
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(posts(fn, '/api/pedidos')).toHaveLength(2));
    const calls = posts(fn, '/api/pedidos');
    expect(calls[0][0]).toBe('/api/pedidos?acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-01');
    expect(calls[1][0]).toBe('/api/pedidos?acao=ajustar-tarefa&spec=SPEC-001&tarefa=TASK-02');
    expect((calls[0][1] as RequestInit).body).toBe('Só registrar a edição manual, sem outras mudanças. Não aprove.');
  });

  it('fecha a janela quando todos os pedidos são aceitos', async () => {
    world();
    const onClose = vi.fn();
    render(<ActionDialog item={two} onClose={onClose} />);
    await refuse();
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('fica aberta quando um pedido é recusado', async () => {
    let refuseSecond = true;
    const fn = world({ pedido: (url) => (url.includes('TASK-02') && refuseSecond ? new Response('pedido já existe', { status: 409 }) : json({ id: 'PED-1' })) });
    const onClose = vi.fn();
    render(<ActionDialog item={two} onClose={onClose} />);
    await refuse();
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(screen.getByRole('alert').textContent).toContain('pedido já existe'));
    expect(onClose).not.toHaveBeenCalled();
    refuseSecond = false;
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    const last = posts(fn, '/api/pedidos').slice(2);
    expect(last).toHaveLength(1);
    expect(String(last[0][0])).toContain('tarefa=TASK-02');
  });

  it('pede o registro só da tarefa com edição sem registro', async () => {
    const fn = world({ refusal: `as tarefas de SPEC-001 não podem ser aprovadas; nenhuma foi alterada:\n${manual('TASK-02')}\nTASK-01: Baseline vazio` });
    render(<ActionDialog item={two} onClose={() => {}} />);
    await refuse();
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(posts(fn, '/api/pedidos')).toHaveLength(1));
    expect(String(posts(fn, '/api/pedidos')[0][0])).toContain('tarefa=TASK-02');
  });

  it('pede o registro da spec', async () => {
    const fn = world({ refusal: 'a spec SPEC-001 não pode ser aprovada:\nedição manual sem registro; registre-a com: vostok spec adjust record-manual SPEC-001' });
    const item: AttentionItem = { key: 'approve-spec', kind: 'approve-spec', spec: 'SPEC-001', title: 'Spec', reason: '', href: '/specs/SPEC-001' };
    render(<ActionDialog item={item} onClose={() => {}} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar spec' }));
    await screen.findByRole('alert');
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(posts(fn, '/api/pedidos')).toHaveLength(1));
    expect(posts(fn, '/api/pedidos')[0][0]).toBe('/api/pedidos?acao=ajustar-spec&spec=SPEC-001');
  });

  it('sem botão de registro em outro motivo', async () => {
    world({ refusal: 'as tarefas de SPEC-001 não podem ser aprovadas:\nTASK-01: Baseline vazio' });
    render(<ActionDialog item={two} onClose={() => {}} />);
    await refuse();
    expect(screen.queryByRole('button', { name: 'Pedir o registro da edição' })).toBeNull();
  });

  it('botão indisponível com pedido ativo da tarefa', async () => {
    world({ requests: [request('TASK-01')], refusal: `as tarefas de SPEC-001 não podem ser aprovadas:\n${manual('TASK-01')}` });
    render(<ActionDialog item={{ ...two, tasks: ['TASK-01'] }} onClose={() => {}} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar tarefa' }));
    await screen.findByRole('alert');
    await screen.findByRole('button', { name: 'Pedir o registro da edição' });
    await waitFor(() => expect(register().disabled).toBe(true));
  });

  it('não aprova ao pedir o registro', async () => {
    const fn = world();
    render(<ActionDialog item={two} onClose={() => {}} />);
    await refuse();
    const before = posts(fn, '/api/aprovacoes').length;
    await waitFor(() => expect(register().disabled).toBe(false));
    fireEvent.click(register());
    await waitFor(() => expect(posts(fn, '/api/pedidos')).toHaveLength(2));
    expect(posts(fn, '/api/aprovacoes')).toHaveLength(before);
  });
});

describe('janela ao vivo', () => {
  const withVersion = (version: number) => (
    <LiveVersion.Provider value={version}>
      <ActionDialog item={two} onClose={() => {}} />
    </LiveVersion.Provider>
  );
  const specWith = (first: string) => () => ({ ...specBody, tasks: ids.map((id) => task(id, id === 'TASK-01' ? first : 'pending')) });

  it('atualiza a lista e tira a recusa quando a tarefa muda', async () => {
    let text = 'antes';
    const fn = world({ body: () => text });
    const view = render(withVersion(0));
    await refuse();
    await waitFor(() => expect(fn.mock.calls.some(([url]) => String(url) === '/api/tarefas/SPEC-001/TASK-01')).toBe(true));
    text = 'depois';
    view.rerender(withVersion(1));
    await waitFor(() => expect(screen.queryByRole('alert')).toBeNull());
  });

  it('mantém a recusa quando a tarefa não mudou', async () => {
    const fn = world();
    const view = render(withVersion(0));
    await refuse();
    await waitFor(() => expect(fn.mock.calls.some(([url]) => String(url) === '/api/tarefas/SPEC-001/TASK-01')).toBe(true));
    const before = fn.mock.calls.filter(([url]) => String(url).startsWith('/api/tarefas/')).length;
    view.rerender(withVersion(1));
    await waitFor(() => expect(fn.mock.calls.filter(([url]) => String(url).startsWith('/api/tarefas/')).length).toBeGreaterThan(before));
    expect(screen.queryByRole('alert')).toBeTruthy();
  });

  it('tira da lista a tarefa aprovada pela CLI', async () => {
    let first = 'pending';
    world({ spec: () => specWith(first)() });
    const view = render(<LiveVersion.Provider value={0}><ActionDialog item={{ ...batch }} onClose={() => {}} /></LiveVersion.Provider>);
    await screen.findByRole('checkbox', { name: /TASK-01/ });
    first = 'approved';
    view.rerender(<LiveVersion.Provider value={1}><ActionDialog item={{ ...batch }} onClose={() => {}} /></LiveVersion.Provider>);
    await waitFor(() => expect(screen.queryByRole('checkbox', { name: /TASK-01/ })).toBeNull());
    expect(screen.getByText('Aprovar 4 tarefas: SPEC-001')).toBeTruthy();
  });

  it('fecha quando todas as tarefas da janela saem', async () => {
    let status = 'pending';
    world({ spec: () => ({ ...specBody, tasks: ids.map((id) => task(id, status)) }) });
    const onClose = vi.fn();
    const ui = (version: number) => <LiveVersion.Provider value={version}><ActionDialog item={batch} onClose={onClose} /></LiveVersion.Provider>;
    const view = render(ui(0));
    await screen.findByRole('checkbox', { name: /TASK-01/ });
    status = 'approved';
    view.rerender(ui(1));
    await waitFor(() => expect(onClose).toHaveBeenCalled());
  });
});

describe('fechamento da janela', () => {
  it('fecha sem mensagem quando a aprovação é aceita', async () => {
    mockFetch((url) => (url.startsWith('/api/aprovacoes') ? json({ aprovadas: ['TASK-01', 'TASK-02'] }) : json({ requests: [], ...specBody })));
    bar();
    fireEvent.click(screen.getByRole('button', { name: 'Mais ações' }));
    fireEvent.click(screen.getByRole('menuitem', { name: 'Aprovar 2 tarefas' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Aprovar 2 tarefas' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('fecha sem mensagem quando o pedido é aceito', async () => {
    mockFetch((url) => (url.startsWith('/api/pedidos') ? json({ id: 'PED-5' }) : json({})));
    const onClose = vi.fn();
    const item: AttentionItem = { key: 'k', kind: 'execute-task', spec: 'SPEC-001', task: 'TASK-01', title: 'T', reason: '', href: '/' };
    render(<ActionDialog item={item} onClose={onClose} />);
    fireEvent.click(screen.getAllByRole('button').at(-1)!);
    await waitFor(() => expect(onClose).toHaveBeenCalled());
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('fica aberta, mostrando a recusa, quando a ação é recusada', async () => {
    mockFetch(() => new Response('recusado pelo painel', { status: 409 }));
    const onClose = vi.fn();
    const item: AttentionItem = { key: 'k', kind: 'execute-task', spec: 'SPEC-001', task: 'TASK-01', title: 'T', reason: '', href: '/' };
    render(<ActionDialog item={item} onClose={onClose} />);
    fireEvent.click(screen.getAllByRole('button').at(-1)!);
    expect((await screen.findByRole('alert')).textContent).toContain('recusado pelo painel');
    expect(onClose).not.toHaveBeenCalled();
  });

  it('fecha quando a pendência sai da tela', async () => {
    mockFetch(() => json({ requests: [], ...specBody }));
    const items = specActions(meta, true, [task('TASK-01', 'pending'), task('TASK-02', 'pending')]);
    const view = render(<ActionBar items={items} />);
    fireEvent.click(screen.getByRole('button', { name: 'Aprovar plano' }));
    expect(screen.getByRole('dialog')).toBeTruthy();
    view.rerender(<ActionBar items={[]} />);
    expect(screen.queryByRole('dialog')).toBeNull();
    view.rerender(<ActionBar items={items} />);
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});

describe('aprovação em lote: onda vazia', () => {
  it('esconde a onda vazia', async () => {
    mockFetch(() => json({ ...specBody, tasks: ids.map((id) => task(id, 'pending', '')) }));
    render(<ActionDialog item={batch} onClose={() => {}} />);
    expect(await screen.findByRole('checkbox', { name: 'TASK-01 · Título TASK-01' })).toBeTruthy();
    expect(screen.queryByText(/Onda/)).toBeNull();
  });
});

describe('mensagens de resultado', () => {
  it('mensagens sem comando nem caminho', async () => {
    mockFetch((url) => (url.startsWith('/api/aprovacoes') ? json({ aprovadas: ['TASK-01'] }) : json({ id: 'PED-3' })));
    const approved = await performAction({ ...batch, tasks: ['TASK-01'] }, '');
    const requested = await performAction({ key: 'k', kind: 'execute-task', spec: 'SPEC-001', task: 'TASK-01', title: 'T', reason: '', href: '/' }, '');
    expect(requested.message).toContain('PED-3');
    for (const { message } of [approved, requested]) {
      expect(message).not.toMatch(/`|vostok |work\/|skill/);
    }
  });
});

describe('lista de commits', () => {
  const lines = (count: number) => ['Commits novos:', ...Array.from({ length: count }, (_, index) => `- abc${1000 + index} Commit ${index} (arquivos: a.txt)`)];
  const releaseItem = (count: number): AttentionItem => ({ key: 'release', kind: 'release-commits', spec: '', title: lines(count).join('\n'), reason: '', href: '/pendencias' });

  it('deixa aberta a lista com 10 commits', () => {
    const { container } = render(<CommitList lines={lines(10)} />);
    expect(container.querySelector('details')).toBeNull();
    expect(screen.getAllByRole('listitem')).toHaveLength(11);
  });

  it('recolhe a lista com 11 commits, mostrando a contagem', () => {
    const { container } = render(<CommitList lines={lines(11)} />);
    const details = container.querySelector('details')!;
    expect(details.hasAttribute('open')).toBe(false);
    expect(container.querySelector('summary')!.textContent).toBe('11 commits apontados');
  });

  it('não conta o título como commit', () => {
    const withRemoved = [...lines(10), 'Commits removidos (estavam liberados e saíram do histórico):'];
    const { container } = render(<CommitList lines={withRemoved} />);
    expect(container.querySelector('details')).toBeNull();
  });

  it('expande a lista pelo teclado', () => {
    const { container } = render(<CommitList lines={lines(11)} />);
    const summary = container.querySelector('summary')!;
    summary.focus();
    expect(document.activeElement).toBe(summary);
    container.querySelector('details')!.setAttribute('open', '');
    expect(screen.getByText('- abc1010 Commit 10 (arquivos: a.txt)')).toBeTruthy();
  });

  it('recolhe a lista na janela de liberação', () => {
    render(<ActionDialog item={releaseItem(11)} onClose={() => {}} />);
    expect(screen.getByText('11 commits apontados')).toBeTruthy();
  });

  it('não muda o que o pedido grava', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-9' }));
    render(<ActionDialog item={releaseItem(11)} onClose={() => {}} />);
    fireEvent.click(screen.getAllByRole('button', { name: 'Liberar commits' }).at(-1)!);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=liberar-commits', expect.objectContaining({ method: 'POST', body: lines(11).join('\n') })));
  });
});

const REAL = [
  '## Pergunta', '', '## O que aconteceu', '', 'A exportação parou na cópia dos arquivos.', '',
  '## Efeito de cada opção', '', '| Opção | Efeito |', '|---|---|', '| A | O arquivo vai como está. |', '',
  '# 1. Como tratar a linha apontada', '', 'Escolha: única', '', 'A linha é a versão do Java.', '',
  '| Opção | Descrição |', '|---|---|', '| A | Aceitar o arquivo (recomendada) |', '| B | Mascarar o número |', '',
  'Responda com o número e a letra (ex.: 1A, 2B). Caso nenhuma se encaixe, descreva como deseja prosseguir.', '',
  '## Inferências', '', '- A cópia fica na pasta de documentos.',
].join('\n');
const answerItem: AttentionItem = {
  key: 'answer', kind: 'answer', spec: 'SPEC-001', title: 'Como tratar a linha apontada', reason: '', href: '/pendencias',
  question: { meta: { id: 'P-001', spec: 'SPEC-001', task: '', status: 'aberta', created_at: '', author: '' }, path: 'p.md', markdown: REAL },
};

describe('janela de ação', () => {
  const open = () => {
    render(<ActionBar items={[answerItem]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Responder' }));
  };

  it('mostra o formulário de seleção na pergunta do template', () => {
    open();
    expect(screen.getByRole('group', { name: '1. Como tratar a linha apontada' })).toBeTruthy();
    expect(screen.getByRole('radio', { name: 'Outro' })).toBeTruthy();
    expect(screen.getByText('A exportação parou na cópia dos arquivos.')).toBeTruthy();
    expect(screen.queryByText('Ou escreva a resposta')).toBeNull();
    expect(within(screen.getByRole('dialog')).queryByRole('button', { name: 'Responder' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Enviar resposta' })).toBeTruthy();
  });

  it('envia a resposta do formulário como pedido e fecha a janela', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-9' }));
    open();
    fireEvent.click(screen.getByRole('radio', { name: /^A / }));
    fireEvent.click(screen.getByRole('button', { name: 'Enviar resposta' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=responder-pergunta&spec=SPEC-001&pergunta=P-001', expect.objectContaining({ method: 'POST', body: '1A' })));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
  });

  it('mantém a janela aberta na recusa da resposta', async () => {
    mockFetch(() => new Response('recusado pelo painel', { status: 409 }));
    open();
    const radio = screen.getByRole('radio', { name: /^A / }) as HTMLInputElement;
    fireEvent.click(radio);
    fireEvent.click(screen.getByRole('button', { name: 'Enviar resposta' }));
    expect((await screen.findByRole('alert')).textContent).toContain('recusado pelo painel');
    expect(screen.getByRole('dialog')).toBeTruthy();
    expect(screen.getByRole('group', { name: /^1\. / })).toBeTruthy();
    expect((screen.getByRole('radio', { name: /^A / }) as HTMLInputElement).checked).toBe(true);
  });

  it('mostra o texto inteiro e o campo livre fora do template', () => {
    const free = { ...answerItem, question: { ...answerItem.question!, markdown: '## Pergunta\n\n## O que aconteceu\n\nO verificador falhou na cópia.\n\nQual caminho seguir?\n' } };
    render(<ActionBar items={[free]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Responder' }));
    expect(screen.getByText(/O verificador falhou na cópia/)).toBeTruthy();
    expect(screen.getByRole('textbox', { name: 'Ou escreva a resposta' })).toBeTruthy();
    expect(screen.queryByRole('radio')).toBeNull();
  });

  it('abre na largura do conteúdo', () => {
    open();
    expect(screen.getByRole('dialog').className).toContain(dialogStyles.content);
  });
});
