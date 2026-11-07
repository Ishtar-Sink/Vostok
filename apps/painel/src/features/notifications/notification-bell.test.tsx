import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { NotificationItem, NotificationList } from '../../shared/api';
import { NotificationBell } from './notification-bell';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
function mockFetch(handler: (url: string) => Response | Promise<Response>) {
  const fn = vi.fn((url: string) => Promise.resolve(handler(url)));
  vi.stubGlobal('fetch', fn);
  return fn;
}
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const item = (id: string, over: Partial<NotificationItem> = {}): NotificationItem => ({ id, acao: 'executar-tarefa', spec: 'SPEC-002', tarefa: 'TASK-01', estado: 'feito', desfecho: 'concluido', pergunta: '', atualizado_em: '2026-11-03T15:10:00Z', lida: false, ...over });
const list = (itens: NotificationItem[], unread = itens.filter((entry) => !entry.lida).length): NotificationList => ({ nao_lidas: unread, itens });
const bell = () => screen.getByRole('button', { name: /^Notificações/ });
const open = async () => {
  fireEvent.click(bell());
  return screen.findByRole('dialog', { name: 'Notificações' });
};

describe('NotificationBell e a contagem', () => {
  it('não mostra contagem com zero', () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={0} version={0} />);
    expect(bell().getAttribute('aria-label')).toBe('Notificações: nenhuma não lida');
    expect(bell().textContent?.trim()).toBe('');
  });

  it('mostra 3', () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={3} version={0} />);
    expect(bell().textContent?.trim()).toBe('3');
    expect(bell().getAttribute('aria-label')).toBe('Notificações: 3 não lidas');
  });

  it('diz 1 não lida no singular', () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={1} version={0} />);
    expect(bell().getAttribute('aria-label')).toBe('Notificações: 1 não lida');
  });

  it('mostra 99+ com 120 e o rótulo diz 120', () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={120} version={0} />);
    expect(bell().textContent?.trim()).toBe('99+');
    expect(bell().getAttribute('aria-label')).toBe('Notificações: 120 não lidas');
  });

  it('mostra 99 inteiro, sem o mais', () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={99} version={0} />);
    expect(bell().textContent?.trim()).toBe('99');
  });

  it('atualiza a contagem quando a visão muda', () => {
    mockFetch(() => json(list([])));
    const view = render(<NotificationBell unread={4} version={1} />);
    expect(bell().textContent?.trim()).toBe('4');
    view.rerender(<NotificationBell unread={3} version={2} />);
    expect(bell().textContent?.trim()).toBe('3');
    expect(bell().getAttribute('aria-label')).toBe('Notificações: 3 não lidas');
  });
});

