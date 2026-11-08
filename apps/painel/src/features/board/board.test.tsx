import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { Board } from './board';
import { TasksPage } from './tasks-page';

const task = (id: string, status: string, over: object = {}) => ({ id, title: `Título ${id}`, status, wave: '1', profile: 'padrao', persona: 'dev', model: '', depends_on: '', covers: '', type: 'tarefa', description: '', last_outcome: null, ...over });
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
  const overview = { project: 'p', archived: [], specs: [
    { meta: meta('SPEC-001'), plan: false, plan_outdated: false, tasks: [task('TASK-01', 'pending')] },
    { meta: meta('SPEC-002'), plan: false, plan_outdated: false, tasks: [task('TASK-07', 'pending')] },
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
    expect(wave1.getByText('Título TASK-02')).toBeTruthy();
    const wave2 = within(screen.getByRole('group', { name: 'Onda 2' }));
    expect(wave2.queryByText('em execução')).toBeNull();
    expect(wave2.getByText('Título TASK-03')).toBeTruthy();
  });

  it('mostra a onda em execução', () => {
    render(<Board tasks={[{ ...task('TASK-01', 'in_progress'), spec: 'SPEC-001' }, { ...task('TASK-04', 'approved', { depends_on: '[TASK-09]' }), spec: 'SPEC-001' }]} />);
    const wave1 = within(screen.getByRole('group', { name: 'Onda 1' }));
    expect(wave1.getByText('em execução')).toBeTruthy();
  });

  it('mostra a spec no grupo do quadro de todas as specs', () => {
    render(<Board tasks={waves} showSpec />);
    expect(screen.getByRole('group', { name: 'SPEC-001 · Onda 1' })).toBeTruthy();
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

  it('agrupa sem onda', () => {
    render(<Board tasks={[t('TASK-01', 'approved', { wave: '' }), t('TASK-02', 'approved', { wave: '' }), t('TASK-03', 'approved', { wave: '2' })]} />);
    const group = within(screen.getByRole('group', { name: 'Sem onda' }));
    expect(group.getByText('Título TASK-01')).toBeTruthy();
    expect(group.getByText('Título TASK-02')).toBeTruthy();
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

  it('mostra as tarefas que faltam com a onda, com as concluídas recolhidas', () => {
    render(<Board tasks={[t('TASK-01', 'done'), t('TASK-02', 'in_progress'), t('TASK-03', 'approved', { wave: '2', depends_on: '[TASK-02]' })]} />);
    expect(within(screen.getByRole('region', { name: 'Concluídas' })).getByText('1')).toBeTruthy();
    expect(screen.queryByText('Título TASK-01')).toBeNull();
    expect(within(section('Em execução').getByText('Título TASK-02').closest('li') as HTMLElement).getByText('onda 1')).toBeTruthy();
    const approved = section('Aprovadas').getByText('Título TASK-03').closest('li') as HTMLElement;
    expect(within(approved).getByText('onda 2')).toBeTruthy();
  });

  it('não mostra botão de execução nem selo do agente', () => {
    render(<Board tasks={[t('TASK-01', 'approved'), t('TASK-02', 'approved', { wave: '2' }), t('TASK-03', 'in_progress')]} />);
    expect(screen.queryByRole('button', { name: /^Executar/ })).toBeNull();
    expect(screen.queryByText(/Com o agente/)).toBeNull();
  });
});
