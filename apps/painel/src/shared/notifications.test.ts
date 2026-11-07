import { act, renderHook } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { OpenSpec, Overview, PanelRequest, RequestOutcome, TaskSummary } from './api';
import { noticeFor, returnNotice, useReturnNotices } from './notifications';

const view = (parts: Partial<Overview> = {}): Overview => ({ project: 'demo', specs: [], archived: [], activity: [], questions: [], requests: [], loop: null, ...parts });
const request = (overrides: Partial<PanelRequest> = {}): PanelRequest => ({ id: 'PED-001', acao: 'planejar-spec', spec: 'SPEC-001', tarefa: '', onda: '', estado: 'em_andamento', atualizado_em: '2026-11-03T10:00:00Z', ...overrides });
const outcome = (overrides: Partial<RequestOutcome> = {}): RequestOutcome => ({ id: 'PED-001', acao: 'planejar-spec', desfecho: 'concluido', pergunta: '', saidas: [], resultado: 'Plano criado.', ...overrides });
const spec = (lastOutcome: RequestOutcome | null, tasks: TaskSummary[] = []) => ({ meta: { id: 'SPEC-001', title: 'Exportar', status: 'approved' }, plan: false, plan_outdated: false, tasks, last_outcome: lastOutcome }) as unknown as OpenSpec;

class FakeNotification {
  static permission: NotificationPermission = 'default';
  static requestPermission = vi.fn(async () => {
    FakeNotification.permission = 'granted';
    return 'granted' as NotificationPermission;
  });
  static created: FakeNotification[] = [];
  onclick: (() => void) | null = null;
  constructor(public title: string, public options?: NotificationOptions) {
    FakeNotification.created.push(this);
  }
  close() {}
}

