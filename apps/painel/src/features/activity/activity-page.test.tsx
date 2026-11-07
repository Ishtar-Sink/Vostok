import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Activity, Overview, PanelRequest, Question } from '../../shared/api';
import { ActivityPage } from './activity-page';

const activity = (overrides: Partial<Activity>): Activity => ({ date: '2026-11-03', type: 'tarefa', spec: 'SPEC-001', task: 'TASK-01', target: 'TASK-01', step: 'início', result: 'in_progress', detail: 'detalhe', ...overrides });
const ALL = [
  activity({ detail: 'primeira tarefa' }),
  activity({ type: 'pedido', spec: 'SPEC-002', task: '', target: 'PED-1', step: 'pedido', result: 'done', detail: 'um pedido' }),
];

const overview = (parts: Partial<Overview> = {}): Overview => ({ project: 'demo', specs: [], archived: [], activity: [], questions: [], requests: [], loop: null, ...parts });
const specMeta = (id: string) => ({ meta: { id }, tasks: [], plan: false, plan_outdated: false }) as unknown as Overview['specs'][number];
const question: Question = { meta: { id: 'P-001', spec: 'SPEC-001', task: '', status: 'aberta', created_at: '2026-01-05T12:00:00Z', author: 'agente' }, path: 'vostok/work/perguntas/aberta/P-001.md', markdown: '## Pergunta\n\n# 1. Separador\n\nQual separador usar?\n' };
const request = (overrides: Partial<PanelRequest>): PanelRequest => ({ id: 'PED-003', acao: 'planejar-spec', spec: 'SPEC-002', tarefa: '', onda: '', estado: 'em_andamento', atualizado_em: '2026-01-05T12:30:00Z', ...overrides });

function stubFetch({ rows = ALL, view = () => overview(), failAt }: { rows?: Activity[]; view?: () => Overview | null; failAt?: number } = {}) {
  const calls: string[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    if (url.startsWith('/api/visao')) {
      const data = view();
      return Promise.resolve(data ? new Response(JSON.stringify(data), { status: 200 }) : new Response('erro', { status: 500 }));
    }
    calls.push(url);
    const params = new URL(url, 'http://x').searchParams;
    const filtered = rows.filter((item) => (!params.get('tipo') || item.type === params.get('tipo')) && (!params.get('spec') || item.spec === params.get('spec')));
    const limit = Number(params.get('limite') ?? 15);
    const offset = Number(params.get('deslocamento') ?? 0);
    if (failAt !== undefined && calls.length === failAt) return Promise.resolve(new Response('erro', { status: 500 }));
    return Promise.resolve(new Response(JSON.stringify(filtered.slice(offset, offset + limit)), { status: 200 }));
  }));
  return calls;
}

function rowsOf(count: number): Activity[] {
  return Array.from({ length: count }, (_, index) => activity({ detail: `linha ${count - index}` }));
}

function stubObserver() {
  let created = false;
  let callback: (entries: { isIntersecting: boolean }[]) => void = () => {};
  vi.stubGlobal('IntersectionObserver', class {
    constructor(handler: typeof callback) { callback = handler; created = true; }
    observe() {}
    disconnect() {}
  });
  return {
    reach: async () => {
      await waitFor(() => expect(created).toBe(true));
      await act(() => callback([{ isIntersecting: true }]));
    },
  };
}

const lines = () => [...document.querySelectorAll('ol li')].map((item) => item.textContent ?? '');

async function choose(label: string, option: string) {
  fireEvent.click(screen.getByRole('button', { name: new RegExp(label) }));
  fireEvent.click(await screen.findByRole('option', { name: option }));
}

