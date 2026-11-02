import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
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
});
