import { act, cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import type { LoopRound, OpenSpec, PanelRequest } from '../shared/api';
import type { Route } from '../shared/router';
import { Sidebar } from './sidebar';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });

const at = (minutes: number, seconds = 0) => new Date(2026, 10, 3, 14, minutes, seconds).toISOString();
const loop = (ultima: string): LoopRound => ({ ultima_rodada: ultima, intervalo: 60, agente: 'claude' });
const queued: PanelRequest[] = [{ id: 'PED-001', acao: 'planejar-spec', spec: 'SPEC-001', tarefa: '', onda: '', estado: 'pendente', atualizado_em: at(0) }];
const active = [{ meta: { id: 'SPEC-001', title: 'Exportar' }, plan: true, plan_outdated: false, tasks: [], last_outcome: null }] as unknown as OpenSpec[];
const sidebar = (props: { loop: LoopRound | null; requests: PanelRequest[]; specs?: OpenSpec[]; unreadAnswers?: number; route?: Route }) =>
  render(<Sidebar name="demo" icon="" specs={props.specs ?? active} pending={0} unreadAnswers={props.unreadAnswers} route={props.route ?? { page: 'dashboard' }} theme="light" onToggleTheme={() => {}} drawer={false} open={false} onClose={() => {}} loop={props.loop} requests={props.requests} />);

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date', 'setInterval', 'clearInterval'] });
  vi.setSystemTime(new Date(2026, 10, 3, 14, 10));
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

describe('Sidebar e o loop de pedidos', () => {
  it('avisa o agente ocupado ou parado junto de Em andamento', () => {
    sidebar({ loop: loop(at(5)), requests: queued });
    expect(screen.getByRole('status').textContent).toBe('O agente está ocupado ou parado desde 14:05');
    expect(screen.getByText('Em andamento')).toBeTruthy();
  });

  it('avisa nenhum agente em loop sem o registro', () => {
    sidebar({ loop: null, requests: queued, specs: [] });
    expect(screen.getByRole('status').textContent).toBe('Nenhum agente em loop');
    expect(screen.getByText('Em andamento')).toBeTruthy();
  });

  it('não avisa com o loop em dia ou sem pedido na fila', () => {
    sidebar({ loop: loop(at(9)), requests: queued });
    expect(screen.queryByRole('status')).toBeNull();
    cleanup();
    sidebar({ loop: loop(at(5)), requests: [] });
    expect(screen.queryByRole('status')).toBeNull();
  });

  it('mostra o aviso quando o tempo passa sem nova rodada', () => {
    sidebar({ loop: loop(at(8, 30)), requests: queued });
    expect(screen.queryByRole('status')).toBeNull();
    act(() => {
      vi.advanceTimersByTime(60_000);
    });
    expect(screen.getByRole('status').textContent).toBe('O agente está ocupado ou parado desde 14:08');
  });
});

describe('Sidebar e as notas do agente', () => {
  const running = (over: Partial<PanelRequest> = {}): PanelRequest => ({ id: 'PED-009', acao: 'executar-tarefa', spec: 'SPEC-001', tarefa: 'TASK-01', onda: '', estado: 'em_andamento', atualizado_em: at(0), nota: 'rodando os testes', nota_em: at(7), ...over });
  const lateLoop = (over: Partial<LoopRound> = {}): LoopRound => ({ ultima_rodada: at(5), intervalo: 60, agente: 'claude', ...over });

  beforeAll(() => {
    HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
    HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
  });
  afterEach(() => vi.unstubAllGlobals());

  it('mostra a última nota e a hora', () => {
    sidebar({ loop: loop(at(9)), requests: [running()] });
    const button = screen.getByRole('button', { name: 'Ver as notas de PED-009: rodando os testes' });
    expect(button.textContent).toContain('rodando os testes');
    expect(button.textContent).toContain('hoje, 14:07');
  });

  it('sem nota, não mostra o botão', () => {
    sidebar({ loop: loop(at(9)), requests: [running({ nota: undefined, nota_em: undefined })] });
    expect(screen.queryByRole('button', { name: /Ver as notas/ })).toBeNull();
  });

  it('pedido na fila com nota não aparece', () => {
    sidebar({ loop: loop(at(9)), requests: [running({ estado: 'pendente' })] });
    expect(screen.queryByRole('button', { name: /Ver as notas/ })).toBeNull();
  });

  it('abre a janela de notas pelo botão', async () => {
    const fetchSpy = vi.fn(() => Promise.resolve(json({ id: 'PED-009', acao: 'executar-tarefa', spec: 'SPEC-001', tarefa: 'TASK-01', estado: 'em_andamento', texto: '', notas: [{ em: at(7), texto: 'rodando os testes', detalhe: 'Faltam dois.' }], resultado: '', lida_em: '' })));
    vi.stubGlobal('fetch', fetchSpy);
    sidebar({ loop: loop(at(9)), requests: [running()] });
    fireEvent.click(screen.getByRole('button', { name: /Ver as notas de PED-009/ }));
    expect(await screen.findByText('Faltam dois.')).toBeTruthy();
    expect(fetchSpy).toHaveBeenCalledWith('/api/pedidos/PED-009');
    expect(screen.getByRole('dialog', { hidden: true }).textContent).toContain('Notas do pedido PED-009');
    expect(document.querySelector('aside dialog')).toBeNull();
  });

  it('mostra o aviso de falta de nota no lugar de ocupado ou parado', () => {
    const requests = [running(), { ...queued[0], id: 'PED-010' }];
    sidebar({ loop: lateLoop({ sem_nota_pedido: 'PED-009', sem_nota_desde: at(5) }), requests });
    const statuses = screen.getAllByRole('status');
    expect(statuses).toHaveLength(1);
    expect(statuses[0].textContent).toBe('Sem nota do agente em PED-009 desde 14:05');
  });

  it('mantém o aviso de rodada atrasada sem o de falta de nota', () => {
    sidebar({ loop: lateLoop(), requests: [running(), { ...queued[0], id: 'PED-010' }] });
    expect(screen.getByRole('status').textContent).toBe('O agente está ocupado ou parado desde 14:05');
  });

  it('ignora o aviso do pedido que saiu de em andamento', () => {
    sidebar({ loop: { ...loop(at(9)), sem_nota_pedido: 'PED-009', sem_nota_desde: at(5) }, requests: [running({ estado: 'pendente', nota: undefined, nota_em: undefined })] });
    expect(screen.queryByRole('status')).toBeNull();
  });
});

describe('Sidebar e a entrada Perguntas', () => {
  it('mostra a entrada Perguntas com a contagem de respostas não lidas', () => {
    sidebar({ loop: null, requests: [], unreadAnswers: 2 });
    const link = screen.getByRole('link', { name: /Perguntas/ });
    expect(link.getAttribute('href')).toBe('/perguntas');
    expect(link.textContent).toContain('2');
  });

  it('esconde a contagem de Perguntas sem resposta não lida', () => {
    sidebar({ loop: null, requests: [] });
    const link = screen.getByRole('link', { name: /Perguntas/ });
    expect(link.textContent).toBe('Perguntas');
  });

  it('marca Perguntas como a página atual', () => {
    sidebar({ loop: null, requests: [], route: { page: 'ask' } });
    expect(screen.getByRole('link', { name: /Perguntas/ }).getAttribute('aria-current')).toBe('page');
  });
});
