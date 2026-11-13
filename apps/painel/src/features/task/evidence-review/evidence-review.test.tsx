import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import type { EvidenceReview as Review } from '../../../shared/api';
import type { EvidenceEntry } from '../evidence-list/evidence-items';
import { EvidenceReview } from './evidence-review';
import styles from './evidence-review.module.css';

const MB = 1024 * 1024;
const LOG = '$ cmd\n# data: d\n# commit: c\n# código de saída: 0\ntest a ... ok\ntest b ... ok\n0 failed\nFAILED x\nFAILED y';

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
  globalThis.IntersectionObserver = class { observe() {} disconnect() {} unobserve() {} takeRecords() { return []; } } as never;
  Element.prototype.scrollIntoView = function scrollIntoView() {};
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

type Call = { url: string; init?: RequestInit };

function mockFiles(body: string | (() => Response) = LOG): Call[] {
  const calls: Call[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string, init?: RequestInit) => {
    calls.push({ url, init });
    return Promise.resolve(typeof body === 'string' ? new Response(body) : body());
  }));
  return calls;
}

function entry(over: Omit<Partial<EvidenceEntry>, 'review'> & { review?: Partial<Review> } = {}): EvidenceEntry {
  const name = over.name ?? 'saida.txt';
  const { review, ...rest } = over;
  return {
    name,
    id: 'EV-01',
    title: 'Testes do quadro passam',
    kind: 'automática',
    kindLabel: 'Log',
    category: 'a-conferir',
    file: { name, path: `x/${name}`, size: 100 },
    notes: { proof: 'Os testes do quadro passam.', excerpt: 'as linhas `test a ... ok` e `nada`.', failure: '`panicked` ou `FAILED`.' },
    ...rest,
    review: { name, kind: 'automática', status: 'a conferir', reason: '', tool: 'pnpm', reproduce: 'pnpm exec vitest run', how: '', ...review },
  };
}

function open(current: EvidenceEntry, entries: EvidenceEntry[] = [current]) {
  const onNavigate = vi.fn();
  const onClose = vi.fn();
  const view = render(<EvidenceReview entries={entries} current={current} onNavigate={onNavigate} onClose={onClose} />);
  return { ...view, onNavigate, onClose };
}

const explanation = () => screen.getByRole('region', { name: 'Explicação' });
const evidence = () => screen.getByRole('region', { name: 'Evidência' });
const sectionTitles = () => within(explanation()).getAllByRole('heading', { level: 3 }).map((heading) => heading.textContent);
const checks = (name: string) => within(screen.getByRole('list', { name })).getAllByRole('button');