describe('NotificationBell e a seção', () => {
  it('abre a seção e lista por ordem, com título, mensagem, hora e link', async () => {
    mockFetch(() => json(list([item('PED-014', { estado: 'recusado', desfecho: '' }), item('PED-013', { desfecho: 'aguardando-pergunta', pergunta: 'P-001' }), item('PED-011')])));
    render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    expect(bell().getAttribute('aria-expanded')).toBe('true');
    const entries = await within(dialog).findAllByRole('listitem');
    expect(entries.map((entry) => entry.getAttribute('data-notification-id'))).toEqual(['PED-014', 'PED-013', 'PED-011']);
    const question = entries[1];
    expect(within(question).getByText('O agente tem uma pergunta')).toBeTruthy();
    expect(question.textContent).toContain('PED-013 (SPEC-002 · TASK-01) espera a sua resposta à P-001.');
    expect(question.querySelector('time')?.textContent).toMatch(/^03\/11\/2026, \d{2}:\d{2}$/);
    expect(within(question).getByRole('link', { name: 'Responder' }).getAttribute('href')).toBe('/pendencias');
    expect(within(entries[2]).getByRole('link', { name: 'Ver a tarefa' }).getAttribute('href')).toBe('/tarefas/SPEC-002/TASK-01');
    expect(within(entries[0]).getByText('Pedido encerrado')).toBeTruthy();
  });

  it('separa lidas de não lidas', async () => {
    mockFetch(() => json(list([item('PED-012'), item('PED-011', { lida: true })])));
    render(<NotificationBell unread={1} version={0} />);
    const dialog = await open();
    const unreadGroup = await within(dialog).findByRole('heading', { name: 'Não lidas' });
    const readGroup = within(dialog).getByRole('heading', { name: 'Lidas' });
    expect(unreadGroup.parentElement?.querySelector('[data-notification-id]')?.getAttribute('data-notification-id')).toBe('PED-012');
    expect(readGroup.parentElement?.querySelector('[data-notification-id]')?.getAttribute('data-notification-id')).toBe('PED-011');
  });

  it('não mostra o grupo vazio', async () => {
    mockFetch(() => json(list([item('PED-011', { lida: true })])));
    render(<NotificationBell unread={0} version={0} />);
    const dialog = await open();
    await within(dialog).findByRole('heading', { name: 'Lidas' });
    expect(within(dialog).queryByRole('heading', { name: 'Não lidas' })).toBeNull();
  });

  it('mostra carregando', async () => {
    mockFetch(() => new Promise<Response>(() => {}));
    render(<NotificationBell unread={0} version={0} />);
    const dialog = await open();
    expect(within(dialog).getByRole('status').textContent).toContain('Carregando as notificações');
  });

  it('mostra o erro com Tentar de novo', async () => {
    let calls = 0;
    mockFetch(() => (++calls === 1 ? json({}, 500) : json(list([item('PED-011')]))));
    render(<NotificationBell unread={1} version={0} />);
    const dialog = await open();
    expect(await within(dialog).findByRole('alert')).toBeTruthy();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Tentar de novo' }));
    expect(await within(dialog).findByText('Pedido concluído')).toBeTruthy();
  });

  it('mostra o estado vazio', async () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={0} version={0} />);
    const dialog = await open();
    expect(await within(dialog).findByText('Nenhuma notificação')).toBeTruthy();
  });

  it('o aviso de pergunta já respondida continua com o link das Pendências', async () => {
    mockFetch(() => json(list([item('PED-013', { desfecho: 'aguardando-pergunta', pergunta: 'P-001', lida: true })])));
    render(<NotificationBell unread={0} version={0} />);
    const dialog = await open();
    const link = await within(dialog).findByRole('link', { name: 'Responder' });
    expect(link.getAttribute('href')).toBe('/pendencias');
  });
});