Element.prototype.scrollIntoView = vi.fn();
HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('ActivityPage', () => {
  it('lista todas as atividades', async () => {
    const calls = stubFetch();
    render(<ActivityPage version={0} />);
    expect(await screen.findByText('primeira tarefa')).toBeTruthy();
    expect(screen.getByText('Um pedido')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Atividades' })).toBeTruthy();
    expect(calls[0]).toContain('limite=50');
  });

  it('filtra por tipo', async () => {
    const calls = stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Tipo', 'Pedido');
    await waitFor(() => expect(screen.queryByText('primeira tarefa')).toBeNull());
    expect(screen.getByText('Um pedido')).toBeTruthy();
    expect(calls.at(-1)).toContain('tipo=pedido');
  });

  it('filtra por spec', async () => {
    const calls = stubFetch({ view: () => overview({ specs: [specMeta('SPEC-001'), specMeta('SPEC-002')] }) });
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Spec', 'SPEC-002');
    await waitFor(() => expect(screen.queryByText('primeira tarefa')).toBeNull());
    expect(calls.at(-1)).toContain('spec=SPEC-002');
  });

  it('mostra a lista vazia com o filtro visível', async () => {
    stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Tipo', 'Aval');
    expect(await screen.findByText('Nenhuma atividade com esses filtros.')).toBeTruthy();
    expect(screen.getByRole('button', { name: /Tipo/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Spec/ })).toBeTruthy();
  });

  it('agrupa as atividades por dia, da mais nova para a mais antiga', async () => {
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z', detail: 'do dia 5' }), activity({ date: '2026-01-04T12:00:00Z', detail: 'do dia 4' })] });
    render(<ActivityPage version={0} />);
    await screen.findByRole('region', { name: '05/01/2026' });
    expect(screen.getAllByRole('heading', { level: 2 }).map((item) => item.textContent)).toEqual(['05/01/2026', '04/01/2026']);
    const day = within(screen.getByRole('region', { name: '05/01/2026' }));
    expect(day.getByText('do dia 5')).toBeTruthy();
    expect(day.queryByText('do dia 4')).toBeNull();
  });

  it('mostra o tipo, o ícone e o link de cada atividade', async () => {
    stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    expect(document.querySelector('li[data-type="tarefa"]')?.textContent).toContain('Tarefa');
    expect(document.querySelector('li[data-type="pedido"]')?.textContent).toContain('Pedido');
    expect(screen.getByRole('link', { name: 'TASK-01 · Início' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
    const items = document.querySelectorAll('ol li');
    expect(items.length).toBeGreaterThan(0);
    items.forEach((li) => expect(li.querySelector('svg[aria-hidden="true"]')).not.toBeNull());
  });

  it('mostra o botão da ação na atividade mais recente do item', async () => {
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z', detail: 'mais nova' }), activity({ date: '2026-01-04T12:00:00Z', detail: 'mais antiga' })], view: () => overview({ questions: [question] }) });
    render(<ActivityPage version={0} />);
    await screen.findByRole('button', { name: 'Responder' });
    expect(screen.getAllByRole('button', { name: 'Responder' })).toHaveLength(1);
    expect(within(screen.getByText('mais nova').closest('li')!).getByRole('button', { name: 'Responder' })).toBeTruthy();
  });

  it('abre o diálogo da ação pelo botão', async () => {
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z' })], view: () => overview({ questions: [question] }) });
    render(<ActivityPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Responder' }));
    expect(await screen.findByRole('dialog')).toBeTruthy();
  });

  it('abre o formulário de seleção pelo Responder', async () => {
    const REAL = ['## Pergunta', '', '## O que aconteceu', '', 'A exportação parou na cópia.', '', '# 1. Como tratar a linha', '', 'Escolha: única', '', '| Opção | Descrição |', '|---|---|', '| A | Aceitar (recomendada) |', '| B | Mascarar |'].join('\n');
    const real: Question = { ...question, markdown: REAL };
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z' })], view: () => overview({ questions: [real] }) });
    render(<ActivityPage version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Responder' }));
    expect(await screen.findByRole('radio', { name: 'Outro' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Enviar resposta' })).toBeTruthy();
    expect(screen.queryByText('Ou escreva a resposta')).toBeNull();
  });

  it('tira o botão quando a ação deixa de ser necessária', async () => {
    let current = overview({ questions: [question] });
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z' })], view: () => current });
    const { rerender } = render(<ActivityPage version={0} />);
    await screen.findByRole('button', { name: 'Responder' });
    current = overview();
    rerender(<ActivityPage version={1} />);
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Responder' })).toBeNull());
  });

  it('mostra os pedidos em andamento no topo', async () => {
    stubFetch({ view: () => overview({ requests: [request({})] }) });
    render(<ActivityPage version={0} />);
    const top = await screen.findByRole('region', { name: 'Em andamento' });
    expect(within(top).getByText('Planejando')).toBeTruthy();
    expect(within(top).getByRole('link', { name: 'SPEC-002' }).getAttribute('href')).toBe('/specs/SPEC-002');
    await screen.findByText('primeira tarefa');
    expect(top.compareDocumentPosition(screen.getAllByRole('region').at(-1)!) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  });

  it('mostra o pedido na fila como aguardando o agente', async () => {
    stubFetch({ view: () => overview({ requests: [request({ estado: 'pendente' })] }) });
    render(<ActivityPage version={0} />);
    expect(within(await screen.findByRole('region', { name: 'Em andamento' })).getByText('Aguardando o agente')).toBeTruthy();
  });

  it('mostra a linha do tempo sem a visão geral', async () => {
    stubFetch({ view: () => null });
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    expect(screen.queryByRole('region', { name: 'Em andamento' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Responder' })).toBeNull();
  });

  it('mostra em português status, ações e tipos', async () => {
    stubFetch({ rows: [
      activity({ type: 'pedido', target: 'PED-1', task: '', step: 'estado', result: 'em_andamento', detail: 'responder-pergunta' }),
    ] });
    render(<ActivityPage version={0} />);
    expect(await screen.findByText('em andamento')).toBeTruthy();
    expect(screen.getByText('Responder pergunta')).toBeTruthy();
    expect(screen.getByText(/Mudança de estado/)).toBeTruthy();
    expect(screen.queryByText(/em_andamento|responder-pergunta/)).toBeNull();
  });

  it('mostra os tipos em português no filtro', async () => {
    stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    fireEvent.click(screen.getByRole('button', { name: /Tipo/ }));
    const names = (await screen.findAllByRole('option')).map((option) => option.textContent);
    expect(names).toEqual(['Todos', 'Spec', 'Plano', 'Tarefa', 'Pedido', 'Aval']);
  });

  it('mostra o aval sem o hash', async () => {
    const hash = 'a'.repeat(64);
    stubFetch({ rows: [activity({ type: 'aval', target: 'SPEC-001/TASK-01', step: 'aprovar', result: 'painel', detail: hash })] });
    render(<ActivityPage version={0} />);
    expect(await screen.findByText(/Aprovação/)).toBeTruthy();
    expect(screen.getByText('pelo painel')).toBeTruthy();
    expect(document.body.textContent).not.toContain(hash);
  });

  it('mostra um cartão por atividade, com o tipo', async () => {
    stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    const cards = document.querySelectorAll('ol li');
    expect(cards).toHaveLength(2);
    expect(cards[0].getAttribute('data-type')).toBe('tarefa');
    expect(cards[0].textContent).toContain('Tarefa');
    expect(cards[1].getAttribute('data-type')).toBe('pedido');
    expect(cards[1].textContent).toContain('Pedido');
  });

  it('mostra sem hífen nem sublinhado o valor sem rótulo', async () => {
    stubFetch({ rows: [activity({ step: 'etapa-nova', result: 'resultado_novo' })] });
    render(<ActivityPage version={0} />);
    expect(await screen.findByText(/Etapa nova/)).toBeTruthy();
    expect(screen.getByText('resultado novo')).toBeTruthy();
    expect(document.body.textContent).not.toMatch(/etapa-nova|resultado_novo/);
  });

  it('usa o botão menor nas ações', async () => {
    stubFetch({ rows: [activity({ date: '2026-01-05T12:00:00Z' })], view: () => overview({ questions: [question] }) });
    render(<ActivityPage version={0} />);
    expect((await screen.findByRole('button', { name: 'Responder' })).className).toMatch(/small/);
  });

  it('pede a primeira página com 50 atividades, nunca 1000', async () => {
    const calls = stubFetch({ rows: rowsOf(120) });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    expect(lines()).toHaveLength(50);
    expect(calls.some((url) => url.includes('limite=1000'))).toBe(false);
  });

  it('carrega a próxima página ao chegar ao fim da lista', async () => {
    const calls = stubFetch({ rows: rowsOf(120) });
    const observer = stubObserver();
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    await observer.reach();
    await screen.findByText('linha 21');
    expect(calls.at(-1)).toContain('limite=50&deslocamento=50');
    expect(lines()).toHaveLength(100);
    expect(new Set(lines()).size).toBe(100);
  });

  it('Carregar mais funciona pelo teclado', async () => {
    stubFetch({ rows: rowsOf(120) });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    const button = screen.getByRole('button', { name: 'Carregar mais' });
    button.focus();
    fireEvent.click(button);
    await waitFor(() => expect(lines()).toHaveLength(100));
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Carregar mais' }));
  });

  it('tira Carregar mais no fim da lista', async () => {
    stubFetch({ rows: rowsOf(120) });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(lines()).toHaveLength(100));
    expect(screen.getByRole('button', { name: 'Carregar mais' })).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(lines()).toHaveLength(120));
    expect(screen.queryByRole('button', { name: 'Carregar mais' })).toBeNull();
  });

  it('tira Carregar mais depois de uma leitura vazia com exatamente 50 linhas', async () => {
    stubFetch({ rows: rowsOf(50) });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 50');
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Carregar mais' })).toBeNull());
    expect(lines()).toHaveLength(50);
  });

  it('relê do início até a quantidade carregada, sem repetir', async () => {
    let rows = rowsOf(120);
    const calls = stubFetch({ rows });
    vi.mocked(fetch).mockImplementation(((url: string) => {
      if (url.startsWith('/api/visao')) return Promise.resolve(new Response(JSON.stringify(overview()), { status: 200 }));
      calls.push(url);
      const params = new URL(url, 'http://x').searchParams;
      const offset = Number(params.get('deslocamento') ?? 0);
      return Promise.resolve(new Response(JSON.stringify(rows.slice(offset, offset + Number(params.get('limite')))), { status: 200 }));
    }) as typeof fetch);
    const { rerender } = render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(lines()).toHaveLength(100));
    rows = [activity({ detail: 'linha nova' }), ...rows];
    rerender(<ActivityPage version={1} />);
    await screen.findByText('linha nova');
    expect(calls.at(-1)).toContain('limite=100');
    expect(calls.at(-1)).not.toContain('deslocamento');
    expect(lines()).toHaveLength(100);
    expect(new Set(lines()).size).toBe(100);
  });

  it('volta à primeira página ao trocar o filtro', async () => {
    const calls = stubFetch({ rows: rowsOf(120) });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(lines()).toHaveLength(100));
    await choose('Tipo', 'Tarefa');
    await waitFor(() => expect(calls.at(-1)).toContain('limite=50'));
    expect(calls.at(-1)).toContain('tipo=tarefa');
    expect(calls.at(-1)).not.toContain('deslocamento');
  });

  it('lista no filtro a spec fora das páginas carregadas', async () => {
    stubFetch({ view: () => overview({ specs: [specMeta('SPEC-001')], archived: [{ meta: { id: 'SPEC-009' }, tasks_total: 0 } as unknown as Overview['archived'][number]] }) });
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    fireEvent.click(screen.getByRole('button', { name: /Spec/ }));
    expect(await screen.findByRole('option', { name: 'SPEC-009' })).toBeTruthy();
  });

  it('mostra o botão da ação quando a atividade carrega', async () => {
    const rows = [...rowsOf(50), activity({ spec: 'SPEC-001', detail: 'da pergunta' })];
    stubFetch({ rows: rows.map((row) => (row.detail.startsWith('linha') ? { ...row, spec: 'SPEC-002', task: '' } : row)), view: () => overview({ questions: [question] }) });
    const observer = stubObserver();
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 50');
    expect(screen.queryByRole('button', { name: 'Responder' })).toBeNull();
    await observer.reach();
    expect(await screen.findByRole('button', { name: 'Responder' })).toBeTruthy();
  });

  it('mantém a lista quando a próxima página falha', async () => {
    stubFetch({ rows: rowsOf(120), failAt: 2 });
    render(<ActivityPage version={0} />);
    await screen.findByText('linha 120');
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    expect(await screen.findByRole('alert')).toBeTruthy();
    expect(lines()).toHaveLength(50);
    fireEvent.click(screen.getByRole('button', { name: 'Carregar mais' }));
    await waitFor(() => expect(lines()).toHaveLength(100));
  });
});
