import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import cardStyles from '../../ui/card/card.module.css';
import { TaskPage } from './task-page';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const state = (over: object = {}) => ({ id: 'TASK-02', title: 'Criar a tela', description: 'Descrição', status: 'in_progress', wave: '2', profile: 'padrao', persona: 'dev', model: 'm', depends_on: 'TASK-01', covers: 'RF-01', approved_at: '', attempts: '0', tests_result: '', regression_result: '', evidence_result: '', ...over });
const taskDetail = (over: object = {}, evidence: object[] = []) => ({
  spec: 'SPEC-001',
  state: state(over),
  files: { executivo: '## Resumo\nTexto executivo.', tecnico: '## T\nTexto técnico.', criterios: '## C\n- [ ] item', execucao: '## E\nPassos.', verificacao: '## V\nComandos.', registro: '## R\nLinhas.' },
  evidence,
  manual_evidence: [],
});
const specDetail = { meta: { id: 'SPEC-001', title: 'Spec', status: 'planned' }, path: 'p', plan: true, markdown: '# S\n', tasks: [{ id: 'TASK-01', title: 'a', status: 'done', wave: '1', profile: 'padrao', covers: '' }] };

function mockFetch(task: unknown, status = 200) {
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task, status) : json(specDetail))));
}
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
  globalThis.IntersectionObserver = class { observe() {} disconnect() {} unobserve() {} takeRecords() { return []; } } as never;
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); history.replaceState(null, '', '/'); });
async function openEvidence() { fireEvent.click(await screen.findByRole('tab', { name: 'Evidências' })); }

