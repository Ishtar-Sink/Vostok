import { afterEach, describe, expect, it, vi } from 'vitest';
import { acceptEvidence, formatDatesIn, formatDateTime, rejectEvidence, withdrawReview } from './api';

const now = new Date(2026, 10, 3, 18, 0);
const at = (day: number, hours: number, minutes: number) => new Date(2026, 10, day, hours, minutes).toISOString();

describe('formatDateTime', () => {
  it('mostra hoje e ontem com a hora local', () => {
    expect(formatDateTime(at(3, 14, 32), now)).toBe('hoje, 14:32');
    expect(formatDateTime(at(2, 9, 5), now)).toBe('ontem, 09:05');
  });

  it('mostra a data completa nos outros dias', () => {
    expect(formatDateTime(at(1, 8, 0), now)).toBe('01/11/2026, 08:00');
  });

  it('mostra só o dia quando o valor não tem hora', () => {
    expect(formatDateTime('2026-11-03', now)).toBe('hoje');
    expect(formatDateTime('2026-10-20', now)).toBe('20/10/2026');
  });

  it('mostra um traço quando não há data', () => {
    expect(formatDateTime('', now)).toBe('—');
  });
});

describe('formatDatesIn', () => {
  it('formata as datas ISO dentro de um texto', () => {
    const iso = at(3, 12, 46).replace(/\.\d{3}Z$/, 'Z');
    expect(formatDatesIn(`aprovada em ${iso}; modelo: sonnet`, now)).toBe('aprovada em hoje, 12:46; modelo: sonnet');
  });
});

describe('Chamadas de conferência', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    document.head.innerHTML = '';
  });

  function stubFetch(response: Response | Error) {
    const fake = vi.fn(() => (response instanceof Error ? Promise.reject(response) : Promise.resolve(response)));
    vi.stubGlobal('fetch', fake);
    return fake;
  }

  const route = (id: string, action: string) => `/api/tarefas/SPEC-051/TASK-01/evidencias/${id}/${action}`;
  const init = (body: string, token = 'abc123') => ({
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-Vostok-Token': token },
    body,
  });

  it('aceita com o token da página, a observação e o hash no corpo', async () => {
    document.head.innerHTML = '<meta name="vostok-token" content="abc123">';
    const fake = stubFetch(new Response('{}', { status: 200 }));
    await expect(acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '631f2c29', 'ok')).resolves.toEqual({ ok: true });
    expect(fake).toHaveBeenCalledWith(route('EV-02', 'aceitar'), init('{"texto":"ok","sha256":"631f2c29"}'));
  });

  it('aceita sem observação com o texto vazio', async () => {
    document.head.innerHTML = '<meta name="vostok-token" content="abc123">';
    const fake = stubFetch(new Response('{}', { status: 200 }));
    await acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '631f2c29');
    expect(fake).toHaveBeenCalledWith(route('EV-02', 'aceitar'), init('{"texto":"","sha256":"631f2c29"}'));
  });

  it('recusa e retira pelas rotas próprias com o texto', async () => {
    document.head.innerHTML = '<meta name="vostok-token" content="abc123">';
    const fake = stubFetch(new Response('{}', { status: 200 }));
    await rejectEvidence('SPEC-051', 'TASK-01', 'EV-03', '9a0c1d2e', 'falta o total');
    expect(fake).toHaveBeenCalledWith(route('EV-03', 'recusar'), init('{"texto":"falta o total","sha256":"9a0c1d2e"}'));
    await withdrawReview('SPEC-051', 'TASK-01', 'EV-01', '631f2c29', 'engano');
    expect(fake).toHaveBeenCalledWith(route('EV-01', 'retirar'), init('{"texto":"engano","sha256":"631f2c29"}'));
  });

  it('não manda hash vazio', async () => {
    document.head.innerHTML = '<meta name="vostok-token" content="abc123">';
    const fake = stubFetch(new Response('{}', { status: 200 }));
    await acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '', 'ok');
    expect(fake).toHaveBeenCalledWith(route('EV-02', 'aceitar'), init('{"texto":"ok"}'));
  });

  it('manda o token vazio sem a meta da página', async () => {
    const fake = stubFetch(new Response('{}', { status: 200 }));
    await acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '631f2c29', 'ok');
    expect(fake).toHaveBeenCalledWith(route('EV-02', 'aceitar'), init('{"texto":"ok","sha256":"631f2c29"}', ''));
  });

  it('devolve a mensagem de cada recusa do servidor', async () => {
    const messages: Record<number, string> = {
      400: 'Escreva o motivo.',
      403: 'O painel recusou a gravação: recarregue a página.',
      404: 'Evidência não encontrada.',
      409: 'A evidência mudou ou a tarefa não está em conferência: recarregue a página.',
    };
    for (const [status, message] of Object.entries(messages)) {
      stubFetch(new Response('erro', { status: Number(status) }));
      await expect(acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '631f2c29', 'ok')).resolves.toEqual({
        ok: false,
        status: Number(status),
        message,
      });
    }
  });

  it('avisa quando o painel não responde', async () => {
    stubFetch(new TypeError('rede'));
    await expect(acceptEvidence('SPEC-051', 'TASK-01', 'EV-02', '631f2c29', 'ok')).resolves.toEqual({
      ok: false,
      status: 0,
      message: 'Sem conexão com o painel.',
    });
  });
});
