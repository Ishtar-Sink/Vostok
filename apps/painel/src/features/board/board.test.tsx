import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import type { PanelRequest } from '../../shared/api';
import { Board } from './board';
import { TasksPage } from './tasks-page';

const task = (id: string, status: string, over: object = {}) => ({ id, title: `Título ${id}`, status, wave: '1', profile: 'padrao', persona: 'dev', model: '', depends_on: '', covers: '', type: 'tarefa', description: '', last_outcome: null, ...over });
const meta = (id: string, status = 'planned') => ({ id, title: `Spec ${id}`, status, type: 'feature', author: 'ana', created_at: '2026-01-02', updated_at: '2026-01-03', refined_at: '', approved_at: '2026-01-04', adjustments: '' });
const request = (id: string, acao: string, over: Partial<PanelRequest> = {}): PanelRequest => ({ id, acao, spec: 'SPEC-001', tarefa: '', onda: '', estado: 'em_andamento', atualizado_em: '2026-11-03T10:00:00Z', ...over });
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

describe('Board', () => {
  it('mostra a cor de status nas colunas do fluxo', () => {
    const { container } = render(<Board tasks={[]} />);
    const tone = (column: string) => container.querySelector(`section[data-column="${column}"] header [data-tone]`);
    const expected: Record<string, string> = { ready: 'success', running: 'info', validating: 'accent', blocked: 'warning', done: 'success' };
    for (const [column, value] of Object.entries(expected)) {
      expect(tone(column)?.getAttribute('data-tone')).toBe(value);
      expect(tone(column)?.getAttribute('aria-hidden')).toBe('true');
    }
    expect(tone('pending')).toBeNull();
    expect(tone('approved')).toBeNull();
  });

  it('mostra o título com aspas como veio', () => {
    render(<Board tasks={[{ ...task('TASK-02', 'in_progress', { title: 'Separar a coluna "Em validação" no quadro' }), spec: 'SPEC-001' }]} />);
    expect(screen.getByText('Separar a coluna "Em validação" no quadro')).toBeTruthy();
  });

  it('groups the cards by status column and flags waiting dependencies', () => {
    const tasks = [
      { ...task('TASK-01', 'done'), spec: 'SPEC-001' },
      { ...task('TASK-02', 'approved', { depends_on: '[TASK-01]' }), spec: 'SPEC-001' },
      { ...task('TASK-03', 'approved', { depends_on: '[TASK-09]' }), spec: 'SPEC-001' },
    ];
    render(<Board tasks={tasks} />);
    expect(within(screen.getByRole('region', { name: 'Prontas para execução' })).getByText('Título TASK-02')).toBeTruthy();
    expect(screen.getByText('Espera TASK-09')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: 'Concluídas' }));
    expect(screen.getByRole('link', { name: /TASK-01/ }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
  });

  it('mostra a espera declarada e o motivo na tarefa bloqueada', () => {
    const tasks = [{ ...task('TASK-02', 'blocked', { waiting_on: '[TASK-01]', waiting_reason: 'espera o candidato', depends_on: '[TASK-01, TASK-09]' }), spec: 'SPEC-001' }];
    const { container } = render(<Board tasks={tasks} />);
    const column = container.querySelector('section[data-column="blocked"]') as HTMLElement;
    expect(within(column).getByText('Espera TASK-01')).toBeTruthy();
    expect(within(column).getByText('espera o candidato')).toBeTruthy();
    expect(screen.queryByText(/TASK-09/)).toBeNull();
  });

  it('bloqueada por falha não mostra motivo', () => {
    const tasks = [{ ...task('TASK-03', 'blocked', { waiting_on: '[]', waiting_reason: '' }), spec: 'SPEC-001' }];
    const { container } = render(<Board tasks={tasks} />);
    const column = container.querySelector('section[data-column="blocked"]') as HTMLElement;
    expect(within(column).queryByText(/Espera/)).toBeNull();
    expect(within(column).queryByText('espera o candidato')).toBeNull();
  });

  it('separa em validação de em execução', () => {
    const tasks = [
      { ...task('TASK-04', 'in_progress'), spec: 'SPEC-001' },
      { ...task('TASK-05', 'validating'), spec: 'SPEC-001' },
    ];
    const { container } = render(<Board tasks={tasks} />);
    const columns = Array.from(container.querySelectorAll('section[data-column]'));
    expect(columns.map((column) => column.querySelector('h3')?.textContent)).toEqual(['Para aprovar', 'Aprovadas', 'Prontas para execução', 'Em execução', 'Em validação', 'Bloqueadas', 'Concluídas']);
    expect(within(columns[3] as HTMLElement).getByText('Título TASK-04')).toBeTruthy();
    expect(within(columns[4] as HTMLElement).getByText('Título TASK-05')).toBeTruthy();
  });
});

