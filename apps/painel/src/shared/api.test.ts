import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { formatDatesIn, formatDateTime, uploadAttachment } from './api';

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

class FakeXhr {
  static last: FakeXhr;
  method = '';
  url = '';
  headers: Record<string, string> = {};
  body: unknown;
  status = 0;
  responseText = '';
  listeners: Record<string, (event: unknown) => void> = {};
  upload = {
    listeners: {} as Record<string, (event: unknown) => void>,
    addEventListener(type: string, listener: (event: unknown) => void) {
      this.listeners[type] = listener;
    },
  };
  constructor() {
    FakeXhr.last = this;
  }
  open(method: string, url: string) {
    this.method = method;
    this.url = url;
  }
  setRequestHeader(name: string, value: string) {
    this.headers[name] = value;
  }
  addEventListener(type: string, listener: (event: unknown) => void) {
    this.listeners[type] = listener;
  }
  send(body: unknown) {
    this.body = body;
  }
  answer(status: number, text: string) {
    this.status = status;
    this.responseText = text;
    this.listeners.load?.({});
  }
}

describe('uploadAttachment', () => {
  beforeEach(() => {
    document.head.innerHTML = '<meta name="painel-token" content="tok-1">';
    vi.stubGlobal('XMLHttpRequest', FakeXhr);
  });
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('envia com o token e o nome codificado', async () => {
    const file = new File(['conteúdo'], 'relatório final.pdf');
    const done = uploadAttachment('r1', file, 'relatório final.pdf', () => undefined);
    const xhr = FakeXhr.last;
    expect(xhr.method).toBe('PUT');
    expect(xhr.url).toBe('/api/pedidos/anexos?rascunho=r1&nome=relat%C3%B3rio%20final.pdf');
    expect(xhr.headers['X-Painel-Token']).toBe('tok-1');
    expect(xhr.body).toBe(file);
    xhr.answer(200, '{}');
    expect(await done).toEqual({ status: 200, ok: true, text: '{}' });
  });

  it('reporta o progresso em porcentagem', () => {
    const seen: number[] = [];
    void uploadAttachment('r1', new File(['x'], 'a.txt'), 'a.txt', (percent) => seen.push(percent));
    const progress = FakeXhr.last.upload.listeners.progress;
    progress?.({ lengthComputable: true, loaded: 1, total: 3 });
    progress?.({ lengthComputable: true, loaded: 3, total: 3 });
    progress?.({ lengthComputable: false, loaded: 0, total: 0 });
    expect(seen).toEqual([33, 100]);
  });

  it('devolve o status e o texto da recusa', async () => {
    const done = uploadAttachment('r1', new File(['x'], 'a.txt'), 'a.txt', () => undefined);
    FakeXhr.last.answer(413, 'Anexo grande demais.');
    expect(await done).toEqual({ status: 413, ok: false, text: 'Anexo grande demais.' });
  });

  it('devolve status 0 na queda da conexão', async () => {
    const done = uploadAttachment('r1', new File(['x'], 'a.txt'), 'a.txt', () => undefined);
    FakeXhr.last.listeners.error?.({});
    expect(await done).toEqual({ status: 0, ok: false, text: 'falha de rede' });
  });
});