describe('TaskPage', () => {
  it('divide a tarefa nas abas Resumo, Evidências e Para o agente', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    const tabs = await screen.findAllByRole('tab');
    expect(tabs.map((t) => t.textContent)).toEqual(['Resumo', 'Evidências', 'Para o agente']);
    expect(screen.getByRole('tablist', { name: 'Partes da tarefa' })).toBeTruthy();
    expect(tabs[0].getAttribute('aria-selected')).toBe('true');
    const h2 = () => screen.getAllByRole('heading', { level: 2 }).map((h) => h.textContent);
    expect(h2()).toEqual(['Resumo executivo', 'Detalhes técnicos', 'Critérios de conclusão']);
    fireEvent.click(screen.getByRole('tab', { name: 'Para o agente' }));
    expect(h2()).toEqual(['Execução', 'Verificação', 'Registro de execução']);
    expect(screen.queryByRole('heading', { level: 2, name: 'Resumo executivo' })).toBeNull();
  });

  it('esconde onda, perfil e persona vazios', async () => {
    mockFetch(taskDetail({ wave: '', profile: '', persona: '' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Status');
    expect(screen.queryByText('Onda')).toBeNull();
    expect(screen.queryByText('Perfil')).toBeNull();
    expect(screen.queryByText('Persona')).toBeNull();
    expect(screen.getByText('Status')).toBeTruthy();
    expect(screen.getByText('Modelo')).toBeTruthy();
  });

  it('mostra onda, perfil e persona com valor', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Status');
    expect(screen.getByText('Onda')).toBeTruthy();
    expect(screen.getByText('Perfil')).toBeTruthy();
    expect(screen.getByText('Persona')).toBeTruthy();
  });

  it('não oferece Editar em nenhuma parte', async () => {
    for (const status of ['in_progress', 'done']) {
      mockFetch(taskDetail({ status }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('Texto executivo.');
      expect(screen.queryByRole('button', { name: 'Editar' })).toBeNull();
      cleanup();
    }
  });

  it('shows the empty evidence note and the error state', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    expect(await screen.findByText(/aparecem aqui quando a tarefa é finalizada/)).toBeTruthy();
    cleanup();
    mockFetch({ erro: 'x' }, 500);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect(await screen.findByText('Não foi possível abrir TASK-02')).toBeTruthy();
  });
  it('mostra a aba Evidências com o resumo e as categorias', async () => {
    const reviews = [
      { id: 'EV-01', name: 'a.txt', kind: 'automática', status: 'a conferir', reason: '' },
      { id: 'EV-02', name: 'b.txt', kind: 'automática', status: 'conferida', reason: '', reviewed_at: '2020-01-02' },
    ];
    mockFetch({ ...taskDetail({}, [{ name: 'a.txt', path: 'x/a.txt', size: 1 }, { name: 'b.txt', path: 'x/b.txt', size: 1 }]), reviews });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    expect(await screen.findByRole('group', { name: 'Filtrar por estado' })).toBeTruthy();
    expect(screen.getByRole('heading', { level: 3, name: /A conferir/ })).toBeTruthy();
    expect(screen.getByRole('heading', { level: 3, name: /Conferidas/ })).toBeTruthy();
    expect(screen.queryByRole('group', { name: 'Filtrar evidências' })).toBeNull();
  });

  it('não mostra botão de ação em nenhum status', async () => {
    for (const status of ['pending', 'approved', 'in_progress', 'validating', 'blocked', 'done']) {
      mockFetch(taskDetail({ status }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('Texto executivo.');
      for (const button of screen.queryAllByRole('button')) expect(button.textContent ?? '').not.toMatch(/Aprovar|Executar|Finalizar|Ajustar|Decidir|Editar|Aceitar|Rejeitar|Anexar|Trocar/);
      cleanup();
    }
  });
});

describe('resposta real da API', () => {
  it('mostra a tarefa quando a resposta não traz manual_evidence', async () => {
    const { manual_evidence: _omit, ...real } = taskDetail({}, [{ name: 'painel-binario.txt', path: 'x/painel-binario.txt', size: 1562 }]);
    mockFetch({ ...real, archived: false });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect(await screen.findByRole('heading', { level: 2, name: 'Resumo executivo' })).toBeTruthy();
    expect(screen.queryByText('Carregando TASK-02')).toBeNull();
  });
});

describe('anexo de evidência', () => {
  const manual = (over: object = {}) => ({ name: 'tela.png', path: 'x/tela.png', size: 0, description: 'Print da tela final.', extension: 'png', attached: false, ...over });
  const attachDetail = (item: object, stateOver: object = {}) => {
    const base = taskDetail(stateOver, (item as { attached: boolean }).attached ? [{ name: (item as { name: string }).name, path: (item as { path: string }).path, size: (item as { size: number }).size }] : []);
    return { ...base, manual_evidence: [item] };
  };
  const open = async (detail: unknown) => {
    vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(detail) : json(specDetail))));
    const view = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    return view;
  };

  afterEach(() => {
    history.pushState(null, '', '/');
    vi.restoreAllMocks();
  });

  it('abre a aba Evidências com #evidencias', async () => {
    history.pushState(null, '', '/tarefas/SPEC-001/TASK-02#evidencias');
    const { rerender } = await open(attachDetail(manual()));
    const tab = () => screen.getByRole('tab', { name: 'Evidências' });
    expect(tab().getAttribute('aria-selected')).toBe('true');
    expect(await screen.findByText(/falta anexar/)).toBeTruthy();
    const taskFetches = () => vi.mocked(fetch).mock.calls.filter(([url]) => String(url).startsWith('/api/tarefas')).length;
    const before = taskFetches();
    rerender(<TaskPage spec="SPEC-001" task="TASK-02" version={1} />);
    await waitFor(() => expect(taskFetches()).toBeGreaterThan(before));
    expect(tab().getAttribute('aria-selected')).toBe('true');
  });
});

describe('TaskPage em abas', () => {
  const tabNames = ['Resumo', 'Evidências', 'Para o agente'];

  it('guarda a aba aberta no hash', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('tab', { name: 'Para o agente' }));
    expect(window.location.hash).toBe('#agente');
  });

  it('abre Resumo com hash desconhecido', async () => {
    history.pushState(null, '', '/tarefas/SPEC-001/TASK-02#nada');
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect((await screen.findByRole('tab', { name: 'Resumo' })).getAttribute('aria-selected')).toBe('true');
  });

  it('não mostra o índice nem os cards e mantém o aside', async () => {
    mockFetch(taskDetail());
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByRole('tab', { name: 'Resumo' });
    expect(screen.queryByRole('navigation', { name: 'Nesta tarefa' })).toBeNull();
    for (const name of tabNames) {
      fireEvent.click(screen.getByRole('tab', { name }));
      expect(container.querySelector('.' + cardStyles.root)).toBeNull();
    }
    const aside = within(screen.getByRole('complementary'));
    for (const label of ['Status', 'Onda', 'Depende de', 'Cobre', 'Perfil', 'Persona', 'Modelo', 'Aprovada', 'Tentativas', 'Validação']) expect(aside.getByText(label)).toBeTruthy();
  });
});