describe('TasksPage', () => {
  const overview = { project: 'p', archived: [], activity: [], questions: [], specs: [
    { meta: meta('SPEC-001'), plan: false, tasks: [task('TASK-01', 'pending')] },
    { meta: meta('SPEC-002'), plan: false, tasks: [task('TASK-07', 'pending')] },
  ] };

  it('filters the board by spec', async () => {
    mockFetch(() => json(overview));
    render(<TasksPage version={0} />);
    expect(await screen.findByText('Título TASK-07')).toBeTruthy();
    fireEvent.click(screen.getByRole('button', { name: /SPEC-001/ }));
    expect(screen.queryByText('Título TASK-07')).toBeNull();
    expect(screen.getByText('Título TASK-01')).toBeTruthy();
  });

  it('shows the error state', async () => {
    mockFetch(() => new Response('x', { status: 500 }));
    render(<TasksPage version={0} />);
    expect(await screen.findByRole('button', { name: 'Tentar de novo' })).toBeTruthy();
  });

  it('passa os pedidos do agente ao quadro', async () => {
    mockFetch(() => json({ ...overview, requests: [request('PED-001', 'aprovar-plano')] }));
    render(<TasksPage version={0} />);
    const section = await screen.findByRole('region', { name: 'Para aprovar' });
    expect(within(section).getAllByText('Com o agente: Detalhando as tarefas')).toHaveLength(1);
    expect(within(section).getByText('Título TASK-07').closest('li')?.textContent).not.toContain('Com o agente');
  });
});

