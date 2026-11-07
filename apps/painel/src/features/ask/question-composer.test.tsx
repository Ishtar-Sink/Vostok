import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from 'vitest';
import type { AttachmentLimits } from '../../shared/api';
import { addAttachments, formatSize, isRefusedName, isThumbnailType, pastedFiles, QuestionComposer, type PendingAttachment, type QuestionComposerProps } from './question-composer';

const REFUSED_NAMES = [
  '.env',
  '.env.*',
  '*.pem',
  '*.key',
  '*.p12',
  '*.pfx',
  '*.jks',
  '*.keystore',
  '*.crt',
  '*.cer',
  '*.der',
  '*.kdbx',
  '*.ovpn',
  'id_rsa',
  'id_dsa',
  'id_ecdsa',
  'id_ed25519',
  'credentials',
  'credentials.*',
  'secrets',
  'secrets.*',
  '.netrc',
  '.npmrc',
  '.pgpass',
];
const LIMITS: AttachmentLimits = { max_mb: 1, max_arquivos: 10, nomes_recusados: REFUSED_NAMES, nomes_aceitos: ['.env.example'] };
const ONE_MB = 1024 * 1024;

const makeFile = (name: string, type = '', size = 1) => new File([new Uint8Array(size)], name, { type });
const png = (name = 'foto.png') => makeFile(name, 'image/png');

interface FakeEntry {
  isDirectory: boolean;
}
function fakeTransfer(files: File[], options: { text?: string; html?: string; directories?: string[]; withoutItems?: boolean } = {}) {
  const items = files.map((file) => ({
    kind: 'file',
    getAsFile: () => file,
    webkitGetAsEntry: (): FakeEntry => ({ isDirectory: options.directories?.includes(file.name) ?? false }),
  }));
  return {
    files,
    ...(options.withoutItems ? {} : { items }),
    getData: (format: string) => (format === 'text/plain' ? (options.text ?? '') : format === 'text/html' ? (options.html ?? '') : ''),
  };
}

function dispatch(target: Element | Document, type: string, key: 'clipboardData' | 'dataTransfer', transfer: ReturnType<typeof fakeTransfer>) {
  const event = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperty(event, key, { value: transfer });
  fireEvent(target as Element, event);
  return event;
}
const paste = (transfer: ReturnType<typeof fakeTransfer>) => dispatch(field(), 'paste', 'clipboardData', transfer);
const drop = (transfer: ReturnType<typeof fakeTransfer>) => dispatch(field(), 'drop', 'dataTransfer', transfer);

const field = () => screen.getByLabelText('Sua pergunta') as HTMLTextAreaElement;
const send = () => screen.getByRole('button', { name: 'Enviar pergunta' }) as HTMLButtonElement;
const attach = () => screen.getByRole('button', { name: 'Anexar' });
const removeButton = (label: string) => screen.getByRole('button', { name: `Remover ${label}` });
const listed = () => [...document.querySelectorAll('ul[aria-label="Anexos"] li')];
const statusText = () => screen.getByRole('status').textContent ?? '';
const type = (value: string) => fireEvent.input(field(), { target: { value } });

function choose(files: File[]) {
  const input = document.querySelector('input[type=file]') as HTMLInputElement;
  Object.defineProperty(input, 'files', { value: files, configurable: true });
  fireEvent.change(input);
}

let onSubmit: Mock<QuestionComposerProps['onSubmit']>;
let createObjectUrl: ReturnType<typeof vi.fn>;
let revokeObjectUrl: ReturnType<typeof vi.fn>;

function renderComposer(limits: AttachmentLimits | null = LIMITS) {
  return render(<QuestionComposer limits={limits} onSubmit={onSubmit} />);
}