beforeEach(() => {
  FakeNotification.permission = 'default';
  FakeNotification.created = [];
  FakeNotification.requestPermission.mockClear();
  vi.stubGlobal('Notification', FakeNotification);
  vi.spyOn(document, 'hasFocus').mockReturnValue(false);
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

const hook = (first: Overview | null) => renderHook(({ overview }: { overview: Overview | null }) => useReturnNotices(overview), { initialProps: { overview: first } });

describe('useReturnNotices', () => {
  it('não avisa na primeira leitura da visão geral', () => {
    const { result } = hook(view({ requests: [request()] }));
    expect(result.current.notices).toEqual([]);
  });

  it('avisa quando um pedido concluído sai da fila', () => {
    const { result, rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome())] }) }));
    expect(result.current.notices).toHaveLength(1);
    const notice = result.current.notices[0];
    expect(notice.title).toBe('Pedido concluído');
    expect(notice.tone).toBe('success');
    expect(notice.href).toBe('/specs/SPEC-001');
    expect(notice.linkLabel).toBe('Ver a spec');
    expect(notice.message).toContain('PED-001');
  });

  it('leva à pergunta quando o pedido espera resposta', () => {
    const { result, rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome({ desfecho: 'aguardando-pergunta', pergunta: 'P-002' }))] }) }));
    const notice = result.current.notices[0];
    expect(notice.title).toBe('O agente tem uma pergunta');
    expect(notice.href).toBe('/pendencias');
    expect(notice.linkLabel).toBe('Responder');
    expect(notice.message).toContain('P-002');
  });

  it('leva à tarefa quando o pedido da tarefa é pausado', () => {
    const { result, rerender } = hook(view({ requests: [request({ tarefa: 'TASK-02' })] }));
    const task = { id: 'TASK-02', last_outcome: outcome({ desfecho: 'pausado', saidas: ['criar-spec'] }) } as unknown as TaskSummary;
    act(() => rerender({ overview: view({ specs: [spec(null, [task])] }) }));
    const notice = result.current.notices[0];
    expect(notice.title).toBe('Pedido pausado');
    expect(notice.tone).toBe('warning');
    expect(notice.href).toBe('/tarefas/SPEC-001/TASK-02');
  });

  it('avisa o pedido encerrado sem desfecho', () => {
    const { result, rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome({ id: 'PED-000' }))] }) }));
    const notice = result.current.notices[0];
    expect(notice.title).toBe('Pedido encerrado');
    expect(notice.href).toBe('/atividades');
  });

  it('não avisa o pedido que só mudou de estado', () => {
    const { result, rerender } = hook(view({ requests: [request({ estado: 'pendente' })] }));
    act(() => rerender({ overview: view({ requests: [request({ estado: 'em_andamento' })] }) }));
    expect(result.current.notices).toEqual([]);
  });

  it('notifica pelo navegador com a permissão concedida e a página sem foco', () => {
    FakeNotification.permission = 'granted';
    const { result, rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome())] }) }));
    expect(FakeNotification.created).toHaveLength(1);
    expect(FakeNotification.created[0].title).toBe('Pedido concluído');
    expect(FakeNotification.created[0].options?.tag).toBe('PED-001');
    expect(result.current.notices).toHaveLength(1);
  });

  it('não duplica a notificação com a página em foco', () => {
    FakeNotification.permission = 'granted';
    vi.spyOn(document, 'hasFocus').mockReturnValue(true);
    const { result, rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome())] }) }));
    expect(FakeNotification.created).toHaveLength(0);
    expect(result.current.notices).toHaveLength(1);
  });

  it('abre a pendência ao clicar na notificação', () => {
    FakeNotification.permission = 'granted';
    vi.spyOn(window, 'focus').mockImplementation(() => {});
    vi.spyOn(window, 'scrollTo').mockImplementation(() => {});
    const { rerender } = hook(view({ requests: [request()] }));
    act(() => rerender({ overview: view({ specs: [spec(outcome({ desfecho: 'aguardando-pergunta', pergunta: 'P-002' }))] }) }));
    FakeNotification.created[0].onclick!();
    expect(window.location.pathname).toBe('/pendencias');
  });

  it('só pede a permissão pelo botão', async () => {
    const { result } = hook(view({ requests: [request()] }));
    expect(result.current.askPermission).toBe(true);
    expect(FakeNotification.requestPermission).not.toHaveBeenCalled();
    await act(() => result.current.enableBrowser());
    expect(FakeNotification.requestPermission).toHaveBeenCalledTimes(1);
    expect(result.current.askPermission).toBe(false);
  });

  it('não convida com a permissão negada', () => {
    FakeNotification.permission = 'denied';
    const { result, rerender } = hook(view({ requests: [request()] }));
    expect(result.current.askPermission).toBe(false);
    act(() => rerender({ overview: view({ specs: [spec(outcome())] }) }));
    expect(result.current.notices).toHaveLength(1);
    expect(FakeNotification.created).toHaveLength(0);
  });

  it('deixa de convidar depois de agora não', () => {
    const { result } = hook(view({ requests: [request()] }));
    act(() => result.current.declineBrowser());
    expect(result.current.askPermission).toBe(false);
    expect(FakeNotification.requestPermission).not.toHaveBeenCalled();
  });

  it('avisa na tela sem a API de notificação do navegador', () => {
    vi.stubGlobal('Notification', undefined);
    const { result, rerender } = hook(view({ requests: [request()] }));
    expect(result.current.askPermission).toBe(false);
    act(() => rerender({ overview: view({ specs: [spec(outcome())] }) }));
    expect(result.current.notices).toHaveLength(1);
  });

  it('fecha o aviso e guarda só os três mais recentes', () => {
    const many = ['PED-001', 'PED-002', 'PED-003', 'PED-004'].map((id) => request({ id }));
    const { result, rerender } = hook(view({ requests: many }));
    act(() => rerender({ overview: view() }));
    expect(result.current.notices).toHaveLength(3);
    const first = result.current.notices[0].id;
    act(() => result.current.dismiss(first));
    expect(result.current.notices).toHaveLength(2);
    expect(result.current.notices.map((notice) => notice.id)).not.toContain(first);
  });
});

