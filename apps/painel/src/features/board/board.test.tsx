import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { Board } from './board';
import { statusLabel } from '../../shared/status';
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
afterEach(() => { cleanup(); vi.unstubAllGlobals(); localStorage.clear(); });

describe('Board', () => {
  it('mostra a cor de status nas colunas do fluxo', () => {
    const { container } = render(<Board tasks={[]} />);
    const tone = (column: string) => container.querySelector(`section[data-column="${column}"] header [data-tone]`);
    const expected: Record<string, string> = { ready: 'success', running: 'info', executed: 'success', validating: 'accent', blocked: 'warning', done: 'success' };
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
    expect(screen.getByText('Depende de TASK-09')).toBeTruthy();
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
    expect(columns.map((column) => column.querySelector('h3')?.textContent)).toEqual(['Para aprovar', 'Aprovadas', 'Prontas para execução', 'Em execução', 'Prontas para finalizar', 'Em validação', 'Bloqueadas', 'Concluídas']);
    expect(within(columns[3] as HTMLElement).getByText('Título TASK-04')).toBeTruthy();
    expect(within(columns[5] as HTMLElement).getByText('Título TASK-05')).toBeTruthy();
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

  it('mostra oito seções sem raia lado a lado, com Nenhuma nas vazias', () => {
    const { container } = render(<Board tasks={[]} />);
    const sections = Array.from(container.querySelectorAll('section[data-column]'));
    expect(sections.map((s) => s.querySelector('h3')?.textContent)).toEqual(['Para aprovar', 'Aprovadas', 'Prontas para execução', 'Em execução', 'Prontas para finalizar', 'Em validação', 'Bloqueadas', 'Concluídas']);
    expect(screen.getAllByText('Nenhuma')).toHaveLength(8);
  });

  it('separa Prontas de Aprovadas', () => {
    render(<Board tasks={[t('TASK-A', 'approved'), t('TASK-B', 'approved', { depends_on: '[TASK-C]' }), t('TASK-C', 'in_progress')]} />);
    const prontas = section('Prontas para execução');
    expect(prontas.getByText('Título TASK-A')).toBeTruthy();
    const aprovadas = section('Aprovadas');
    expect(aprovadas.getByText('Título TASK-B')).toBeTruthy();
    expect(aprovadas.getByText('Depende de TASK-C')).toBeTruthy();
    expect(aprovadas.queryAllByRole('button')).toHaveLength(0);
  });

  it('mantém em Aprovadas quem espera dependência que não existe', () => {
    render(<Board tasks={[t('TASK-B', 'approved', { depends_on: '[TASK-99]' })]} />);
    const aprovadas = section('Aprovadas');
    expect(aprovadas.getByText('Depende de TASK-99')).toBeTruthy();
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

describe('Board agrupado e ordenado', () => {
  it('agrupa por Spec e por Onda, com Etapa por padrão', () => {
    const tasks = [
      { ...task('TASK-01', 'approved', { wave: '1' }), spec: 'SPEC-001' },
      { ...task('TASK-02', 'pending', { wave: '2' }), spec: 'SPEC-002' },
    ];
    const { rerender } = render(<Board tasks={tasks} />);
    expect(screen.getByRole('region', { name: 'Para aprovar' })).toBeTruthy();
    rerender(<Board tasks={tasks} group="spec" />);
    expect(screen.getByRole('region', { name: 'SPEC-001' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'SPEC-002' })).toBeTruthy();
    rerender(<Board tasks={tasks} group="onda" />);
    expect(screen.getByRole('region', { name: 'Onda 1' })).toBeTruthy();
    expect(screen.getByRole('region', { name: 'Onda 2' })).toBeTruthy();
    expect(screen.queryByRole('region', { name: 'Para aprovar' })).toBeNull();
  });

  it('ordena os cartões do grupo por número, spec e onda', () => {
    const tasks = [
      { ...task('TASK-01', 'pending', { wave: '3' }), spec: 'SPEC-002' },
      { ...task('TASK-02', 'pending', { wave: '2' }), spec: 'SPEC-001' },
      { ...task('TASK-10', 'pending', { wave: '1' }), spec: 'SPEC-001' },
    ];
    const hrefs = () => within(screen.getByRole('region', { name: 'Para aprovar' })).getAllByRole('link').map((link) => link.getAttribute('href'));
    const a = '/tarefas/SPEC-002/TASK-01';
    const b = '/tarefas/SPEC-001/TASK-02';
    const c = '/tarefas/SPEC-001/TASK-10';
    const { rerender } = render(<Board tasks={tasks} showSpec order="numero" />);
    expect(hrefs()).toEqual([a, b, c]);
    rerender(<Board tasks={tasks} showSpec order="spec" />);
    expect(hrefs()).toEqual([b, c, a]);
    rerender(<Board tasks={tasks} showSpec order="onda" />);
    expect(hrefs()).toEqual([c, b, a]);
  });

  it('mostra a etapa no cartão fora do agrupamento por Etapa', () => {
    const tasks = [{ ...task('TASK-01', 'pending'), spec: 'SPEC-001' }];
    const { rerender } = render(<Board tasks={tasks} group="spec" />);
    expect(screen.getByText(statusLabel('pending'))).toBeTruthy();
    rerender(<Board tasks={tasks} />);
    expect(within(screen.getByRole('region', { name: 'Para aprovar' })).queryByText(statusLabel('pending'))).toBeNull();
  });
});

describe('TasksPage com seletores', () => {
  const overview = { project: 'p', archived: [], specs: [
    { meta: meta('SPEC-001'), plan: false, plan_outdated: false, tasks: [task('TASK-01', 'pending', { wave: '1', persona: 'dev' }), task('TASK-02', 'pending', { wave: '2', persona: 'dev' }), task('TASK-03', 'pending', { wave: '2', persona: 'qa' })] },
    { meta: meta('SPEC-002'), plan: false, plan_outdated: false, tasks: [task('TASK-07', 'pending', { wave: '2', persona: 'dev' })] },
  ] };
  const pick = (name: string, value: string) => fireEvent.change(screen.getByRole('combobox', { name }), { target: { value } });
  const value = (name: string) => (screen.getByRole('combobox', { name }) as HTMLSelectElement).value;
  const open = async () => { mockFetch(() => json(overview)); render(<TasksPage version={0} />); await screen.findByText('Título TASK-07'); };

  it('soma os filtros de spec, onda e persona', async () => {
    await open();
    fireEvent.click(screen.getByRole('button', { name: /SPEC-001/ }));
    pick('Onda', '2');
    pick('Persona', 'dev');
    expect(screen.getByText('Título TASK-02')).toBeTruthy();
    expect(screen.queryByText('Título TASK-01')).toBeNull();
    expect(screen.queryByText('Título TASK-03')).toBeNull();
    expect(screen.queryByText('Título TASK-07')).toBeNull();
  });

  it('mostra o estado vazio com os filtros visíveis', async () => {
    await open();
    fireEvent.click(screen.getByRole('button', { name: /SPEC-002/ }));
    pick('Persona', 'qa');
    expect(screen.getByText('Nenhuma tarefa com esses filtros')).toBeTruthy();
    expect(screen.getByRole('combobox', { name: 'Persona' })).toBeTruthy();
  });

  it('guarda e restaura as escolhas do quadro', async () => {
    await open();
    pick('Agrupar por', 'onda');
    pick('Ordenar por', 'spec');
    pick('Onda', '2');
    pick('Persona', 'dev');
    expect(localStorage.getItem('vostok.quadro.agrupamento')).toBe('onda');
    expect(localStorage.getItem('vostok.quadro.ordem')).toBe('spec');
    expect(localStorage.getItem('vostok.quadro.onda')).toBe('2');
    expect(localStorage.getItem('vostok.quadro.persona')).toBe('dev');
    cleanup();
    await open();
    expect([value('Agrupar por'), value('Ordenar por'), value('Onda'), value('Persona')]).toEqual(['onda', 'spec', '2', 'dev']);
  });

  it('abre com os padrões quando o localStorage falha', async () => {
    const blocked = () => { throw new Error('bloqueado'); };
    vi.stubGlobal('localStorage', { getItem: blocked, setItem: blocked, clear: () => undefined });
    await open();
    expect([value('Agrupar por'), value('Ordenar por'), value('Onda'), value('Persona')]).toEqual(['etapa', 'numero', 'todas', 'todas']);
    pick('Agrupar por', 'spec');
    expect(screen.getByRole('region', { name: 'SPEC-001' })).toBeTruthy();
  });

  it('usa o padrão com valor desconhecido guardado', async () => {
    localStorage.setItem('vostok.quadro.agrupamento', 'acao');
    localStorage.setItem('vostok.quadro.ordem', 'x');
    await open();
    expect([value('Agrupar por'), value('Ordenar por')]).toEqual(['etapa', 'numero']);
  });
});

describe('Esperas no cartão', () => {
  const t = (id: string, status: string, over: object = {}) => ({ ...task(id, status, over), spec: 'SPEC-001' });
  const card = (id: string) => within(screen.getByRole('link', { name: new RegExp(id) }));
  const waits = { waiting_on: '[TASK-03]', waiting_reason: 'motivo', closes_after: '[TASK-12]', depends_on: '[TASK-01]' };

  it('mostra Espera, Fecha depois de e Depende de em textos distintos', () => {
    render(<Board tasks={[t('TASK-01', 'in_progress'), t('TASK-12', 'in_progress'), t('TASK-05', 'approved', waits)]} />);
    const espera = card('TASK-05').getByText('Espera TASK-03: motivo');
    const fecha = card('TASK-05').getByText('Fecha depois de TASK-12');
    const depende = card('TASK-05').getByText('Depende de TASK-01');
    expect(new Set([espera, fecha, depende]).size).toBe(3);
  });

  it('esconde Fecha depois de quando a tarefa já está concluída', () => {
    render(<Board tasks={[t('TASK-01', 'in_progress'), t('TASK-12', 'done'), t('TASK-05', 'approved', waits)]} />);
    expect(card('TASK-05').queryByText(/Fecha depois de/)).toBeNull();
  });

  it('não mostra espera na tarefa concluída sem waiting_on', () => {
    render(<Board tasks={[t('TASK-05', 'done', { waiting_on: '[]', closes_after: '[]', depends_on: '[TASK-01]' })]} />);
    fireEvent.click(screen.getByRole('button', { name: 'Concluídas' }));
    expect(card('TASK-05').queryByText(/^Espera/)).toBeNull();
    expect(card('TASK-05').queryByText(/Fecha depois de/)).toBeNull();
    expect(card('TASK-05').queryByText(/Depende de/)).toBeNull();
  });

  it('abre como hoje sem os campos novos', () => {
    render(<Board tasks={[t('TASK-05', 'approved')]} />);
    expect(card('TASK-05').getByText('Título TASK-05')).toBeTruthy();
    expect(card('TASK-05').queryByText(/^Espera|Fecha depois de|Depende de/)).toBeNull();
  });
});

describe('Prontas para finalizar', () => {
  const t = (id: string, status: string, over: object = {}) => ({ ...task(id, status, over), spec: 'SPEC-001' });

  it('põe a tarefa executed na coluna Prontas para finalizar', () => {
    const { container } = render(<Board tasks={[t('TASK-04', 'in_progress'), t('TASK-06', 'executed')]} />);
    expect(within(container.querySelector('section[data-column="executed"]') as HTMLElement).getByText('Título TASK-06')).toBeTruthy();
    expect(within(container.querySelector('section[data-column="running"]') as HTMLElement).getByText('Título TASK-04')).toBeTruthy();
  });

  it('mostra o badge pronta para finalizar', () => {
    render(<Board tasks={[t('TASK-06', 'executed')]} group="spec" />);
    expect(screen.getByText('pronta para finalizar')).toBeTruthy();
  });

  it('mostra a onda só com tarefas executed como em execução', () => {
    render(<Board tasks={[t('TASK-01', 'executed'), t('TASK-02', 'approved')]} />);
    expect(within(screen.getByRole('group', { name: 'Onda 1' })).getByText('em execução')).toBeTruthy();
  });
});
