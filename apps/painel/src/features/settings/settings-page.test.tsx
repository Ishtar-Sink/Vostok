import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { saveEvidenceSettings } from '../../shared/api';
import { navigate } from '../../shared/router';
import { SettingsPage } from './settings-page';

vi.mock('../../shared/api', async (original) => ({
  ...(await original<typeof import('../../shared/api')>()),
  saveEvidenceSettings: vi.fn(),
}));
vi.mock('../../shared/router', async (original) => ({
  ...(await original<typeof import('../../shared/router')>()),
  navigate: vi.fn(),
}));

const READ = {
  arquivos: { 'tasks.yml': 'sha-tarefas', 'documents.yml': 'sha-documentos' },
  valores: {
    'tasks.evidence_review_kinds': ['automática', 'automática, da verificação', 'diff', 'manual', 'print'],
    'tasks.evidence_baseline_timeout': 120,
    'tasks.evidence_title_max': 60,
    'tasks.persist_dir': '',
    'tasks.persist_max_mb': 10,
    'tasks.persist_readme': 'README.md',
    'tasks.regression_ignore': ['Start at', 'Duration', 'finished in', 'Time:'],
    'tasks.regression_alert': ['FAILED', 'error', 'warning', 'panicked'],
    'documents.embed_max_mb': 10,
    'documents.text_extensions': ['out', 'txt', 'log', 'md', 'json'],
    'documents.image_extensions': ['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'],
    'documents.video_extensions': ['mp4', 'webm'],
  },
};

const NO_DIFF = ['automática', 'automática, da verificação', 'manual', 'print'];
const save = vi.mocked(saveEvidenceSettings);

function mockRead(...bodies: unknown[]) {
  let call = 0;
  const fake = vi.fn(() => {
    const body = bodies[Math.min(call, bodies.length - 1)];
    call += 1;
    return Promise.resolve(new Response(JSON.stringify(body), { status: 200 }));
  });
  vi.stubGlobal('fetch', fake);
  return fake;
}

async function ready() {
  await screen.findByRole('heading', { name: 'Evidências' });
}

const button = (name: string) => screen.getByRole('button', { name }) as HTMLButtonElement;
const box = (name: string) => screen.getByRole('checkbox', { name }) as HTMLInputElement;
const toggle = () => screen.getByRole('switch', { name: 'Copiar as evidências para o projeto' });

function typeTag(label: string, text: string) {
  const input = screen.getByRole('combobox', { name: label });
  fireEvent.input(input, { target: { value: text } });
  fireEvent.keyDown(input, { key: 'Enter' });
}

