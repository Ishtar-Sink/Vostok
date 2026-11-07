import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import type { PanelRequest, RequestOutcome } from './api';
import { RequestState } from './request-state';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
function mockFetch(handler: (url: string, init?: RequestInit) => Response | Promise<Response>) {
  const fn = vi.fn((url: string, init?: RequestInit) => Promise.resolve(handler(url, init)));
  vi.stubGlobal('fetch', fn);
  return fn;
}
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

const request = (over: object = {}) => ({ id: 'PED-3', acao: 'planejar-spec', spec: 'SPEC-001', tarefa: '', estado: 'pendente', atualizado_em: '2026-11-03T10:00:00Z', ...over }) as PanelRequest;
const outcome = (over: object = {}) => ({ id: 'PED-6', acao: 'aprovar-plano', desfecho: 'pausado', pergunta: '', saidas: [], resultado: 'Falta a spec da lista de tarefas.', ...over }) as RequestOutcome;
const region = () => screen.getByRole('region', { name: 'Estado do pedido' });

describe('RequestState', () => {
  it('mostra o pedido na fila com a contagem', () => {
    render(<RequestState spec="SPEC-001" request={request()} outcome={null} queued={2} />);
    const text = region().textContent ?? '';
    expect(text).toContain('Aguardando o agente');
    expect(text).toContain('PED-3');
    expect(text).toContain('2 pedidos na fila');
  });

  it('mostra o verbo do pedido em andamento', () => {
    render(<RequestState spec="SPEC-001" request={request({ estado: 'em_andamento' })} outcome={null} queued={1} />);
    const text = region().textContent ?? '';
    expect(text).toContain('Planejando');
    expect(text).not.toContain('pedidos na fila');
  });

  it('mostra a pergunta que espera a resposta', () => {
    render(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ desfecho: 'aguardando-pergunta', pergunta: 'P-002', resultado: 'O agente precisa da stack.' })} />);
    const text = region().textContent ?? '';
    expect(text).toContain('Aguardando sua resposta à P-002');
    expect(text).toContain('O agente precisa da stack.');
    expect(within(region()).getByRole('link', { name: 'Responder em Pendências' }).getAttribute('href')).toBe('/pendencias');
  });

  it('mostra o motivo e as saídas da pausa', () => {
    render(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ saidas: ['criar-spec', 'acao-removida'] })} />);
    const text = region().textContent ?? '';
    expect(text).toContain('Pedido pausado');
    expect(text).toContain('Falta a spec da lista de tarefas.');
    const buttons = within(region()).getAllByRole('button');
    expect(buttons).toHaveLength(1);
    expect(buttons[0].textContent).toBe('Nova spec');
  });

  it('mantém o motivo sem saída válida', () => {
    render(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ saidas: ['acao-removida', 'executar-tarefa'] })} />);
    expect(region().textContent).toContain('Falta a spec da lista de tarefas.');
    expect(within(region()).queryAllByRole('button')).toHaveLength(0);
  });

  it('pede a saída da pausa', async () => {
    const fetchMock = mockFetch(() => json({ id: 'PED-9' }));
    render(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ saidas: ['criar-spec'] })} />);
    fireEvent.click(within(region()).getByRole('button', { name: 'Nova spec' }));
    fireEvent.input(screen.getByRole('textbox', { name: 'Assunto da spec' }), { target: { value: 'Lista de tarefas com filtros' } });
    fireEvent.click(screen.getAllByRole('button', { name: 'Nova spec' }).at(-1) as HTMLElement);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/pedidos?acao=criar-spec', expect.objectContaining({ method: 'POST', body: 'Lista de tarefas com filtros' })));
  });

  it('prefere o pedido ativo ao desfecho anterior', () => {
    render(<RequestState spec="SPEC-001" request={request({ estado: 'em_andamento' })} outcome={outcome()} />);
    const text = region().textContent ?? '';
    expect(text).toContain('Planejando');
    expect(text).not.toContain('Pedido pausado');
  });

  it('não mostra nada sem pedido nem parada', () => {
    const { rerender } = render(<RequestState spec="SPEC-001" request={undefined} outcome={null} />);
    expect(screen.queryByRole('region', { name: 'Estado do pedido' })).toBeNull();
    rerender(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ desfecho: 'concluido' })} />);
    expect(screen.queryByRole('region', { name: 'Estado do pedido' })).toBeNull();
  });
});

describe('RequestState em modo não confiável', () => {
  const attack = '<img src="x" onerror="y">\n\n[clique](javascript:z)\n\n![foto](http://fora.invalid/a.png)';

  it.each([
    { nome: 'pausado', desfecho: { desfecho: 'pausado' } },
    { nome: 'aguardando pergunta', desfecho: { desfecho: 'aguardando-pergunta', pergunta: 'P-002' } },
  ])('mostra como texto o resultado $nome', ({ desfecho }) => {
    render(<RequestState spec="SPEC-001" request={undefined} outcome={outcome({ ...desfecho, resultado: attack })} />);
    const text = region().textContent ?? '';
    expect(text).toContain('<img src="x" onerror="y">');
    expect(text).toContain('[clique](javascript:z)');
    expect(text).toContain('foto');
    expect(region().querySelector('img')).toBeNull();
    expect(region().querySelector('a[href^="javascript"]')).toBeNull();
  });
});