describe('Esperas no aside', () => {
  const waitsSpec = { ...specDetail, tasks: [{ id: 'TASK-01', title: 'a', status: 'done', wave: '1', profile: 'padrao', covers: '' }, { id: 'TASK-12', title: 'b', status: 'in_progress', wave: '1', profile: 'padrao', covers: '' }] };
  const mockWaits = (task: unknown) => vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task) : json(waitsSpec))));
  const aside = () => within(screen.getByRole('complementary'));
  const row = (label: string) => aside().getByText(label, { selector: 'dt' }).parentElement;

  it('mostra Espera, Fecha depois de e Depende de no aside', async () => {
    mockWaits(taskDetail({ waiting_on: '[TASK-03]', waiting_reason: 'motivo', closes_after: '[TASK-12]', depends_on: '[TASK-01]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Fecha depois de', { selector: 'dt' });
    expect(row('Espera')?.textContent).toContain('TASK-03: motivo');
    expect(row('Fecha depois de')?.textContent).toContain('TASK-12');
    expect(row('Depende de')?.textContent).toContain('TASK-01');
  });

  it('esconde Fecha depois de quando a tarefa já está concluída', async () => {
    mockWaits(taskDetail({ closes_after: '[TASK-01]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Depende de', { selector: 'dt' });
    await waitFor(() => expect(aside().getByText('TASK-01', { selector: 'a' })).toBeTruthy());
    expect(aside().queryByText('Fecha depois de', { selector: 'dt' })).toBeNull();
  });

  it('não mostra espera sem waiting_on', async () => {
    mockWaits(taskDetail({ status: 'done', waiting_on: '[]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Depende de', { selector: 'dt' });
    expect(aside().queryByText('Espera', { selector: 'dt' })).toBeNull();
    expect(row('Depende de')?.textContent).toContain('TASK-01');
  });
});

const summary = (id: string, wave: string) => ({ id, title: id, status: 'done', wave, profile: 'padrao', covers: '' });
const waveSpec = { ...specDetail, tasks: [summary('TASK-01', '1'), summary('TASK-02', '1'), summary('TASK-03', '1'), summary('TASK-04', '2')] };
const toReview = [{ name: 'a.txt', kind: 'automática', status: 'a conferir', reason: '' }];
const reviewed = [{ name: 'a.txt', kind: 'automática', status: 'conferida', reason: '' }];

function mockRoutes(details: Record<string, unknown> = {}, tasks = waveSpec) {
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    if (!url.startsWith('/api/tarefas')) return Promise.resolve(json(tasks));
    const id = url.split('/').pop() as string;
    const body = details[id] ?? taskDetail({ id, wave: '1' });
    return Promise.resolve(typeof body === 'number' ? json({}, body) : json(body));
  }));
}
const withReviews = (id: string, reviews: object[]) => ({ ...taskDetail({ id, wave: '1' }), reviews });
const renderTask = async (task = 'TASK-02') => {
  render(<TaskPage spec="SPEC-001" task={task} version={0} />);
  return screen.findByRole('navigation', { name: 'Tarefas da onda' });
};

describe('Barra da onda', () => {
  beforeEach(() => { window.scrollTo = vi.fn(); });

  it('volta por padrão ao quadro filtrado pela spec', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    expect(screen.getByRole('link', { name: /Quadro de SPEC-001|Voltar/ }).getAttribute('href')).toBe('/tarefas?spec=SPEC-001');
  });

  it('mostra a anterior, a próxima e a posição na onda', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    const bar = await renderTask();
    expect(bar.textContent).toContain('TASK-02 · Onda 1 · 2 de 3');
    expect(within(bar).getByRole('link', { name: 'Tarefa anterior: TASK-01' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
    expect(within(bar).getByRole('link', { name: 'Próxima tarefa: TASK-03' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-03');
  });

  it('abre a próxima com → e a anterior com ←', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    await waitFor(() => {
      fireEvent.keyDown(document.body, { key: 'ArrowRight' });
      expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-03');
    });
    history.replaceState(null, '', '/');
    fireEvent.keyDown(document.body, { key: 'ArrowLeft' });
    expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-01');
  });

  it('não troca de tarefa com as setas nas abas, num campo ou com modificador', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Resumo' }), { key: 'ArrowRight' });
    const input = document.body.appendChild(document.createElement('input'));
    fireEvent.keyDown(input, { key: 'ArrowRight' });
    fireEvent.keyDown(document.body, { key: 'ArrowRight', altKey: true });
    input.remove();
    expect(window.location.pathname).toBe('/');
  });

  it('mostra só a posição na tarefa única na onda ou sem onda', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '2' }) }, { ...waveSpec, tasks: [summary('TASK-01', '1'), summary('TASK-02', '2')] });
    const alone = await renderTask();
    expect(alone.textContent).toContain('TASK-02 · Onda 2 · 1 de 1');
    expect(within(alone).queryAllByRole('link')).toHaveLength(0);
    expect(within(alone).queryByRole('button', { name: 'Próxima a conferir' })).toBeNull();
    cleanup();
    mockRoutes({ 'TASK-02': taskDetail({ wave: '' }) });
    const none = await renderTask();
    expect(none.textContent).toContain('TASK-02 · Sem onda');
    expect(within(none).queryAllByRole('link')).toHaveLength(0);
    expect(within(none).queryByRole('button', { name: 'Próxima a conferir' })).toBeNull();
  });
});

describe('Próxima a conferir', () => {
  beforeEach(() => { window.scrollTo = vi.fn(); });

  it('pula para a próxima tarefa da onda com evidência a conferir', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': withReviews('TASK-03', reviewed), 'TASK-01': withReviews('TASK-01', toReview) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(window.location.pathname + window.location.hash).toBe('/tarefas/SPEC-001/TASK-01#evidencias'));
    expect(screen.getByRole('tab', { name: 'Evidências' }).getAttribute('aria-selected')).toBe('true');
  });

  it('avisa e fica na tarefa quando nenhuma tem evidência a conferir', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': withReviews('TASK-03', reviewed), 'TASK-01': withReviews('TASK-01', reviewed) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(screen.getByRole('status').textContent).toBe('Nenhuma outra tarefa da onda tem evidência a conferir.'));
    expect(window.location.pathname).toBe('/');
  });

  it('pula a tarefa que não abre e segue para a próxima', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': 500, 'TASK-01': withReviews('TASK-01', toReview) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-01'));
  });
});