beforeEach(() => {
  save.mockReset();
  vi.mocked(navigate).mockReset();
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('SettingsPage', () => {
  it('mostra o carregando e depois o grupo Evidências com as quatro seções', async () => {
    mockRead(READ);
    render(<SettingsPage />);
    expect(screen.getByText('Carregando as configurações')).toBeTruthy();
    await ready();
    expect(screen.getByRole('heading', { level: 1, name: 'Configurações' })).toBeTruthy();
    for (const title of ['Conferência e geração', 'Persistência no projeto', 'Evidências nos documentos', 'Regressão']) {
      expect(screen.getByRole('heading', { name: title })).toBeTruthy();
    }
  });

  it('mostra o erro da leitura com Tentar de novo', async () => {
    const fake = vi.fn(() => Promise.resolve(new Response('{}', { status: 500 })));
    vi.stubGlobal('fetch', fake);
    render(<SettingsPage />);
    await screen.findByText('Não foi possível ler as configurações');
    fireEvent.click(button('Tentar de novo'));
    await waitFor(() => expect(fake).toHaveBeenCalledTimes(2));
  });

  it('mostra um tipo por caixa de seleção, todos marcados sem a chave no arquivo', async () => {
    const { 'tasks.evidence_review_kinds': _omitted, ...valores } = READ.valores;
    mockRead({ ...READ, valores });
    render(<SettingsPage />);
    await ready();
    for (const name of ['Automática', 'Automática, da verificação', 'Diff', 'Manual', 'Print']) {
      expect(box(name).checked).toBe(true);
    }
  });

  it('mostra os campos numéricos com a unidade e a persistência desligada', async () => {
    mockRead(READ);
    render(<SettingsPage />);
    await ready();
    expect((screen.getByLabelText('Tempo-limite das evidências no baseline') as HTMLInputElement).value).toBe('120');
    expect((screen.getByLabelText('Tamanho máximo do título') as HTMLInputElement).value).toBe('60');
    expect((screen.getByLabelText('Embutir até') as HTMLInputElement).value).toBe('10');
    for (const unit of ['s', 'caracteres', 'MB']) expect(screen.getByText(unit)).toBeTruthy();
    expect(toggle().getAttribute('aria-checked')).toBe('false');
    expect(screen.queryByLabelText('Pasta de destino')).toBeNull();
  });

  it('deixa Salvar e Descartar desabilitados sem mudança', async () => {
    mockRead(READ);
    render(<SettingsPage />);
    await ready();
    expect(button('Salvar').disabled).toBe(true);
    expect(button('Descartar').disabled).toBe(true);
  });

  it('desmarca Diff e salva só a chave alterada com o sha256 lido', async () => {
    mockRead(READ);
    save.mockResolvedValue({
      ok: true,
      data: {
        arquivos: { 'tasks.yml': 'sha-novo', 'documents.yml': 'sha-documentos' },
        valores: { ...READ.valores, 'tasks.evidence_review_kinds': NO_DIFF },
      },
    });
    render(<SettingsPage />);
    await ready();
    fireEvent.click(box('Diff'));
    expect(button('Salvar').disabled).toBe(false);
    expect(button('Descartar').disabled).toBe(false);
    fireEvent.click(button('Salvar'));
    await screen.findByText('Configurações salvas');
    expect(save).toHaveBeenCalledWith({ 'tasks.evidence_review_kinds': NO_DIFF }, READ.arquivos);
    expect(box('Diff').checked).toBe(false);
    expect(button('Salvar').disabled).toBe(true);
    expect(button('Descartar').disabled).toBe(true);
  });

  it('avisa quando nenhum tipo pede conferência', async () => {
    mockRead(READ);
    save.mockResolvedValue({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    for (const name of ['Automática', 'Automática, da verificação', 'Diff', 'Manual', 'Print']) fireEvent.click(box(name));
    expect(screen.getByText('Nenhuma evidência vai pedir conferência.')).toBeTruthy();
    fireEvent.click(button('Salvar'));
    await waitFor(() => expect(save).toHaveBeenCalledWith({ 'tasks.evidence_review_kinds': [] }, READ.arquivos));
  });

  it('liga a persistência, exige a pasta e salva o caminho', async () => {
    mockRead(READ);
    save.mockResolvedValue({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    fireEvent.click(toggle());
    expect(screen.getByLabelText('Pasta de destino')).toBeTruthy();
    expect(screen.getByLabelText('Copiar arquivos até')).toBeTruthy();
    expect(screen.getByLabelText('Guia da pasta')).toBeTruthy();
    fireEvent.click(button('Salvar'));
    expect(save).not.toHaveBeenCalled();
    expect(screen.getAllByText('Preencha a pasta de destino.').length).toBeGreaterThan(0);
    expect(screen.getByLabelText('Pasta de destino').getAttribute('aria-invalid')).toBe('true');
    fireEvent.input(screen.getByLabelText('Pasta de destino'), { target: { value: 'docs/evidencias/<SPEC>/<TASK>' } });
    fireEvent.click(button('Salvar'));
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith({ 'tasks.persist_dir': 'docs/evidencias/<SPEC>/<TASK>' }, READ.arquivos),
    );
  });

  it('desliga a persistência, esconde os campos e salva a pasta vazia', async () => {
    mockRead({ ...READ, valores: { ...READ.valores, 'tasks.persist_dir': 'docs/evidencias/<SPEC>/<TASK>' } });
    save.mockResolvedValue({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    expect(screen.getByLabelText('Pasta de destino')).toBeTruthy();
    fireEvent.click(toggle());
    expect(screen.queryByLabelText('Pasta de destino')).toBeNull();
    fireEvent.click(button('Salvar'));
    await waitFor(() => expect(save).toHaveBeenCalledWith({ 'tasks.persist_dir': '' }, READ.arquivos));
  });

  it('acrescenta a extensão normalizada e recusa a com espaço', async () => {
    mockRead(READ);
    save.mockResolvedValue({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    typeTag('Extensões de texto', '.NDJSON');
    const list = screen.getByRole('list', { name: 'Extensões de texto adicionadas' });
    expect(within(list).getByText('ndjson')).toBeTruthy();
    typeTag('Extensões de texto', 'nd json');
    expect(within(list).queryByText('nd json')).toBeNull();
    fireEvent.click(button('Salvar'));
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith(
        { 'documents.text_extensions': ['out', 'txt', 'log', 'md', 'json', 'ndjson'] },
        READ.arquivos,
      ),
    );
  });

  it('escolhe uma sugestão e acrescenta um trecho novo na regressão', async () => {
    mockRead(READ);
    save.mockResolvedValue({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    fireEvent.input(screen.getByRole('combobox', { name: 'Trechos ignorados' }), { target: { value: 'Ela' } });
    fireEvent.mouseDown(screen.getByRole('option', { name: 'Elapsed' }));
    fireEvent.click(screen.getByRole('option', { name: 'Elapsed' }));
    typeTag('Trechos ignorados', 'Tempo total');
    fireEvent.click(button('Salvar'));
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith(
        { 'tasks.regression_ignore': ['Start at', 'Duration', 'finished in', 'Time:', 'Elapsed', 'Tempo total'] },
        READ.arquivos,
      ),
    );
  });

  it('mostra ao lado do campo o erro devolvido pela CLI', async () => {
    mockRead(READ);
    save.mockResolvedValue({
      ok: false,
      status: 400,
      erros: { 'tasks.evidence_baseline_timeout': 'Use um número de 0 a 3600.' },
      message: 'Corrija os campos marcados.',
    });
    render(<SettingsPage />);
    await ready();
    const field = screen.getByLabelText('Tempo-limite das evidências no baseline');
    fireEvent.input(field, { target: { value: '-1' } });
    fireEvent.click(button('Salvar'));
    await screen.findByText('Use um número de 0 a 3600.');
    expect(field.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByText('Corrija os campos marcados.').closest('[role="alert"]')).toBeTruthy();
    expect(button('Salvar').disabled).toBe(false);
    fireEvent.input(field, { target: { value: '5' } });
    expect(screen.queryByText('Use um número de 0 a 3600.')).toBeNull();
  });

  it('mostra no aviso do grupo o erro que não é de um campo', async () => {
    mockRead(READ);
    save.mockResolvedValue({
      ok: false,
      status: 400,
      erros: { corpo: 'Corpo ilegível.', 'arquivos.tasks.yml': 'Falta o sha256 lido.' },
      message: 'Corrija os campos marcados.',
    });
    render(<SettingsPage />);
    await ready();
    fireEvent.click(box('Diff'));
    fireEvent.click(button('Salvar'));
    const alert = await screen.findByText(/Corpo ilegível\./);
    expect(alert.closest('[role="alert"]')?.textContent).toContain('Falta o sha256 lido.');
  });

  it('avisa o conflito e recarrega mantendo as mudanças', async () => {
    const fake = mockRead(READ, { ...READ, arquivos: { ...READ.arquivos, 'tasks.yml': 'sha-novo' } });
    save.mockResolvedValueOnce({ ok: false, status: 409, arquivo: 'tasks.yml', message: 'O arquivo tasks.yml mudou desde a leitura. Recarregue para continuar; as suas mudanças ficam na tela.' });
    save.mockResolvedValueOnce({ ok: true, data: READ });
    render(<SettingsPage />);
    await ready();
    fireEvent.click(box('Diff'));
    fireEvent.click(button('Salvar'));
    const alert = (await screen.findByText(/tasks\.yml/)).closest('[role="alert"]') as HTMLElement;
    fireEvent.click(within(alert).getByRole('button', { name: 'Recarregar' }));
    await waitFor(() => expect(fake).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Recarregar' })).toBeNull());
    expect(box('Diff').checked).toBe(false);
    expect(button('Salvar').disabled).toBe(false);
    fireEvent.click(button('Salvar'));
    await waitFor(() =>
      expect(save).toHaveBeenLastCalledWith(
        { 'tasks.evidence_review_kinds': NO_DIFF },
        { ...READ.arquivos, 'tasks.yml': 'sha-novo' },
      ),
    );
  });

  it('Descartar volta aos valores lidos', async () => {
    mockRead(READ);
    render(<SettingsPage />);
    await ready();
    fireEvent.click(box('Diff'));
    fireEvent.input(screen.getByLabelText('Tamanho máximo do título'), { target: { value: '80' } });
    fireEvent.click(button('Descartar'));
    expect(box('Diff').checked).toBe(true);
    expect((screen.getByLabelText('Tamanho máximo do título') as HTMLInputElement).value).toBe('60');
    expect(button('Salvar').disabled).toBe(true);
    expect(button('Descartar').disabled).toBe(true);
  });

  it('pede confirmação ao sair com mudança pendente e fica ao cancelar', async () => {
    mockRead(READ);
    const followed = vi.fn((event: { preventDefault: () => void }) => event.preventDefault());
    render(
      <>
        <a href="/tarefas" onClick={followed}>Tarefas</a>
        <SettingsPage />
      </>,
    );
    await ready();
    fireEvent.click(box('Diff'));
    fireEvent.click(screen.getByRole('link', { name: 'Tarefas' }));
    const dialog = await screen.findByRole('dialog', { name: 'Sair sem salvar?' });
    await waitFor(() => expect(document.activeElement).toBe(within(dialog).getByRole('button', { name: 'Continuar editando' })));
    expect(followed).not.toHaveBeenCalled();
    expect(navigate).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole('button', { name: 'Continuar editando' }));
    expect(screen.queryByRole('dialog', { name: 'Sair sem salvar?' })).toBeNull();
    fireEvent.click(screen.getByRole('link', { name: 'Tarefas' }));
    fireEvent.click(within(await screen.findByRole('dialog', { name: 'Sair sem salvar?' })).getByRole('button', { name: 'Sair sem salvar' }));
    expect(navigate).toHaveBeenCalledWith('/tarefas');
  });

  it('sai sem confirmação quando não há mudança', async () => {
    mockRead(READ);
    const followed = vi.fn((event: { preventDefault: () => void }) => event.preventDefault());
    render(
      <>
        <a href="/tarefas" onClick={followed}>Tarefas</a>
        <SettingsPage />
      </>,
    );
    await ready();
    fireEvent.click(screen.getByRole('link', { name: 'Tarefas' }));
    expect(followed).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('dialog')).toBeNull();
  });

  it('pede a confirmação do navegador ao fechar com mudança pendente', async () => {
    mockRead(READ);
    render(<SettingsPage />);
    await ready();
    const before = new Event('beforeunload', { cancelable: true });
    window.dispatchEvent(before);
    expect(before.defaultPrevented).toBe(false);
    fireEvent.click(box('Diff'));
    const after = new Event('beforeunload', { cancelable: true });
    window.dispatchEvent(after);
    expect(after.defaultPrevented).toBe(true);
  });
});