describe('NotificationBell e o teclado', () => {
  it('Esc fecha e devolve o foco ao sino', async () => {
    mockFetch(() => json(list([item('PED-011')])));
    render(<NotificationBell unread={1} version={0} />);
    const dialog = await open();
    await waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(document.activeElement).toBe(bell());
    expect(bell().getAttribute('aria-expanded')).toBe('false');
  });

  it('clique fora fecha', async () => {
    mockFetch(() => json(list([item('PED-011')])));
    render(<div><span data-testid="fora">fora</span><NotificationBell unread={1} version={0} /></div>);
    const dialog = await open();
    fireEvent.click(within(dialog).getByRole('heading', { name: 'Notificações' }));
    expect(screen.queryByRole('dialog')).not.toBeNull();
    fireEvent.click(screen.getByTestId('fora'));
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('o sino abre e fecha a seção', async () => {
    mockFetch(() => json(list([])));
    render(<NotificationBell unread={0} version={0} />);
    await open();
    fireEvent.click(bell());
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('recarregar a lista mantém o foco no item', async () => {
    mockFetch(() => json(list([item('PED-012'), item('PED-011')])));
    const view = render(<NotificationBell unread={2} version={0} />);
    const dialog = await open();
    const link = (await within(dialog).findAllByRole('link'))[1];
    link.focus();
    view.rerender(<NotificationBell unread={2} version={1} />);
    await act(async () => {
      await Promise.resolve();
    });
    expect(document.activeElement).toBe(link);
    expect(dialog.contains(link)).toBe(true);
  });

  it('segue o link e fecha a seção', async () => {
    mockFetch(() => json(list([item('PED-011')])));
    render(<NotificationBell unread={1} version={0} />);
    const dialog = await open();
    fireEvent.click(await within(dialog).findByRole('link', { name: 'Ver a tarefa' }));
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(window.location.pathname).toBe('/tarefas/SPEC-002/TASK-01');
  });
});

describe('NotificationBell e marcar como lida', () => {
  const unreadThree = () => [item('PED-013', { desfecho: 'aguardando-pergunta', pergunta: 'P-001' }), item('PED-012', { desfecho: 'pausado' }), item('PED-011'), item('PED-010', { lida: true })];
  const marks = (fn: ReturnType<typeof mockFetch>) => fn.mock.calls.map((call) => String(call[0])).filter((url) => url.includes('/lidas'));
  const box = (dialog: HTMLElement, id: string) => dialog.querySelector<HTMLInputElement>(`[data-notification-id=${id}] input[type=checkbox]`) as HTMLInputElement;
  const action = (dialog: HTMLElement, name: string) => within(dialog).getByRole('button', { name });

  it('só não lidas têm caixa de seleção', async () => {
    mockFetch(() => json(list(unreadThree())));
    render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    await within(dialog).findByText('Pedido pausado');
    expect(within(dialog).getAllByRole('checkbox')).toHaveLength(3);
    expect(within(dialog).getByRole('checkbox', { name: 'Selecionar PED-012' })).toBeTruthy();
    expect(dialog.querySelector('[data-notification-id=PED-010] input')).toBeNull();
  });

  it('Marcar selecionadas fica desabilitado sem seleção', async () => {
    mockFetch(() => json(list(unreadThree())));
    render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    await within(dialog).findByText('Pedido pausado');
    expect((action(dialog, 'Marcar selecionadas') as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(box(dialog, 'PED-012'));
    expect((action(dialog, 'Marcar selecionadas') as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(box(dialog, 'PED-012'));
    expect((action(dialog, 'Marcar selecionadas') as HTMLButtonElement).disabled).toBe(true);
  });

  it('marca duas juntas com ids', async () => {
    const fn = mockFetch((url) => (url.includes('/lidas') ? json({ marcadas: 2 }) : json(list(unreadThree()))));
    render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    await within(dialog).findByText('Pedido pausado');
    fireEvent.click(box(dialog, 'PED-012'));
    fireEvent.click(box(dialog, 'PED-013'));
    fireEvent.click(action(dialog, 'Marcar selecionadas'));
    await waitFor(() => expect(marks(fn)).toHaveLength(1));
    expect(marks(fn)[0]).toBe('/api/notificacoes/lidas?ids=PED-013,PED-012');
    const init = (fn.mock.calls as unknown as [string, RequestInit][]).find((call) => call[0].includes('/lidas'))?.[1];
    expect(init?.method).toBe('POST');
  });

  it('Marcar todas envia todas=sim e fica desabilitado sem não lidas', async () => {
    const fn = mockFetch((url) => (url.includes('/lidas') ? json({ marcadas: 3 }) : json(list(unreadThree()))));
    const view = render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    await within(dialog).findByText('Pedido pausado');
    fireEvent.click(action(dialog, 'Marcar todas como lidas'));
    await waitFor(() => expect(marks(fn)).toEqual(['/api/notificacoes/lidas?todas=sim']));
    view.unmount();
    mockFetch(() => json(list([item('PED-010', { lida: true })])));
    render(<NotificationBell unread={0} version={0} />);
    const again = await open();
    await within(again).findByRole('heading', { name: 'Lidas' });
    expect((action(again, 'Marcar todas como lidas') as HTMLButtonElement).disabled).toBe(true);
    expect((action(again, 'Marcar selecionadas') as HTMLButtonElement).disabled).toBe(true);
  });

  it('com erro, mostra o motivo, mantém a seleção e não muda a contagem', async () => {
    const fn = mockFetch((url) => (url.includes('/lidas') ? new Response('disco cheio', { status: 500 }) : json(list(unreadThree()))));
    render(<NotificationBell unread={3} version={0} />);
    const dialog = await open();
    await within(dialog).findByText('Pedido pausado');
    fireEvent.click(box(dialog, 'PED-012'));
    fireEvent.click(box(dialog, 'PED-013'));
    fireEvent.click(action(dialog, 'Marcar selecionadas'));
    const alert = await within(dialog).findByRole('alert');
    expect(alert.textContent).toBe('Não foi possível marcar: disco cheio');
    expect(dialog.querySelectorAll('input[type=checkbox]:checked')).toHaveLength(2);
    expect(bell().getAttribute('aria-label')).toBe('Notificações: 3 não lidas');
    expect(marks(fn)).toHaveLength(1);
  });

  it('depois de marcar, limpa a seleção e o foco fica na seção', async () => {
    let done = false;
    mockFetch((url) => {
      if (url.includes('/lidas')) {
        done = true;
        return json({ marcadas: 1 });
      }
      return json(list(done ? [item('PED-012', { lida: true }), item('PED-011')] : [item('PED-012'), item('PED-011')]));
    });
    render(<NotificationBell unread={2} version={0} />);
    const dialog = await open();
    await within(dialog).findAllByRole('checkbox');
    fireEvent.click(box(dialog, 'PED-012'));
    box(dialog, 'PED-012').focus();
    fireEvent.click(action(dialog, 'Marcar selecionadas'));
    await waitFor(() => expect(dialog.querySelector('[data-notification-id=PED-012] input')).toBeNull());
    expect(dialog.querySelectorAll('input[type=checkbox]:checked')).toHaveLength(0);
    expect(within(dialog).queryByRole('alert')).toBeNull();
    expect(document.activeElement).toBe(within(dialog).getByRole('heading', { name: 'Notificações' }));
  });

  it('recarregar não perde a seleção de quem continua não lida', async () => {
    let second = false;
    mockFetch(() => json(list(second ? [item('PED-012'), item('PED-011', { lida: true })] : [item('PED-012'), item('PED-011')])));
    const view = render(<NotificationBell unread={2} version={0} />);
    const dialog = await open();
    await within(dialog).findAllByRole('checkbox');
    fireEvent.click(box(dialog, 'PED-012'));
    fireEvent.click(box(dialog, 'PED-011'));
    second = true;
    view.rerender(<NotificationBell unread={1} version={1} />);
    await waitFor(() => expect(dialog.querySelector('[data-notification-id=PED-011] input')).toBeNull());
    expect(box(dialog, 'PED-012').checked).toBe(true);
    expect((action(dialog, 'Marcar selecionadas') as HTMLButtonElement).disabled).toBe(false);
  });

  it('seguir o link marca como lida e fecha a seção', async () => {
    const fn = mockFetch((url) => (url.includes('/lidas') ? json({ marcadas: 1 }) : json(list([item('PED-011'), item('PED-010', { lida: true })]))));
    render(<NotificationBell unread={1} version={0} />);
    const dialog = await open();
    const links = await within(dialog).findAllByRole('link', { name: 'Ver a tarefa' });
    fireEvent.click(links[1]);
    expect(marks(fn)).toEqual([]);
    expect(screen.queryByRole('dialog')).toBeNull();
    fireEvent.click(bell());
    const reopened = await screen.findByRole('dialog', { name: 'Notificações' });
    fireEvent.click((await within(reopened).findAllByRole('link', { name: 'Ver a tarefa' }))[0]);
    expect(marks(fn)).toEqual(['/api/notificacoes/lidas?ids=PED-011']);
    expect(screen.queryByRole('dialog')).toBeNull();
  });
});