describe('tempo do aviso', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  const tick = (ms: number) => act(() => void vi.advanceTimersByTime(ms));
  const withNotice = (parts: Partial<Overview> = {}, outcomeParts: Partial<RequestOutcome> = {}) => {
    const rendered = hook(view({ requests: [request()] }));
    act(() => rendered.rerender({ overview: view({ ...parts, specs: [spec(outcome(outcomeParts))] }) }));
    return rendered;
  };

  it('some sozinho depois de 5 segundos sem o campo na visão', () => {
    const { result } = withNotice();
    expect(result.current.notices).toHaveLength(1);
    tick(4900);
    expect(result.current.notices).toHaveLength(1);
    tick(200);
    expect(result.current.notices).toEqual([]);
  });

  it('some depois de 10 segundos com o valor 10', () => {
    const { result } = withNotice({ toast_dismiss_seconds: 10 });
    tick(9900);
    expect(result.current.notices).toHaveLength(1);
    tick(200);
    expect(result.current.notices).toEqual([]);
  });

  it('nunca some com 0', () => {
    const { result } = withNotice({ toast_dismiss_seconds: 0 });
    tick(10 * 60 * 1000);
    expect(result.current.notices).toHaveLength(1);
    act(() => result.current.dismiss('PED-001'));
    expect(result.current.notices).toEqual([]);
  });

  it.each([
    ['success', { desfecho: 'concluido' as const }],
    ['warning', { desfecho: 'pausado' as const }],
    ['info', { id: 'PED-000' }],
  ])('some sozinho no tom %s', (tone, outcomeParts) => {
    const { result } = withNotice({}, outcomeParts);
    expect(result.current.notices[0].tone).toBe(tone);
    tick(5100);
    expect(result.current.notices).toEqual([]);
  });

  it('para com o ponteiro na região e recomeça inteiro ao sair', () => {
    const { result } = withNotice();
    tick(3000);
    act(() => result.current.regionProps.onMouseEnter());
    tick(60000);
    expect(result.current.notices).toHaveLength(1);
    act(() => result.current.regionProps.onMouseLeave());
    tick(4900);
    expect(result.current.notices).toHaveLength(1);
    tick(200);
    expect(result.current.notices).toEqual([]);
  });

  it('para com o foco na região e recomeça inteiro ao sair', () => {
    const { result } = withNotice();
    tick(3000);
    act(() => result.current.regionProps.onFocus());
    tick(60000);
    expect(result.current.notices).toHaveLength(1);
    act(() => result.current.regionProps.onBlur());
    tick(4900);
    expect(result.current.notices).toHaveLength(1);
    tick(200);
    expect(result.current.notices).toEqual([]);
  });

  it('um aviso novo não reinicia o tempo do anterior', () => {
    const rendered = hook(view({ requests: [request({ id: 'PED-001' }), request({ id: 'PED-002' })] }));
    act(() => rendered.rerender({ overview: view({ requests: [request({ id: 'PED-002' })] }) }));
    tick(3000);
    act(() => rendered.rerender({ overview: view() }));
    expect(rendered.result.current.notices.map((notice) => notice.id)).toEqual(['PED-002', 'PED-001']);
    tick(2100);
    expect(rendered.result.current.notices.map((notice) => notice.id)).toEqual(['PED-002']);
    tick(3000);
    expect(rendered.result.current.notices).toEqual([]);
  });

  it('fechar ou sumir não chama a rede', () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal('fetch', fetchSpy);
    const rendered = hook(view({ requests: [request({ id: 'PED-001' }), request({ id: 'PED-002' })] }));
    act(() => rendered.rerender({ overview: view() }));
    act(() => rendered.result.current.dismiss('PED-001'));
    tick(6000);
    expect(rendered.result.current.notices).toEqual([]);
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('o que não coube nos três não volta', () => {
    const many = ['PED-001', 'PED-002', 'PED-003', 'PED-004'].map((id) => request({ id }));
    const rendered = hook(view({ requests: many }));
    act(() => rendered.rerender({ overview: view() }));
    expect(rendered.result.current.notices).toHaveLength(3);
    act(() => rendered.result.current.dismiss(rendered.result.current.notices[0].id));
    tick(6000);
    expect(rendered.result.current.notices).toEqual([]);
    act(() => rendered.rerender({ overview: view({ activity: [] }) }));
    expect(rendered.result.current.notices).toEqual([]);
  });

  it('na aba em segundo plano, some e notifica pelo navegador', () => {
    FakeNotification.permission = 'granted';
    const { result } = withNotice();
    expect(FakeNotification.created).toHaveLength(1);
    tick(5100);
    expect(result.current.notices).toEqual([]);
    expect(FakeNotification.created).toHaveLength(1);
  });

  it('o convite continua depois do tempo', () => {
    const { result } = withNotice({ requests: [request({ id: 'PED-009' })] });
    expect(result.current.askPermission).toBe(true);
    tick(60000);
    expect(result.current.notices).toEqual([]);
    expect(result.current.askPermission).toBe(true);
  });

  it('seguir o link fecha o aviso', () => {
    const { result } = withNotice();
    const event = { preventDefault: vi.fn(), defaultPrevented: false, button: 0, metaKey: false, ctrlKey: false, shiftKey: false, altKey: false, currentTarget: { getAttribute: () => '/specs/SPEC-001', target: '' } } as unknown as Parameters<ReturnType<typeof useReturnNotices>["followNotice"]>[0];
    vi.spyOn(window, 'scrollTo').mockImplementation(() => {});
    act(() => result.current.followNotice(event, result.current.notices[0]));
    expect(result.current.notices).toEqual([]);
  });
});

