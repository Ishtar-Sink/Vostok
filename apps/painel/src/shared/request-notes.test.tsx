import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { LiveVersion } from './action-bar';
import type { RequestDetail } from './api';
import { RequestNotes } from './request-notes';

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

const detail = (notas: RequestDetail['notas']): RequestDetail => ({ id: 'PED-001', acao: 'executar-tarefa', spec: 'SPEC-001', tarefa: 'TASK-01', estado: 'em_andamento', texto: 'Execute.', notas, resultado: '', lida_em: '' });
const note = (texto: string, detalhe = '', em = '2026-11-03T14:05:00Z') => ({ em, texto, detalhe });

describe('RequestNotes', () => {
  it('mostra as notas em ordem com o detalhe', async () => {
    mockFetch(() => json(detail([note('lendo a spec'), note('rodando os testes', 'Faltam dois testes.')])));
    render(<RequestNotes id="PED-001" onClose={() => {}} />);
    const items = await screen.findAllByRole('listitem');
    expect(items.map((item) => item.querySelector('p')?.textContent)).toEqual(['lendo a spec', 'rodando os testes']);
    expect(within(items[1]).getByText('Faltam dois testes.')).toBeTruthy();
    expect(within(items[0]).queryByText('Faltam dois testes.')).toBeNull();
    expect(screen.getByRole('dialog', { hidden: true }).textContent).toContain('Notas do pedido PED-001');
  });

  it('mostra o título ## Resultado do detalhe como texto, sem criar seção', async () => {
    mockFetch(() => json(detail([note('com título', 'antes\n## Resultado\ndepois')])));
    const { container } = render(<RequestNotes id="PED-001" onClose={() => {}} />);
    await screen.findByText('com título');
    expect(container.querySelector('pre')?.textContent).toBe('antes\n## Resultado\ndepois');
    expect(container.querySelector('h1, h2, h3, h4')?.textContent).toBe('Notas do pedido PED-001');
    expect(container.querySelectorAll('h2').length).toBeLessThanOrEqual(1);
  });

  it('mostra HTML da nota como texto', async () => {
    mockFetch(() => json(detail([note('<b>negrito</b>', '<img src=x onerror=alert(1)>')])));
    const { container } = render(<RequestNotes id="PED-001" onClose={() => {}} />);
    await screen.findByText('<b>negrito</b>');
    expect(container.querySelector('b')).toBeNull();
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('pre')?.textContent).toBe('<img src=x onerror=alert(1)>');
  });

  it('mostra o carregando antes da resposta', () => {
    mockFetch(() => new Promise<Response>(() => {}) as never);
    render(<RequestNotes id="PED-001" onClose={() => {}} />);
    expect(screen.getByText('Carregando as notas…')).toBeTruthy();
  });

  it('mostra o vazio sem notas', async () => {
    mockFetch(() => json(detail([])));
    render(<RequestNotes id="PED-001" onClose={() => {}} />);
    expect(await screen.findByText('O agente ainda não gravou notas neste pedido.')).toBeTruthy();
  });

  it('mostra o erro e tenta de novo', async () => {
    let calls = 0;
    mockFetch(() => (++calls === 1 ? json({}, 500) : json(detail([note('depois do erro')]))));
    render(<RequestNotes id="PED-001" onClose={() => {}} />);
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('Não foi possível ler as notas: Erro 500.');
    fireEvent.click(within(alert).getByRole('button', { name: 'Tentar de novo' }));
    expect(await screen.findByText('depois do erro')).toBeTruthy();
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('busca de novo com a versão ao vivo e mantém as notas e o foco', async () => {
    let current = [note('primeira')];
    const fetchSpy = mockFetch(() => json(detail(current)));
    const view = (version: number) => (
      <LiveVersion.Provider value={version}>
        <RequestNotes id="PED-001" onClose={() => {}} />
      </LiveVersion.Provider>
    );
    const { rerender } = render(view(1));
    await screen.findByText('primeira');
    const close = screen.getByRole('button', { name: 'Fechar', hidden: true });
    expect(document.activeElement).toBe(close);
    current = [note('primeira'), note('nota nova')];
    rerender(view(2));
    await waitFor(() => expect(screen.getByText('nota nova')).toBeTruthy());
    expect(screen.getByText('primeira')).toBeTruthy();
    expect(document.activeElement).toBe(close);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it('fecha pelo Fechar', async () => {
    mockFetch(() => json(detail([note('uma')])));
    const onClose = vi.fn();
    render(<RequestNotes id="PED-001" onClose={onClose} />);
    await screen.findByText('uma');
    fireEvent.click(screen.getByRole('button', { name: 'Fechar', hidden: true }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