describe('Board por onda', () => {
  const waves = [
    { ...task('TASK-01', 'validating'), spec: 'SPEC-001' },
    { ...task('TASK-02', 'approved'), spec: 'SPEC-001' },
    { ...task('TASK-03', 'approved', { wave: '2', depends_on: '[TASK-01]' }), spec: 'SPEC-001' },
  ];

  it('agrupa as aprovadas por onda com o estado', () => {
    render(<Board tasks={waves} />);
    const wave1 = within(screen.getByRole('group', { name: 'Onda 1' }));
    expect(wave1.getByText('em execução')).toBeTruthy();
    expect(wave1.getByRole('button', { name: 'Executar onda 1' })).toBeTruthy();
    expect(wave1.getByText('Título TASK-02')).toBeTruthy();
    const wave2 = within(screen.getByRole('group', { name: 'Onda 2' }));
    expect(wave2.queryByText('em execução')).toBeNull();
    expect(wave2.getByText('Título TASK-03')).toBeTruthy();
    expect(wave2.queryByRole('button', { name: 'Executar onda 2' })).toBeNull();
  });

  it('mostra a onda em execução', () => {
    render(<Board tasks={[{ ...task('TASK-01', 'in_progress'), spec: 'SPEC-001' }, { ...task('TASK-04', 'approved', { depends_on: '[TASK-09]' }), spec: 'SPEC-001' }]} />);
    const wave1 = within(screen.getByRole('group', { name: 'Onda 1' }));
    expect(wave1.getByText('em execução')).toBeTruthy();
    expect(wave1.queryByRole('button', { name: 'Executar onda 1' })).toBeNull();
  });

  it('pede a execução da onda pronta', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-1' }));
    render(<Board tasks={waves} />);
    fireEvent.click(screen.getByRole('button', { name: 'Executar onda 1' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Executar onda' }).at(-1) as HTMLElement);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=executar-onda&spec=SPEC-001&onda=1', expect.objectContaining({ method: 'POST' })));
  });

  it('pede a execução da tarefa pronta pelo cartão', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-2' }));
    render(<Board tasks={waves} />);
    fireEvent.click(screen.getByRole('button', { name: 'Executar TASK-02' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Pedir execução' }).at(-1) as HTMLElement);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=executar-tarefa&spec=SPEC-001&tarefa=TASK-02', expect.objectContaining({ method: 'POST' })));
  });

  it('não oferece executar no cartão que espera', () => {
    render(<Board tasks={waves} />);
    expect(screen.queryByRole('button', { name: 'Executar TASK-03' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Executar TASK-01' })).toBeNull();
  });

  it('mostra a spec no grupo do quadro de todas as specs', () => {
    render(<Board tasks={waves} showSpec />);
    expect(screen.getByRole('group', { name: 'SPEC-001 · Onda 1' })).toBeTruthy();
  });
});

describe('Board com o agente', () => {
  const pending = [{ ...task('TASK-01', 'pending'), spec: 'SPEC-001' }, { ...task('TASK-02', 'pending'), spec: 'SPEC-001' }];
  const ready = [{ ...task('TASK-01', 'approved'), spec: 'SPEC-001' }, { ...task('TASK-02', 'approved'), spec: 'SPEC-001' }];
  const section = (name: string) => screen.getByRole('region', { name });

  it('põe o selo no cartão pronto, sem raia', () => {
    render(<Board tasks={ready} requests={[request('PED-002', 'executar-tarefa', { tarefa: 'TASK-01' })]} />);
    const prontas = within(section('Prontas para execução'));
    expect(prontas.getByText('Título TASK-01')).toBeTruthy();
    expect(prontas.getByText('Com o agente: Executando a TASK-01')).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Executar TASK-01' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Executar onda 1' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Executar TASK-02' })).toBeTruthy();
    expect(screen.queryByRole('region', { name: 'Com o agente' })).toBeNull();
  });

  it('põe o selo nas tarefas em detalhamento, em Para aprovar', () => {
    render(<Board tasks={pending} requests={[request('PED-001', 'aprovar-plano')]} />);
    expect(within(section('Para aprovar')).getAllByText('Com o agente: Detalhando as tarefas')).toHaveLength(2);
  });

  it('põe o selo só na onda pedida ao agente', () => {
    render(<Board tasks={[{ ...task('TASK-01', 'approved'), spec: 'SPEC-001' }, { ...task('TASK-03', 'approved', { wave: '2' }), spec: 'SPEC-001' }]} requests={[request('PED-004', 'executar-onda', { onda: '1' })]} />);
    const card = (title: string) => screen.getByText(title).closest('li') as HTMLElement;
    expect(within(card('Título TASK-01')).getByText('Com o agente: Executando a onda 1')).toBeTruthy();
    expect(within(card('Título TASK-03')).queryByText(/Com o agente/)).toBeNull();
    expect(screen.queryByRole('button', { name: 'Executar TASK-03' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Executar onda 2' })).toBeNull();
  });

  it('põe o selo do pedido da spec nos cartões prontos e esconde os botões', () => {
    render(<Board tasks={ready} requests={[request('PED-003', 'ajustar-spec')]} />);
    expect(within(section('Prontas para execução')).getAllByText('Com o agente: Ajustando a spec')).toHaveLength(2);
    expect(screen.queryByRole('button', { name: /Executar/ })).toBeNull();
  });

  it('não põe o selo do pedido da spec em quem não está pronto', () => {
    render(<Board tasks={[{ ...task('TASK-01', 'in_progress'), spec: 'SPEC-001' }]} requests={[request('PED-003', 'ajustar-spec')]} />);
    expect(screen.queryByText(/Com o agente/)).toBeNull();
  });

  it('mantém a tarefa em execução na própria seção, com o selo', () => {
    render(<Board tasks={[{ ...task('TASK-01', 'in_progress'), spec: 'SPEC-001' }]} requests={[request('PED-005', 'finalizar-tarefa', { tarefa: 'TASK-01' })]} />);
    expect(within(section('Em execução')).getByText('Com o agente: Finalizando a TASK-01')).toBeTruthy();
  });

  it('mostra a última nota e a hora abaixo do selo', () => {
    render(<Board tasks={ready} requests={[request('PED-002', 'executar-tarefa', { tarefa: 'TASK-01', nota: 'rodando os testes', nota_em: new Date(2026, 10, 3, 14, 7).toISOString() })]} />);
    const card = screen.getByText('Título TASK-01').closest('li') as HTMLElement;
    expect(within(card).getByText('Com o agente: Executando a TASK-01')).toBeTruthy();
    expect(within(card).getByText(/rodando os testes ·/).textContent).toMatch(/14:07$/);
  });

  it('oferece Ver notas só com nota', () => {
    render(<Board tasks={ready} requests={[request('PED-002', 'executar-tarefa', { tarefa: 'TASK-01' })]} />);
    expect(screen.queryByRole('button', { name: /Ver as notas/ })).toBeNull();
    cleanup();
    render(<Board tasks={ready} requests={[request('PED-002', 'executar-tarefa', { tarefa: 'TASK-01', nota: 'lendo', nota_em: '2026-11-03T14:07:00Z' })]} />);
    expect(screen.getAllByRole('button', { name: /Ver as notas/ })).toHaveLength(1);
  });

  it('abre a janela de notas pelo Ver notas', async () => {
    mockFetch(() => json({ id: 'PED-002', acao: 'executar-tarefa', spec: 'SPEC-001', tarefa: 'TASK-01', estado: 'em_andamento', texto: '', notas: [{ em: '2026-11-03T14:07:00Z', texto: 'lendo a spec', detalhe: '' }], resultado: '', lida_em: '' }));
    render(<Board tasks={ready} requests={[request('PED-002', 'executar-tarefa', { tarefa: 'TASK-01', nota: 'lendo a spec', nota_em: '2026-11-03T14:07:00Z' })]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Ver as notas de PED-002 em TASK-01' }));
    await waitFor(() => expect(screen.getByRole('dialog', { hidden: true }).textContent).toContain('Notas do pedido PED-002'));
    expect(await screen.findByRole('list', { name: 'Notas', hidden: true })).toBeTruthy();
  });

  it('pergunta livre não esconde Executar nem põe o selo', () => {
    render(<Board tasks={ready} requests={[request('PED-009', 'perguntar-agente', { estado: 'pendente' })]} />);
    expect(screen.getByRole('button', { name: 'Executar TASK-01' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Executar onda 1' })).toBeTruthy();
    expect(screen.queryByText(/Com o agente/)).toBeNull();
  });

  it('pergunta livre na fila deixa pedir a execução', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-010' }));
    render(<Board tasks={ready} requests={[request('PED-009', 'perguntar-agente', { estado: 'pendente' })]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Executar TASK-01' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Pedir execução' }).at(-1) as HTMLElement);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=executar-tarefa&spec=SPEC-001&tarefa=TASK-01', expect.objectContaining({ method: 'POST' })));
  });

  it('não mostra o selo sem pedido', () => {
    render(<Board tasks={ready} />);
    expect(screen.queryByText(/Com o agente/)).toBeNull();
    expect(screen.getByRole('button', { name: 'Executar TASK-01' })).toBeTruthy();
  });
});

describe('Board em listas', () => {
  const t = (id: string, status: string, over: object = {}) => ({ ...task(id, status, over), spec: 'SPEC-001' });
  const section = (name: string) => within(screen.getByRole('region', { name }));

  it('mostra sete seções sem raia lado a lado, com Nenhuma nas vazias', () => {
    const { container } = render(<Board tasks={[]} />);
    const sections = Array.from(container.querySelectorAll('section[data-column]'));
    expect(sections.map((s) => s.querySelector('h3')?.textContent)).toEqual(['Para aprovar', 'Aprovadas', 'Prontas para execução', 'Em execução', 'Em validação', 'Bloqueadas', 'Concluídas']);
    expect(screen.getAllByText('Nenhuma')).toHaveLength(7);
  });

  it('separa Prontas de Aprovadas', () => {
    render(<Board tasks={[t('TASK-A', 'approved'), t('TASK-B', 'approved', { depends_on: '[TASK-C]' }), t('TASK-C', 'in_progress')]} />);
    const prontas = section('Prontas para execução');
    expect(prontas.getByText('Título TASK-A')).toBeTruthy();
    expect(prontas.getByRole('button', { name: 'Executar TASK-A' })).toBeTruthy();
    expect(prontas.getByRole('button', { name: 'Executar onda 1' })).toBeTruthy();
    const aprovadas = section('Aprovadas');
    expect(aprovadas.getByText('Título TASK-B')).toBeTruthy();
    expect(aprovadas.getByText('Espera TASK-C')).toBeTruthy();
    expect(aprovadas.queryAllByRole('button')).toHaveLength(0);
  });

  it('mantém em Aprovadas quem espera dependência que não existe', () => {
    render(<Board tasks={[t('TASK-B', 'approved', { depends_on: '[TASK-99]' })]} />);
    const aprovadas = section('Aprovadas');
    expect(aprovadas.getByText('Espera TASK-99')).toBeTruthy();
    expect(aprovadas.queryAllByRole('button')).toHaveLength(0);
  });

  it('passa a tarefa para Prontas ao terminar a espera, sem recarregar', () => {
    const before = [t('TASK-B', 'approved', { depends_on: '[TASK-C]' }), t('TASK-C', 'in_progress')];
    const { rerender } = render(<Board tasks={before} />);
    const aprovadas = screen.getByRole('region', { name: 'Aprovadas' });
    expect(within(aprovadas).getByText('Título TASK-B')).toBeTruthy();
    rerender(<Board tasks={[before[0], { ...before[1], status: 'done' }]} />);
    expect(screen.getByRole('region', { name: 'Aprovadas' })).toBe(aprovadas);
    expect(within(aprovadas).queryByText('Título TASK-B')).toBeNull();
    expect(section('Prontas para execução').getByText('Título TASK-B')).toBeTruthy();
  });

  it('abre Concluídas recolhida, com a contagem, e expande', () => {
    const tasks = [t('TASK-01', 'done'), t('TASK-02', 'done')];
    const { rerender } = render(<Board tasks={tasks} />);
    const toggle = screen.getByRole('button', { name: 'Concluídas' });
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(within(screen.getByRole('region', { name: 'Concluídas' })).getByText('2')).toBeTruthy();
    expect(screen.queryByText('Título TASK-01')).toBeNull();
    toggle.focus();
    fireEvent.click(toggle);
    expect(screen.getByRole('button', { name: 'Concluídas' }).getAttribute('aria-expanded')).toBe('true');
    expect(screen.getByText('Título TASK-01')).toBeTruthy();
    expect(screen.getByText('Título TASK-02')).toBeTruthy();
    rerender(<Board tasks={[...tasks]} />);
    expect(screen.getByRole('button', { name: 'Concluídas' }).getAttribute('aria-expanded')).toBe('true');
  });

  it('mostra Nenhuma em Concluídas vazia, sem botão de expandir', () => {
    render(<Board tasks={[]} />);
    expect(screen.queryByRole('button', { name: 'Concluídas' })).toBeNull();
    expect(section('Concluídas').getByText('Nenhuma')).toBeTruthy();
  });

  it('põe o selo em execução só no grupo da própria onda', () => {
    render(<Board tasks={[t('TASK-01', 'in_progress'), t('TASK-02', 'approved'), t('TASK-03', 'approved', { wave: '2', depends_on: '[TASK-01]' })]} />);
    expect(within(screen.getByRole('group', { name: 'Onda 1' })).getByText('em execução')).toBeTruthy();
    expect(within(screen.getByRole('group', { name: 'Onda 2' })).queryByText('em execução')).toBeNull();
  });

  it('agrupa sem onda, sem Executar onda', () => {
    render(<Board tasks={[t('TASK-01', 'approved', { wave: '' }), t('TASK-02', 'approved', { wave: '' }), t('TASK-03', 'approved', { wave: '2' })]} />);
    const group = within(screen.getByRole('group', { name: 'Sem onda' }));
    expect(group.getByRole('button', { name: 'Executar TASK-01' })).toBeTruthy();
    expect(group.getByRole('button', { name: 'Executar TASK-02' })).toBeTruthy();
    expect(group.queryByRole('button', { name: /Executar onda/ })).toBeNull();
    const names = screen.getAllByRole('group').map((g) => g.getAttribute('aria-label'));
    expect(names).toEqual(['Onda 2', 'Sem onda']);
  });

  it('esconde onda, perfil e persona vazios no cartão', () => {
    const { container, rerender } = render(<Board tasks={[t('TASK-01', 'pending', { wave: '', profile: '', persona: '' })]} />);
    expect(container.textContent).not.toMatch(/onda/);
    expect(container.textContent).not.toContain('·');
    rerender(<Board tasks={[t('TASK-01', 'pending', { wave: '', profile: 'padrao', persona: '' })]} />);
    expect(screen.getByText('padrao')).toBeTruthy();
    expect(container.textContent).not.toContain('·');
  });
});