describe('EvidenceReview', () => {
  it('abre com a explicação na ordem do RF-08 e o log à direita', async () => {
    mockFiles();
    open(entry());
    const dialog = screen.getByRole('dialog', { name: 'Testes do quadro passam' });
    expect(within(dialog).getByText('EV-01')).toBeTruthy();
    expect(within(dialog).getByText('A conferir')).toBeTruthy();
    expect(sectionTitles()).toEqual(['O que prova', 'Como foi feita', 'Como reproduzir', 'Trecho esperado', 'Sinais de falha']);
    expect(await within(evidence()).findByText('test b ... ok')).toBeTruthy();
    cleanup();

    mockFiles();
    open(entry({ category: 'conferidas', review: { status: 'conferida', reviewed_at: '2020-01-02', reason: 'ok' } }));
    expect(sectionTitles().at(-1)).toBe('Conferência');
    expect(within(explanation()).getByText('Conferida em 02/01/2020 · Aval: “ok”')).toBeTruthy();
    cleanup();

    mockFiles();
    open(entry({ category: 'recusadas', review: { status: 'recusada', reviewed_at: '2020-01-03', reason: 'falta o total' } }));
    expect(sectionTitles().at(-1)).toBe('Recusa');
    expect(within(explanation()).getByText(/Motivo: “falta o total”/)).toBeTruthy();
  });

  it('mostra Como ler antes do Trecho no print e no vídeo', () => {
    mockFiles();
    for (const [name, kindLabel] of [['tela.png', 'Print'], ['a.webm', 'Vídeo']]) {
      open(entry({ name, kind: 'print', kindLabel, notes: { excerpt: 'o quadro de teste colorido', failure: 'a tela em branco' } }));
      const titles = sectionTitles();
      expect(titles.indexOf('Como ler')).toBeGreaterThan(-1);
      expect(titles.indexOf('Como ler')).toBeLessThan(titles.indexOf('Trecho esperado'));
      expect(within(explanation()).getByText('o quadro de teste colorido')).toBeTruthy();
      expect(within(explanation()).getByText('a tela em branco')).toBeTruthy();
      cleanup();
    }
  });

  it('mostra a ferramenta, o tipo, a frase do Como e o comando com cópia', () => {
    mockFiles();
    open(entry({ review: { how: 'roda os testes da página' } }));
    const side = explanation();
    expect(within(side).getByText('pnpm')).toBeTruthy();
    expect(within(side).getByText('automática')).toBeTruthy();
    expect(within(side).getByText('roda os testes da página')).toBeTruthy();
    expect(within(side).getByText('pnpm exec vitest run')).toBeTruthy();
    expect(within(side).getByRole('button', { name: 'Copiar comando' })).toBeTruthy();
  });

  it('marca cada texto do Trecho e da Falha pela expectativa', async () => {
    mockFiles();
    open(entry());
    await screen.findByText('apareceu · 1 linha');
    const [found, missing] = checks('Trecho esperado');
    expect(found.textContent).toContain('apareceu · 1 linha');
    expect(found.getAttribute('data-tone')).toBe('good');
    expect(missing.textContent).toContain('não apareceu');
    expect(missing.getAttribute('data-tone')).toBe('bad');
    const [panicked, failed] = checks('Sinais de falha');
    expect(panicked.textContent).toContain('não apareceu');
    expect(panicked.getAttribute('data-tone')).toBe('good');
    expect(failed.textContent).toContain('apareceu · 2 linhas');
    expect(failed.getAttribute('data-tone')).toBe('bad');
  });

  it('destaca as linhas do texto clicado e tira ao clicar de novo', async () => {
    mockFiles();
    const { container } = open(entry());
    await screen.findByText('apareceu · 2 linhas');
    const failed = checks('Sinais de falha')[1];
    fireEvent.click(failed);
    expect(failed.getAttribute('aria-pressed')).toBe('true');
    const hits = () => [...container.querySelectorAll(`.${styles.hit}`)].map((line) => line.getAttribute('data-line'));
    expect(hits()).toEqual(['7', '8']);
    fireEvent.click(failed);
    expect(failed.getAttribute('aria-pressed')).toBe('false');
    expect(hits()).toEqual([]);
  });

  it('destaca as linhas 83, 86, 87 e 90 da EV-02 da SPEC-049 TASK-09', async () => {
    const real: Record<number, string> = {
      1: '$ pnpm --dir apps/painel exec vitest run src/features/task/task-page.test.tsx --reporter=verbose',
      2: '# data: 2026-11-11T19:46:19Z',
      3: '# commit: 8047718 (com mudanças não commitadas)',
      4: '# código de saída: 0',
      83: ' ✓ src/features/task/task-page.test.tsx > Barra da onda > abre a próxima com → e a anterior com ← 62ms',
      86: ' ✓ src/features/task/task-page.test.tsx > Próxima a conferir > pula para a próxima tarefa da onda com evidência a conferir 22ms',
      87: ' ✓ src/features/task/task-page.test.tsx > Próxima a conferir > avisa e fica na tarefa quando nenhuma tem evidência a conferir 21ms',
      90: ' Test Files  1 passed (1)',
    };
    const log = Array.from({ length: 94 }, (_, index) => real[index + 1] ?? `linha ${index + 1}`).join('\n');
    mockFiles(log);
    const excerpt = 'as linhas `✓ ... Barra da onda > abre a próxima com → e a anterior com ←`, `✓ ... Próxima a conferir > pula para a próxima tarefa da onda com evidência a conferir`, `... > avisa e fica na tarefa quando nenhuma tem evidência a conferir` e `Test Files  1 passed`.';
    const { container } = open(entry({ name: 'barra-da-onda.txt', id: 'EV-02', notes: { excerpt, failure: '`×`' } }));
    await waitFor(() => expect(screen.getAllByText('apareceu · 1 linha')).toHaveLength(4));
    const numbers = checks('Trecho esperado').map((item) => {
      fireEvent.click(item);
      return [...container.querySelectorAll(`.${styles.hit} .${styles.lineNumber}`)].map((number) => number.textContent);
    });
    expect(numbers).toEqual([['83'], ['86'], ['87'], ['90']]);
  });

  it('mostra Não se aplica com o motivo no print e na Falha vazia do diff', () => {
    mockFiles();
    open(entry({ name: 'tela.png', kind: 'print', kindLabel: 'Print', notes: { excerpt: 'o quadro', failure: 'tela branca' } }));
    expect(within(explanation()).getAllByText('Não se aplica.')).toHaveLength(2);
    expect(within(explanation()).getByText('Prints e vídeos não têm texto para procurar: confira olhando, como diz "Como ler".')).toBeTruthy();
    expect(within(explanation()).getByText('Prints e vídeos não têm texto para procurar: a falha é o que "Como ler" descreve não aparecer.')).toBeTruthy();
    cleanup();

    mockFiles('+linha nova');
    open(entry({ name: 'a.diff', kind: 'diff', kindLabel: 'Diff', notes: { excerpt: 'a linha `+linha nova`', failure: '' } }));
    expect(within(explanation()).getAllByText('Não se aplica.')).toHaveLength(1);
    expect(within(explanation()).getByText('Num diff, a falha é a mudança esperada não aparecer, e o Trecho esperado já confere isso.')).toBeTruthy();
  });

  it('navega só pelas evidências recebidas com os botões e as setas', () => {
    mockFiles();
    const first = entry({ name: 'a.txt', id: 'EV-01', title: 'A' });
    const third = entry({ name: 'c.txt', id: 'EV-03', title: 'C' });
    const { onNavigate, rerender } = open(first, [first, third]);
    expect(screen.getByText('1 de 2')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Anterior (←)' }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Próxima (→)' }));
    expect(onNavigate).toHaveBeenLastCalledWith('c.txt');
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'ArrowRight' });
    expect(onNavigate).toHaveBeenCalledTimes(2);
    rerender(<EvidenceReview entries={[first, third]} current={third} onNavigate={onNavigate} onClose={vi.fn()} />);
    expect(screen.getByText('2 de 2')).toBeTruthy();
    expect((screen.getByRole('button', { name: 'Próxima (→)' }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'ArrowLeft' });
    expect(onNavigate).toHaveBeenLastCalledWith('a.txt');
  });

  it('abre a primeira recebida pela próxima quando a aberta saiu do filtro', () => {
    mockFiles();
    const first = entry({ name: 'a.txt', title: 'A' });
    const outside = entry({ name: 'b.txt', title: 'B' });
    const { onNavigate } = open(outside, [first]);
    expect(screen.queryByText(/ de 1/)).toBeNull();
    expect((screen.getByRole('button', { name: 'Anterior (←)' }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole('button', { name: 'Próxima (→)' }));
    expect(onNavigate).toHaveBeenCalledWith('a.txt');
  });

  it('fecha pelo Fechar e pelo Esc', () => {
    mockFiles();
    const { onClose } = open(entry());
    fireEvent.click(screen.getByRole('button', { name: 'Fechar (Esc)' }));
    fireEvent.keyDown(screen.getByRole('dialog'), { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('troca para a aba Evidência ao destacar um texto no celular', async () => {
    vi.stubGlobal('matchMedia', vi.fn(() => ({ matches: true })));
    mockFiles();
    open(entry());
    await screen.findByText('apareceu · 1 linha');
    fireEvent.click(checks('Trecho esperado')[0]);
    expect(screen.getByRole('tab', { name: 'Evidência' }).getAttribute('aria-selected')).toBe('true');
  });

  it('mostra a imagem no visualizador, o vídeo no player e dá o link aos demais', () => {
    const calls = mockFiles();
    open(entry({ name: 'tela.png', kind: 'print', kindLabel: 'Print' }));
    expect(within(evidence()).getByRole('button', { name: 'Ampliar Evidência tela.png' })).toBeTruthy();
    cleanup();
    open(entry({ name: 'a.mp4', kind: 'print', kindLabel: 'Vídeo' }));
    expect(within(evidence()).getByRole('group', { name: 'Vídeo a.mp4' })).toBeTruthy();
    cleanup();
    open(entry({ name: 'pacote.zip', kind: 'manual', kindLabel: 'Manual' }));
    const link = within(evidence()).getByRole('link', { name: 'Baixar pacote.zip' });
    expect(link.hasAttribute('download')).toBe(true);
    expect(calls).toHaveLength(0);
  });

  it('pede só o primeiro 1 MB e avisa quando há mais', async () => {
    const calls = mockFiles(() => new Response('linha', { status: 206, headers: { 'Content-Range': `bytes 0-${MB - 1}/${3 * MB}` } }));
    open(entry({ name: 'grande.log', file: { name: 'grande.log', path: 'x/grande.log', size: 3 * MB } }));
    expect(await screen.findByText(/Mostrando o primeiro 1 MB de 3,0 MB\./)).toBeTruthy();
    expect(screen.getByRole('link', { name: 'Baixar o arquivo inteiro' })).toBeTruthy();
    expect(calls).toHaveLength(1);
    expect((calls[0].init?.headers as Record<string, string>).Range).toBe(`bytes=0-${MB - 1}`);
  });

  it('não pede o conteúdo de um texto vazio', () => {
    const calls = mockFiles();
    open(entry({ file: { name: 'saida.txt', path: 'x/saida.txt', size: 0 } }));
    expect(screen.queryByText('Carregando…')).toBeNull();
    expect(calls).toHaveLength(0);
  });

  it('mostra a falha ao ler o texto', async () => {
    mockFiles(() => new Response('erro', { status: 500 }));
    open(entry());
    expect(await screen.findByText('Não foi possível ler o arquivo.')).toBeTruthy();
  });

  it('mostra Falta anexar na manual sem anexo', () => {
    const calls = mockFiles();
    const manual = { name: 'foto.png', path: '', size: 0, description: 'Foto do aparelho.', extension: 'png', attached: false };
    open(entry({ name: 'foto.png', kind: 'manual', kindLabel: 'Manual', file: undefined, manual }));
    expect(within(evidence()).getByText('Falta anexar: Foto do aparelho.')).toBeTruthy();
    expect(calls).toHaveLength(0);
  });

  it('copia o log inteiro com o cabeçalho', async () => {
    const writeText = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    mockFiles();
    open(entry());
    fireEvent.click(await screen.findByRole('button', { name: 'Copiar saida.txt' }));
    expect(writeText).toHaveBeenCalledWith(LOG);
  });

  it('mostra o código de saída e ignora os códigos ANSI na exibição', async () => {
    mockFiles('$ cmd\n# data: d\n# commit: c\n# código de saída: 1\n\u001b[32mtudo certo\u001b[0m');
    open(entry({ review: { status: 'a conferir', reviewed_at: '' } }));
    expect(await within(evidence()).findByText('tudo certo')).toBeTruthy();
    expect(within(evidence()).getByText('código de saída 1')).toBeTruthy();
    expect(evidence().textContent).not.toContain('\u001b');
  });
});