beforeEach(() => {
  onSubmit = vi.fn<QuestionComposerProps['onSubmit']>(() => Promise.resolve(null));
  let counter = 0;
  createObjectUrl = vi.fn(() => `blob:miniatura-${(counter += 1)}`);
  revokeObjectUrl = vi.fn();
  URL.createObjectURL = createObjectUrl as unknown as typeof URL.createObjectURL;
  URL.revokeObjectURL = revokeObjectUrl as unknown as typeof URL.revokeObjectURL;
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe('entrada', () => {
  it('cola imagem sem texto e anexa', () => {
    renderComposer();
    const event = paste(fakeTransfer([png()]));
    expect(event.defaultPrevented).toBe(true);
    expect(listed()).toHaveLength(1);
    expect(listed()[0]?.textContent).toContain('foto.png');
  });

  it.each(['relatorio.pdf', '/home/ana/relatorio.pdf', 'C:\\Users\\ana\\relatorio.pdf'])('cola arquivo copiado com o nome como texto e anexa (%s)', (text) => {
    renderComposer();
    const event = paste(fakeTransfer([makeFile('relatorio.pdf', 'application/pdf')], { text }));
    expect(event.defaultPrevented).toBe(true);
    expect(listed()).toHaveLength(1);
    expect(field().value).toBe('');
  });

  it('cola só texto e deixa no campo', () => {
    renderComposer();
    const event = paste(fakeTransfer([], { text: 'só um texto' }));
    expect(event.defaultPrevented).toBe(false);
    expect(listed()).toHaveLength(0);
  });

  it('cola texto com imagem e entra só o texto', () => {
    renderComposer();
    const event = paste(fakeTransfer([png('celula.png')], { text: 'a\tb\nc\td', html: '<table><tr><td>a</td></tr></table>' }));
    expect(event.defaultPrevented).toBe(false);
    expect(listed()).toHaveLength(0);
    expect(createObjectUrl).not.toHaveBeenCalled();
  });

  it('arrasta arquivos para o campo', () => {
    renderComposer();
    drop(fakeTransfer([makeFile('a.zip'), makeFile('b.zip')]));
    expect(listed()).toHaveLength(2);
  });

  it('recusa a pasta arrastada e mantém os outros arquivos', () => {
    renderComposer();
    drop(fakeTransfer([makeFile('fotos'), makeFile('a.zip')], { directories: ['fotos'] }));
    expect(listed()).toHaveLength(1);
    expect(listed()[0]?.textContent).toContain('a.zip');
    expect(statusText()).toContain('fotos');
    expect(statusText()).toContain('pasta');
  });

  it('sem items, o arquivo arrastado não vira pasta', () => {
    renderComposer();
    drop(fakeTransfer([makeFile('a.zip')], { withoutItems: true }));
    expect(listed()).toHaveLength(1);
  });

  it('solta fora do campo sem trocar de página nem anexar', () => {
    renderComposer();
    type('texto escrito');
    const event = dispatch(document.body, 'drop', 'dataTransfer', fakeTransfer([makeFile('solto.zip')]));
    expect(event.defaultPrevented).toBe(true);
    expect(listed()).toHaveLength(0);
    expect(field().value).toBe('texto escrito');
  });

  it('retira de window o que evita a troca de página ao desmontar', () => {
    const { unmount } = renderComposer();
    unmount();
    const event = new Event('drop', { bubbles: true, cancelable: true });
    document.body.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
  });

  it('escolhe pelo botão Anexar', () => {
    renderComposer();
    const input = document.querySelector('input[type=file]') as HTMLInputElement;
    const click = vi.spyOn(input, 'click');
    fireEvent.click(attach());
    expect(click).toHaveBeenCalledTimes(1);
    expect(input.multiple).toBe(true);
    choose([makeFile('a.zip'), makeFile('b.zip')]);
    expect(listed()).toHaveLength(2);
    expect(input.value).toBe('');
  });

  it('os três caminhos enchem a mesma lista', () => {
    renderComposer();
    paste(fakeTransfer([png('colada.png')]));
    drop(fakeTransfer([makeFile('solto.zip')]));
    choose([makeFile('escolhido.pdf')]);
    expect(listed().map((item) => item.textContent)).toEqual([expect.stringContaining('colada.png'), expect.stringContaining('solto.zip'), expect.stringContaining('escolhido.pdf')]);
  });

  it('imagem colada sem nome ganha o nome image.png', () => {
    renderComposer();
    paste(fakeTransfer([png('')]));
    expect(listed()[0]?.textContent).toContain('image.png');
    expect(removeButton('image.png')).toBeTruthy();
  });

  it('pastedFiles só devolve o que a regra de colagem anexa', () => {
    const file = makeFile('a.txt');
    expect(pastedFiles(fakeTransfer([], { text: 'x' }) as never)).toEqual([]);
    expect(pastedFiles(fakeTransfer([file]) as never)).toEqual([{ file, isDirectory: false }]);
    expect(pastedFiles(fakeTransfer([file], { text: '/tmp/a.txt\n/tmp/a.txt\n' }) as never)).toHaveLength(1);
    expect(pastedFiles(fakeTransfer([file], { text: '/tmp/a.txt\noutra linha' }) as never)).toEqual([]);
    expect(pastedFiles(fakeTransfer([file], { directories: ['a.txt'] }) as never)).toEqual([{ file, isDirectory: true }]);
  });
});

describe('lista', () => {
  it('mostra nome, tamanho e Remover em todos, com miniatura só na imagem', () => {
    renderComposer();
    choose([makeFile('foto.png', 'image/png', 12), makeFile('dados.zip', 'application/zip', 2048), makeFile('logo.svg', 'image/svg+xml', 3), makeFile('pagina.html', 'text/html', 4)]);
    const rows = listed();
    expect(rows).toHaveLength(4);
    expect(rows[0]?.textContent).toContain('12 B');
    expect(rows[1]?.textContent).toContain('2,0 KB');
    for (const label of ['foto.png', 'dados.zip', 'logo.svg', 'pagina.html']) expect(removeButton(label)).toBeTruthy();
    expect(rows.map((row) => row.querySelectorAll('img').length)).toEqual([1, 0, 0, 0]);
    expect(rows[0]?.querySelector('img')?.getAttribute('alt')).toBe('');
    expect(createObjectUrl).toHaveBeenCalledTimes(1);
  });

  it('remove pelo teclado e passa o foco ao item seguinte', () => {
    renderComposer();
    choose([makeFile('a.zip'), makeFile('b.zip'), makeFile('c.zip')]);
    fireEvent.click(removeButton('a.zip'));
    expect(listed()).toHaveLength(2);
    expect(document.activeElement).toBe(removeButton('b.zip'));
  });

  it('sem item seguinte, o foco vai ao anterior e, sem nenhum, ao Anexar', () => {
    renderComposer();
    choose([makeFile('a.zip'), makeFile('b.zip')]);
    fireEvent.click(removeButton('b.zip'));
    expect(document.activeElement).toBe(removeButton('a.zip'));
    fireEvent.click(removeButton('a.zip'));
    expect(document.activeElement).toBe(attach());
  });

  it('anuncia o anexado, o recusado e o removido por aria-live', () => {
    renderComposer();
    const status = screen.getByRole('status');
    expect(status.getAttribute('aria-live')).toBe('polite');
    choose([makeFile('a.zip'), makeFile('.env')]);
    expect(statusText()).toContain('a.zip');
    expect(statusText()).toContain('.env');
    fireEvent.click(removeButton('a.zip'));
    expect(statusText()).toContain('a.zip');
    expect(statusText()).not.toContain('.env');
    expect(screen.getByRole('status')).toBe(status);
  });

  it('não chama a rede ao anexar e remover', () => {
    const fetchSpy = vi.fn();
    const xhrSpy = vi.fn();
    vi.stubGlobal('fetch', fetchSpy);
    vi.stubGlobal('XMLHttpRequest', xhrSpy);
    renderComposer();
    paste(fakeTransfer([png()]));
    drop(fakeTransfer([makeFile('a.zip')]));
    choose([makeFile('b.zip')]);
    fireEvent.click(removeButton('a.zip'));
    expect(fetchSpy).not.toHaveBeenCalled();
    expect(xhrSpy).not.toHaveBeenCalled();
  });

  it('não grava nada no navegador', () => {
    const before = [Object.keys(localStorage), Object.keys(sessionStorage)];
    renderComposer();
    type('texto');
    paste(fakeTransfer([png()]));
    fireEvent.click(removeButton('foto.png'));
    expect([Object.keys(localStorage), Object.keys(sessionStorage)]).toEqual(before);
  });

  it('revoga a miniatura ao remover', () => {
    renderComposer();
    paste(fakeTransfer([png()]));
    fireEvent.click(removeButton('foto.png'));
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:miniatura-1');
  });

  it('revoga as miniaturas ao desmontar', () => {
    const { unmount } = renderComposer();
    paste(fakeTransfer([png()]));
    unmount();
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:miniatura-1');
  });

  it('duas image.png têm rótulos diferentes', () => {
    renderComposer();
    paste(fakeTransfer([png('image.png')]));
    paste(fakeTransfer([png('image.png')]));
    expect(removeButton('image.png')).toBeTruthy();
    expect(removeButton('image.png (2)')).toBeTruthy();
    fireEvent.click(removeButton('image.png'));
    paste(fakeTransfer([png('image.png')]));
    expect(removeButton('image.png')).toBeTruthy();
    expect(removeButton('image.png (2)')).toBeTruthy();
  });

  it('formata o tamanho em B, KB e MB', () => {
    expect(formatSize(12)).toBe('12 B');
    expect(formatSize(3482)).toBe('3,4 KB');
    expect(formatSize(1258291)).toBe('1,2 MB');
  });

  it.each([
    ['image/png', true],
    ['image/jpeg', true],
    ['image/gif', true],
    ['image/webp', true],
    ['image/svg+xml', false],
    ['text/html', false],
    ['', false],
  ])('isThumbnailType(%s) é %s', (mime, expected) => {
    expect(isThumbnailType(mime)).toBe(expected);
  });
});

describe('limites', () => {
  it('recusa o décimo primeiro com o motivo e mantém os outros', () => {
    renderComposer();
    const names = Array.from({ length: 11 }, (_, index) => `f${index + 1}.txt`);
    choose(names.map((name) => makeFile(name)));
    expect(listed()).toHaveLength(10);
    expect(statusText()).toContain('f11.txt');
    expect(statusText()).toContain('10');
  });

  it('recusa a soma acima do limite com o motivo', () => {
    renderComposer();
    choose([makeFile('a.bin', '', ONE_MB - 10), makeFile('b.bin', '', 11), makeFile('c.bin', '', 10)]);
    expect(listed().map((item) => item.textContent)).toEqual([expect.stringContaining('a.bin'), expect.stringContaining('c.bin')]);
    expect(statusText()).toContain('b.bin');
    expect(statusText()).toContain('1 MB');
  });

  it('recusa arquivo vazio com o motivo', () => {
    renderComposer();
    choose([makeFile('vazio.txt', '', 0)]);
    expect(listed()).toHaveLength(0);
    expect(statusText()).toContain('vazio.txt');
    expect(statusText()).toContain('vazio');
  });

  it.each(['.env', '.ENV', '.env.local', '.ENV.local', 'servidor.PEM', 'chave.key', 'ID_RSA', 'id_ed25519', 'credentials', 'Credentials.json', 'secrets.yml', '.NPMRC', '.pgpass', '/tmp/.env', 'C:\\x\\id_rsa', '.env. ', 'id_rsa.'])(
    'recusa cada nome de segredo sem diferenciar maiúsculas e aceita .env.example (%s)',
    (name) => {
      expect(isRefusedName(name, LIMITS)).toBe(true);
      expect(isRefusedName('.env.example', LIMITS)).toBe(false);
      expect(isRefusedName('.ENV.EXAMPLE', LIMITS)).toBe(false);
    },
  );

  it.each(['environment.txt', 'env', 'minha.env', 'credentials2.json', 'secretsx', 'pem'])('aceita o nome parecido que não é de segredo (%s)', (name) => {
    expect(isRefusedName(name, LIMITS)).toBe(false);
  });

  it('a recusa do nome diz como pedir na própria pergunta', () => {
    renderComposer();
    choose([makeFile('.ENV.local')]);
    expect(listed()).toHaveLength(0);
    expect(statusText()).toContain('.ENV.local parece um arquivo de segredo. Para o agente ler, peça na própria pergunta, citando o arquivo.');
  });

  it('sem os limites da API, não confere na tela e não trava', () => {
    renderComposer(null);
    choose([...Array.from({ length: 11 }, (_, index) => makeFile(`f${index}.txt`)), makeFile('.env'), makeFile('grande.bin', '', ONE_MB * 2)]);
    expect(listed()).toHaveLength(13);
    type('oi');
    expect(send().disabled).toBe(false);
  });

  it('addAttachments não muda a lista recebida', () => {
    const current: ReturnType<typeof addAttachments>['next'] = [];
    const { next, refused } = addAttachments(current, [{ file: makeFile('a.txt'), isDirectory: false }], LIMITS);
    expect(current).toHaveLength(0);
    expect(next).toHaveLength(1);
    expect(refused).toEqual([]);
  });

  it('Enviar fica desabilitado sem texto, com o motivo', () => {
    renderComposer();
    paste(fakeTransfer([png()]));
    expect(send().disabled).toBe(true);
    const reasonId = send().getAttribute('aria-describedby') ?? '';
    expect(document.getElementById(reasonId)?.textContent).toBe('Escreva a pergunta: o texto continua obrigatório, mesmo com anexos.');
    type('   ');
    expect(send().disabled).toBe(true);
    type('oi');
    expect(send().disabled).toBe(false);
    expect(send().getAttribute('aria-describedby')).toBeNull();
  });
});

describe('Prévia', () => {
  const tab = (name: string) => screen.getByRole('tab', { name });

  it('alterna entre Escrever e Prévia pelo teclado', () => {
    renderComposer();
    type('**forte**');
    const write = tab('Escrever');
    write.focus();
    fireEvent.keyDown(write, { key: 'ArrowRight' });
    expect(document.activeElement).toBe(tab('Prévia'));
    // O jsdom não transforma Enter em clique, como o navegador faz num botão com foco.
    fireEvent.keyDown(document.activeElement as Element, { key: 'Enter' });
    fireEvent.click(document.activeElement as Element);
    expect(tab('Prévia').getAttribute('aria-selected')).toBe('true');
    expect(document.querySelector('[role=tabpanel] strong')?.textContent).toBe('forte');
    expect(screen.queryByLabelText('Sua pergunta')).toBeNull();
    fireEvent.keyDown(tab('Prévia'), { key: 'ArrowLeft' });
    fireEvent.click(document.activeElement as Element);
    expect(field().value).toBe('**forte**');
  });

  it('a Prévia mostra a formatação e o texto continua o digitado', () => {
    renderComposer();
    type('**forte** e _leve_');
    fireEvent.click(tab('Prévia'));
    expect(document.querySelector('[role=tabpanel] strong')?.textContent).toBe('forte');
    fireEvent.click(tab('Escrever'));
    expect(field().value).toBe('**forte** e _leve_');
  });

  it('Prévia vazia diz que não há o que mostrar', () => {
    renderComposer();
    fireEvent.click(tab('Prévia'));
    expect(screen.getByText('Nada para mostrar ainda.')).toBeTruthy();
  });

  it('ataque na Prévia aparece como texto', () => {
    renderComposer();
    type('<img src=x onerror=y> <script>z</script> [a](javascript:b) ![foto](http://exemplo.test/y.png)');
    fireEvent.click(tab('Prévia'));
    const panel = document.querySelector('[role=tabpanel]:not([hidden])') as HTMLElement;
    expect(panel.querySelector('img')).toBeNull();
    expect(panel.querySelector('script')).toBeNull();
    expect(panel.querySelector('a[href^="javascript"]')).toBeNull();
    expect(panel.textContent).toContain('onerror=y');
    expect(panel.textContent).toContain('<script>');
  });

  it('Enviar entrega o texto digitado e os arquivos', () => {
    renderComposer();
    type('**forte**');
    paste(fakeTransfer([png()]));
    fireEvent.click(tab('Prévia'));
    fireEvent.click(send());
    expect(onSubmit).toHaveBeenCalledTimes(1);
    const [text, attachments] = onSubmit.mock.calls[0] ?? ['', []];
    expect(text).toBe('**forte**');
    expect(attachments.map((item) => item.name)).toEqual(['foto.png']);
    expect(attachments[0]?.file.name).toBe('foto.png');
  });
});

describe('envio', () => {
  type Report = Parameters<QuestionComposerProps['onSubmit']>[2];
  function hold() {
    let finish: (reason: string | null) => void = () => undefined;
    let report: Report = () => undefined;
    onSubmit.mockImplementation((_text, _items, reporter) => {
      report = reporter;
      return new Promise<string | null>((resolve) => {
        finish = resolve;
      });
    });
    return { report: (id: string, percent: number) => act(() => report(id, percent)), finish: (reason: string | null) =>
        act(async () => {
          finish(reason);
          await new Promise((resolve) => setTimeout(resolve, 0));
        }) };
  }
  const bars = () => [...document.querySelectorAll('[role=progressbar]')];

  it('mostra o progresso de cada arquivo', async () => {
    const control = hold();
    renderComposer();
    type('Pergunta');
    paste(fakeTransfer([png('a.png')]));
    drop(fakeTransfer([makeFile('b.zip')]));
    expect(bars()).toHaveLength(0);
    fireEvent.click(send());
    const [first, second] = onSubmit.mock.calls[0]?.[1] ?? [];
    expect(bars().map((bar) => bar.getAttribute('aria-label'))).toEqual(['Enviando a.png', 'Enviando b.zip']);
    await control.report(first?.id ?? '', 40);
    await control.report(second?.id ?? '', 0);
    expect(bars().map((bar) => bar.getAttribute('aria-valuenow'))).toEqual(['40', '0']);
    expect(statusText()).toContain('Enviando b.zip.');
    expect(statusText()).not.toContain('40');
    await control.finish(null);
  });

  it('recusa em um dos arquivos mostra o motivo, mantém texto e lista', async () => {
    const control = hold();
    renderComposer();
    type('Pergunta');
    paste(fakeTransfer([png('a.png')]));
    fireEvent.click(send());
    await control.report(onSubmit.mock.calls[0]?.[1][0]?.id ?? '', 60);
    await control.finish('Não foi possível enviar a.png: grande demais. Nada entrou na fila.');
    expect((await screen.findByRole('alert')).textContent).toContain('grande demais');
    expect(field().value).toBe('Pergunta');
    expect(listed()).toHaveLength(1);
    expect(bars()).toHaveLength(0);
    expect(send().disabled).toBe(false);
    await waitFor(() => expect(document.activeElement).toBe(send()));
  });

  it('desabilita Enviar, Anexar e Remover durante o envio', async () => {
    const control = hold();
    renderComposer();
    type('Pergunta');
    paste(fakeTransfer([png('a.png')]));
    fireEvent.click(send());
    expect(send().disabled).toBe(true);
    expect((attach() as HTMLButtonElement).disabled).toBe(true);
    expect((removeButton('a.png') as HTMLButtonElement).disabled).toBe(true);
    expect(field().disabled).toBe(true);
    expect(screen.getAllByRole('tab').every((tab) => tab.matches(':disabled'))).toBe(true);
    await control.finish(null);
    expect(send().disabled).toBe(true);
    expect(field().disabled).toBe(false);
  });

  it('limpa o campo e a lista depois de aceito, e leva o foco ao campo', async () => {
    const control = hold();
    renderComposer();
    type('Pergunta');
    paste(fakeTransfer([png('a.png')]));
    fireEvent.click(send());
    await control.finish(null);
    expect(field().value).toBe('');
    expect(listed()).toHaveLength(0);
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:miniatura-1');
    expect(statusText()).toContain('Pergunta enviada.');
    await waitFor(() => expect(document.activeElement).toBe(field()));
  });
});
