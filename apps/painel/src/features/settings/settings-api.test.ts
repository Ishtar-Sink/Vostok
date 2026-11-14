import { afterEach, describe, expect, it, vi } from 'vitest';
import { saveEvidenceSettings } from '../../shared/api';

describe('saveEvidenceSettings', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    document.head.innerHTML = '';
  });

  function stubFetch(response: Response | Error) {
    const fake = vi.fn(() => (response instanceof Error ? Promise.reject(response) : Promise.resolve(response)));
    vi.stubGlobal('fetch', fake);
    return fake;
  }

  const json = (body: unknown, status: number) => new Response(JSON.stringify(body), { status });

  it('salva com o token da página, os valores e o sha256 lidos', async () => {
    document.head.innerHTML = '<meta name="vostok-token" content="abc123">';
    const body = { arquivos: { 'tasks.yml': 'novo' }, valores: { 'tasks.evidence_title_max': 80 } };
    const fake = stubFetch(json(body, 200));
    await expect(saveEvidenceSettings({ 'tasks.evidence_title_max': 80 }, { 'tasks.yml': 'lido' })).resolves.toEqual({
      ok: true,
      data: body,
    });
    expect(fake).toHaveBeenCalledWith('/api/configuracoes/evidencias', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', 'X-Vostok-Token': 'abc123' },
      body: '{"valores":{"tasks.evidence_title_max":80},"arquivos":{"tasks.yml":"lido"}}',
    });
  });

  it('devolve os erros por chave no 400', async () => {
    const erros = { 'tasks.evidence_title_max': 'Use um número de 20 a 200.' };
    stubFetch(json({ erros }, 400));
    await expect(saveEvidenceSettings({}, {})).resolves.toEqual({
      ok: false,
      status: 400,
      erros,
      message: 'Corrija os campos marcados.',
    });
  });

  it('devolve o arquivo que mudou no 409', async () => {
    stubFetch(json({ arquivo: 'tasks.yml' }, 409));
    const result = await saveEvidenceSettings({}, {});
    expect(result).toMatchObject({ ok: false, status: 409, arquivo: 'tasks.yml' });
    expect(result.ok ? '' : result.message).toContain('tasks.yml');
    expect(result.ok ? '' : result.message).toContain('Recarregue');
  });

  it('explica o 403 do painel aberto sem o token', async () => {
    stubFetch(new Response('', { status: 403 }));
    await expect(saveEvidenceSettings({}, {})).resolves.toMatchObject({
      ok: false,
      status: 403,
      message: 'O painel recusou a gravação: abra o painel pelo vostok painel e recarregue a página.',
    });
  });

  it('avisa quando o painel não responde', async () => {
    stubFetch(new Error('offline'));
    await expect(saveEvidenceSettings({}, {})).resolves.toEqual({ ok: false, status: 0, message: 'Sem conexão com o painel.' });
  });
});