describe('noticeFor', () => {
  it.each([
    ['concluido', ''],
    ['pausado', ''],
    ['aguardando-pergunta', 'P-002'],
    ['', ''],
  ])('noticeFor dá o mesmo texto do aviso, em cada desfecho %s', (desfecho, pergunta) => {
    const overview = view({ specs: [spec(desfecho ? outcome({ desfecho: desfecho as RequestOutcome['desfecho'], pergunta }) : null)] });
    const fromRequest = returnNotice(request(), overview);
    const fromSource = noticeFor({ id: 'PED-001', acao: 'planejar-spec', spec: 'SPEC-001', tarefa: '', desfecho, pergunta });
    expect(fromSource).toEqual(fromRequest);
  });
});

describe('noticeFor e a pergunta livre', () => {
  const base = { id: 'PED-021', acao: 'perguntar-agente', tarefa: '', desfecho: 'concluido', pergunta: '' };

  it('leva a perguntas com spec', () => {
    const notice = noticeFor({ ...base, spec: 'SPEC-002' });
    expect(notice.href).toBe('/perguntas');
    expect(notice.linkLabel).toBe('Ver a resposta');
    expect(notice.message).toContain('PED-021 (SPEC-002) terminou');
    expect(notice.title).toBe('Pedido concluído');
    expect(notice.tone).toBe('success');
  });

  it('leva a perguntas sem spec e não cita spec vazia', () => {
    const notice = noticeFor({ ...base, spec: '' });
    expect(notice.href).toBe('/perguntas');
    expect(notice.linkLabel).toBe('Ver a resposta');
    expect(notice.message).toContain('PED-021 terminou');
    expect(notice.message).not.toContain('()');
  });

  it('noticeFor mantém o texto das demais ações', () => {
    const notice = noticeFor({ id: 'PED-001', acao: 'executar-tarefa', spec: 'SPEC-002', tarefa: 'TASK-01', desfecho: 'pausado', pergunta: '' });
    expect(notice.message).toBe('PED-001 (SPEC-002 · TASK-01) parou e espera a sua decisão.');
    expect(notice.href).toBe('/tarefas/SPEC-002/TASK-01');
    expect(notice.linkLabel).toBe('Ver o motivo');
  });
});

describe('seguir o link do aviso', () => {
  const event = (href: string) => ({ preventDefault: vi.fn(), defaultPrevented: false, button: 0, metaKey: false, ctrlKey: false, shiftKey: false, altKey: false, currentTarget: { getAttribute: () => href, target: '' } }) as unknown as Parameters<ReturnType<typeof useReturnNotices>['followNotice']>[0];

  it('seguir o link do aviso marca como lida', () => {
    document.head.insertAdjacentHTML('beforeend', '<meta name="painel-token" content="tok-1">');
    const fetchSpy = vi.fn(() => Promise.resolve(new Response('{"marcadas":1}')));
    vi.stubGlobal('fetch', fetchSpy);
    vi.spyOn(window, 'scrollTo').mockImplementation(() => {});
    const rendered = hook(view({ requests: [request({ id: 'PED-001', spec: 'SPEC-001', tarefa: 'TASK-02' })] }));
    act(() => rendered.rerender({ overview: view() }));
    act(() => rendered.result.current.followNotice(event('/tarefas/SPEC-001/TASK-02'), rendered.result.current.notices[0]));
    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const [url, init] = fetchSpy.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/notificacoes/lidas?ids=PED-001');
    expect(init.method).toBe('POST');
    expect((init.headers as Record<string, string>)['X-Painel-Token']).toBe('tok-1');
    expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-02');
    expect(rendered.result.current.notices).toEqual([]);
    document.head.querySelector('meta[name="painel-token"]')?.remove();
  });

  it('fechar o aviso continua sem chamar a rede', () => {
    const fetchSpy = vi.fn();
    vi.stubGlobal('fetch', fetchSpy);
    const rendered = hook(view({ requests: [request({ id: 'PED-001' })] }));
    act(() => rendered.rerender({ overview: view() }));
    act(() => rendered.result.current.dismiss('PED-001'));
    expect(rendered.result.current.notices).toEqual([]);
    expect(fetchSpy).not.toHaveBeenCalled();
  });
});
