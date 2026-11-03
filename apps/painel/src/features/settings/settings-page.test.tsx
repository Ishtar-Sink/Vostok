import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { SettingsPage } from './settings-page';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const MAIN = '# Geral.\n\nharness:\n  tool: claude-code\n  repository: dono/repo\n\nextra_option: 5\n';
const file = { name: 'main.yml', path: 'vostok/config/main.yml', content: MAIN, hash: 'h1' };
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

function stub(handler: (url: string, init?: RequestInit) => Response) {
  const calls: { url: string; init?: RequestInit }[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
    calls.push({ url, init });
    return Promise.resolve(handler(url, init));
  }));
  return calls;
}

async function edit() {
  render(<SettingsPage version={0} />);
  const input = (await screen.findByLabelText('Repositório do harness')) as HTMLInputElement;
  fireEvent.input(input, { target: { value: 'outro/repo' } });
}

describe('SettingsPage', () => {
  it('shows the interface names and puts unmapped keys in Advanced', async () => {
    stub(() => json([file]));
    render(<SettingsPage version={0} />);
    expect(await screen.findByLabelText('Repositório do harness')).toBeTruthy();
    expect(screen.getByText('Avançado')).toBeTruthy();
    expect(screen.getByText('Extra option')).toBeTruthy();
  });

  it('shows the empty and error states', async () => {
    stub(() => json([]));
    const { unmount } = render(<SettingsPage version={0} />);
    expect(await screen.findByText('Nenhuma configuração')).toBeTruthy();
    unmount();
    stub(() => json({ erro: 'x' }, 500));
    render(<SettingsPage version={0} />);
    expect(await screen.findByText('Não foi possível ler as configurações')).toBeTruthy();
  });

  it('saves with the read hash and the rewritten YAML', async () => {
    const calls = stub((url, init) => (init?.method === 'PUT' ? json({ hash: 'h2', mensagem: 'configuração gravada', problemas: [] }) : json([file])));
    await edit();
    fireEvent.click(await screen.findByRole('button', { name: 'Salvar' }));
    await waitFor(() => expect(calls.some((call) => call.init?.method === 'PUT')).toBe(true));
    const put = calls.find((call) => call.init?.method === 'PUT')!;
    expect(put.url).toBe('/api/configuracao?caminho=vostok%2Fconfig%2Fmain.yml&hash=h1');
    expect(put.init?.body).toBe(MAIN.replace('dono/repo', 'outro/repo'));
    expect(await screen.findByText(/Configuração gravada/)).toBeTruthy();
  });

  it('shows the problems returned by the save', async () => {
    stub((url, init) => (init?.method === 'PUT' ? json({ hash: 'h2', mensagem: 'configuração gravada', problemas: ['regra inválida'] }) : json([file])));
    await edit();
    fireEvent.click(await screen.findByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('regra inválida')).toBeTruthy();
  });

  it('shows both texts on a 409 conflict', async () => {
    stub((url, init) => (init?.method === 'PUT' ? json({ hash: 'h9', conteudo: 'harness:\n  tool: codex\n' }, 409) : json([file])));
    await edit();
    fireEvent.click(await screen.findByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('Seu texto')).toBeTruthy();
    expect(screen.getByText('Versão no disco')).toBeTruthy();
    const areas = screen.getAllByRole('textbox', { hidden: true }).filter((el) => el.tagName === 'TEXTAREA') as HTMLTextAreaElement[];
    expect(areas.some((area) => area.value.includes('outro/repo'))).toBe(true);
    expect(areas.some((area) => area.value.includes('tool: codex'))).toBe(true);
  });

  it('shows a save failure', async () => {
    stub((url, init) => (init?.method === 'PUT' ? new Response('sem permissão', { status: 403 }) : json([file])));
    await edit();
    fireEvent.click(await screen.findByRole('button', { name: 'Salvar' }));
    expect(await screen.findByText('sem permissão')).toBeTruthy();
  });
  describe('limpeza dos pedidos', () => {
    const PATHS = ['vostok/work/.archive/SPEC-009/pedidos/PED-001.md', 'vostok/work/pedidos/feito/PED-002.md'];
    beforeAll(() => {
      HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
      HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
    });
    const prune = (list: () => Response, remove: () => Response = () => json({ apagados: PATHS })) =>
      stub((url) => (url === '/api/limpeza?confirmar=sim' ? remove() : url === '/api/limpeza' ? list() : json([file])));
    const confirmed = (calls: { url: string }[]) => calls.filter((call) => call.url.includes('confirmar')).length;
    async function openPrune() {
      render(<SettingsPage version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Limpar pedidos arquivados' }));
    }

    it('lista os pedidos arquivados antes de apagar', async () => {
      const calls = prune(() => json({ arquivos: PATHS }));
      await openPrune();
      const dialog = await screen.findByRole('dialog', { name: 'Limpar pedidos arquivados' });
      await waitFor(() => expect(dialog.textContent).toContain(PATHS[0]));
      expect(dialog.textContent).toContain(PATHS[1]);
      expect(calls.some((call) => call.url === '/api/limpeza' && call.init?.method === 'POST')).toBe(true);
      expect(confirmed(calls)).toBe(0);
    });

    it('apaga só depois de confirmar', async () => {
      const calls = prune(() => json({ arquivos: PATHS }));
      await openPrune();
      fireEvent.click(await screen.findByRole('button', { name: 'Cancelar' }));
      expect(confirmed(calls)).toBe(0);
      fireEvent.click(screen.getByRole('button', { name: 'Limpar pedidos arquivados' }));
      fireEvent.click(await screen.findByRole('button', { name: 'Apagar 2 arquivos' }));
      await waitFor(() => expect(confirmed(calls)).toBe(1));
      expect((await screen.findByRole('status')).textContent).toContain('2 arquivos apagados.');
    });

    it('diz que não há o que limpar', async () => {
      prune(() => json({ arquivos: [] }));
      await openPrune();
      expect(await screen.findByText('Não há pedidos nem perguntas para limpar.')).toBeTruthy();
      expect(screen.queryByRole('button', { name: /^Apagar/ })).toBeNull();
    });

    it('mostra o erro da limpeza', async () => {
      prune(() => new Response('falhou', { status: 500 }));
      await openPrune();
      expect((await screen.findByRole('alert')).textContent).toContain('falhou');
    });
  });
});
