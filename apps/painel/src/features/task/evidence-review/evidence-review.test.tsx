import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { acceptEvidence, rejectEvidence, withdrawReview, type EvidenceReview as Review } from '../../../shared/api';
import type { EvidenceEntry } from '../evidence-list/evidence-items';
import { EvidenceReview } from './evidence-review';
import styles from './evidence-review.module.css';

vi.mock('../../../shared/api', async (original) => ({
  ...(await original<typeof import('../../../shared/api')>()),
  acceptEvidence: vi.fn(),
  rejectEvidence: vi.fn(),
  withdrawReview: vi.fn(),
}));

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

function open(current: EvidenceEntry, entries: EvidenceEntry[] = [current], taskStatus = 'executed') {
  const onNavigate = vi.fn();
  const onClose = vi.fn();
  const view = render(
    <EvidenceReview spec="SPEC-001" task="TASK-02" taskStatus={taskStatus} entries={entries} current={current} onNavigate={onNavigate} onClose={onClose} />,
  );
  return { ...view, onNavigate, onClose };
}

const explanation = () => screen.getByRole('region', { name: 'Explicação' });
const evidence = () => screen.getByRole('region', { name: 'Evidência' });
const sectionTitles = () => within(explanation()).getAllByRole('heading', { level: 3 }).map((heading) => heading.textContent);
const checks = (name: string) => within(screen.getByRole('list', { name })).getAllByRole('button');

describe('EvidenceReview', () => {
  it('mostra a Conferência e a Recusa depois dos blocos da explicação', async () => {
    mockFiles();
    open(entry());
    const dialog = screen.getByRole('dialog', { name: 'Testes do quadro passam' });
    expect(within(dialog).getByText('EV-01')).toBeTruthy();
    expect(within(dialog).getByText('A conferir')).toBeTruthy();
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

  it('mostra a marca de conferida sob o trecho anterior separada do aval', () => {
    mockFiles();
    open(
      entry({
        category: 'conferidas',
        review: { status: 'conferida', reviewed_at: '2020-01-02', reason: 'bate com o log · conferida sob o trecho anterior (ajuste 2)' },
      }),
    );
    expect(within(explanation()).getByText(/Aval: “bate com o log”$/)).toBeTruthy();
    expect(within(explanation()).getByText('Conferida sob o trecho anterior (ajuste 2): o Trecho ou a Falha mudaram depois desta conferência.')).toBeTruthy();
  });

  it('abre por O que mostra, Onde olhar e Falha, com o resto recolhido', () => {
    mockFiles();
    open(entry({ notes: { shows: 'o resumo dos testes', whereToLook: 'nas linhas do resumo', covers: 'RF-13, CT-11', reproduce: '`pnpm exec vitest run`', excerpt: '`test a ... ok`', failure: 'se aparecer `FAILED`, então o teste quebrou' } }));
    expect(sectionTitles()).toEqual(['O que mostra', 'Onde olhar', 'Trecho esperado', 'Sinais de falha']);
    for (const title of ['Como foi feita', 'Reproduzir', 'Cobre']) {
      const details = within(explanation()).getByText(title, { selector: 'summary' }).closest('details');
      expect(details).toBeTruthy();
      expect(details?.hasAttribute('open')).toBe(false);
    }
    expect(within(explanation()).getByText('pnpm exec vitest run')).toBeTruthy();
    expect(within(explanation()).getByText('RF-13')).toBeTruthy();
    expect(within(explanation()).getByText('CT-11')).toBeTruthy();
  });

  it('mostra Mostra e deixa a Prova recolhida quando as duas existem', () => {
    mockFiles();
    open(entry({ notes: { shows: 'o que aparece', proof: 'a prova longa' } }));
    expect(within(explanation()).getByText('o que aparece')).toBeTruthy();
    const details = within(explanation()).getByText('Prova', { selector: 'summary' }).closest('details');
    expect(details?.hasAttribute('open')).toBe(false);
    expect(details?.textContent).toContain('a prova longa');
  });

  it('abre pela Prova e pelo Trecho no formato atual', () => {
    mockFiles();
    open(entry());
    expect(sectionTitles()).toEqual(['O que mostra', 'Trecho esperado', 'Sinais de falha']);
    expect(within(explanation()).getByText('Os testes do quadro passam.')).toBeTruthy();
    expect(within(explanation()).queryByText('Prova', { selector: 'summary' })).toBeNull();
  });

  it('não mostra Não se aplica no print, no vídeo nem no diff', () => {
    mockFiles();
    open(entry({ name: 'tela.png', kind: 'print', kindLabel: 'Print', notes: { shows: 'a tela', whereToLook: 'no centro', failure: 'a tela em branco' } }));
    expect(sectionTitles()).toEqual(['O que mostra', 'Onde olhar', 'Falha']);
    expect(explanation().textContent).not.toContain('Não se aplica');
    cleanup();
    open(entry({ name: 'a.webm', kind: 'print', kindLabel: 'Vídeo', notes: { shows: 'o vídeo', failure: 'sem imagem' } }));
    expect(explanation().textContent).not.toContain('Não se aplica');
    cleanup();
    mockFiles('+linha nova');
    open(entry({ name: 'a.diff', kind: 'diff', kindLabel: 'Diff', notes: { excerpt: 'a linha `+linha nova`', failure: '' } }));
    expect(explanation().textContent).not.toContain('Não se aplica');
  });

  it('mostra o Trecho do print como Onde olhar quando a tarefa não o escreveu', () => {
    mockFiles();
    open(entry({ name: 'tela.png', kind: 'print', kindLabel: 'Print', notes: { excerpt: 'o quadro de teste', failure: 'a tela em branco' } }));
    const block = (title: string) => within(explanation()).getByRole('heading', { name: title }).closest('section') as HTMLElement;
    expect(within(block('Onde olhar')).getByText('o quadro de teste')).toBeTruthy();
    expect(within(block('Falha')).getByText('a tela em branco')).toBeTruthy();
  });

  it('recolhe Como foi feita, Reproduzir e Cobre e abre ao clicar', () => {
    mockFiles();
    open(entry({ review: { how: 'roda os testes' }, notes: { shows: 'x', covers: 'RF-01' } }));
    for (const title of ['Como foi feita', 'Reproduzir', 'Cobre']) {
      const summary = within(explanation()).getByText(title, { selector: 'summary' });
      const details = summary.closest('details') as HTMLDetailsElement;
      expect(details.open).toBe(false);
      fireEvent.click(summary);
      expect(details.open).toBe(true);
    }
    cleanup();
    open(entry({ review: { reproduce: '' }, notes: { shows: 'x' } }));
    expect(within(explanation()).queryByText('Reproduzir', { selector: 'summary' })).toBeNull();
    expect(within(explanation()).queryByText('Cobre', { selector: 'summary' })).toBeNull();
  });

  it('pinta o trecho de verde e a falha de vermelho no log', async () => {
    mockFiles();
    const { container } = open(entry());
    await screen.findByText('apareceu · 1 linha');
    const lines = (cls: string) => [...container.querySelectorAll(`.${cls}`)].map((line) => line.getAttribute('data-line'));
    expect(lines(styles.good)).toEqual(['4']);
    expect(lines(styles.bad)).toEqual(['7', '8']);
  });

  it('pinta de vermelho a linha que é trecho e falha ao mesmo tempo', async () => {
    mockFiles('$ c\n# data: d\n# commit: c\n# código de saída: 0\nresultado: ok FAILED');
    const { container } = open(entry({ notes: { excerpt: '`resultado: ok`', failure: '`FAILED`' } }));
    await waitFor(() => expect(container.querySelectorAll(`.${styles.bad}`)).toHaveLength(1));
    expect(container.querySelector(`.${styles.bad}`)?.getAttribute('data-line')).toBe('4');
    expect(container.querySelectorAll(`.${styles.good}`)).toHaveLength(0);
  });

  it('não pinta o trecho que não apareceu nem a falha ausente', async () => {
    mockFiles();
    const { container } = open(entry({ notes: { excerpt: '`nada`', failure: '`panicked`' } }));
    await within(evidence()).findByText('test b ... ok');
    expect(container.querySelectorAll(`.${styles.good}`)).toHaveLength(0);
    expect(container.querySelectorAll(`.${styles.bad}`)).toHaveLength(0);
  });

  it('leva o vídeo ao tempo clicado no Onde olhar', () => {
    mockFiles();
    const { container } = open(entry({ name: 'a.webm', kind: 'print', kindLabel: 'Vídeo', notes: { shows: 'o vídeo', whereToLook: 'aos 0:05 abre a janela e aos 1:10 aparece o aviso' } }));
    const video = container.querySelector('video') as HTMLVideoElement;
    fireEvent.click(screen.getByRole('button', { name: '0:05' }));
    expect(video.currentTime).toBe(5);
    fireEvent.click(screen.getByRole('button', { name: '1:10' }));
    expect(video.currentTime).toBe(70);
  });

  it('deixa sem link o tempo fora do formato m:ss e o texto fora de vídeo', () => {
    mockFiles();
    open(entry({ name: 'a.webm', kind: 'print', kindLabel: 'Vídeo', notes: { shows: 'v', whereToLook: 'perto de 5 segundos e em 0:5' } }));
    expect(within(explanation()).queryAllByRole('button', { name: /^\d+:\d+$/ })).toHaveLength(0);
    expect(within(explanation()).getByText('perto de 5 segundos e em 0:5')).toBeTruthy();
    cleanup();
    open(entry({ name: 'tela.png', kind: 'print', kindLabel: 'Print', notes: { shows: 'p', whereToLook: 'aos 0:05' } }));
    expect(within(explanation()).queryAllByRole('button', { name: /^\d+:\d+$/ })).toHaveLength(0);
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
    rerender(<EvidenceReview spec="SPEC-001" task="TASK-02" taskStatus="executed" entries={[first, third]} current={third} onNavigate={onNavigate} onClose={vi.fn()} />);
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

describe('Ações da conferência', () => {
  const ok = { ok: true } as const;
  const toReview = (name: string, over: Parameters<typeof entry>[0] = {}) =>
    entry({ name, id: `EV-${name}`, title: name, ...over, review: { sha256: 'h1', ...over.review } });
  const first = toReview('a.txt');
  const done = toReview('b.txt', { category: 'conferidas', review: { status: 'conferida' } });
  const last = toReview('c.txt');
  const popover = (name: string) => screen.getByRole('dialog', { name });
  const field = (name: string) => within(popover(name)).getByRole('textbox') as HTMLTextAreaElement;
  const type = (name: string, value: string) => fireEvent.input(field(name), { target: { value } });
  const confirm = (name: string, label: string) => fireEvent.click(within(popover(name)).getByRole('button', { name: label }));

  beforeEach(() => {
    vi.mocked(acceptEvidence).mockReset().mockResolvedValue(ok);
    vi.mocked(rejectEvidence).mockReset().mockResolvedValue(ok);
    vi.mocked(withdrawReview).mockReset().mockResolvedValue(ok);
  });

  it('mostra Recusar e Aceitar na evidência a conferir e Retirar na decidida', () => {
    mockFiles();
    open(first);
    expect(screen.getByRole('button', { name: 'Recusar' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Aceitar' })).toBeTruthy();
    cleanup();
    open(done);
    expect(screen.getByRole('button', { name: 'Retirar conferência' })).toBeTruthy();
    cleanup();
    open(toReview('d.txt', { category: 'recusadas', review: { status: 'recusada', reason: 'falta' } }));
    expect(screen.getByRole('button', { name: 'Retirar recusa' })).toBeTruthy();
    cleanup();
    for (const item of [toReview('e.txt', { category: 'conferidas', review: { status: 'sem aval' } }), toReview('f.txt', { id: '' })]) {
      open(item);
      expect(screen.queryByRole('button', { name: /^(Recusar|Aceitar|Retirar)/ })).toBeNull();
      cleanup();
    }
  });

  it('esconde as ações na tarefa concluída', () => {
    mockFiles();
    open(first, [first], 'done');
    expect(screen.queryByRole('button', { name: /^(Recusar|Aceitar|Retirar)/ })).toBeNull();
  });

  it('aceita com observação e abre a próxima a conferir do filtro', async () => {
    mockFiles();
    const { onNavigate } = open(first, [first, done, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Aceitar' }));
    expect(document.activeElement).toBe(field('Aceitar a evidência'));
    expect(field('Aceitar a evidência').getAttribute('aria-label')).toBe('Observação');
    type('Aceitar a evidência', 'ok');
    confirm('Aceitar a evidência', 'Aceitar');
    await waitFor(() => expect(onNavigate).toHaveBeenCalledWith('c.txt'));
    expect(acceptEvidence).toHaveBeenCalledWith('SPEC-001', 'TASK-02', 'EV-a.txt', 'h1', 'ok');
  });

  it('aceita sem observação', async () => {
    mockFiles();
    open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Aceitar' }));
    confirm('Aceitar a evidência', 'Aceitar');
    await waitFor(() => expect(acceptEvidence).toHaveBeenCalledWith('SPEC-001', 'TASK-02', 'EV-a.txt', 'h1', ''));
  });

  it('não recusa sem motivo', () => {
    mockFiles();
    open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    confirm('Recusar a evidência', 'Recusar');
    expect(rejectEvidence).not.toHaveBeenCalled();
    expect(field('Recusar a evidência').getAttribute('aria-label')).toBe('Motivo da recusa');
    expect(field('Recusar a evidência').getAttribute('aria-invalid')).toBe('true');
    expect(within(popover('Recusar a evidência')).getByRole('alert').textContent).toBe('Obrigatório');
  });

  it('recusa com motivo e abre a próxima', async () => {
    mockFiles();
    const { onNavigate } = open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    type('Recusar a evidência', 'falta o total');
    confirm('Recusar a evidência', 'Recusar');
    await waitFor(() => expect(onNavigate).toHaveBeenCalledWith('c.txt'));
    expect(rejectEvidence).toHaveBeenCalledWith('SPEC-001', 'TASK-02', 'EV-a.txt', 'h1', 'falta o total');
  });

  it('retira a conferência com motivo e fica na evidência', async () => {
    mockFiles();
    const { onNavigate } = open(done, [first, done, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Retirar conferência' }));
    confirm('Retirar a decisão', 'Retirar');
    expect(withdrawReview).not.toHaveBeenCalled();
    type('Retirar a decisão', 'engano');
    confirm('Retirar a decisão', 'Retirar');
    expect(await screen.findByText('Decisão retirada: a evidência volta para A conferir.')).toBeTruthy();
    expect(withdrawReview).toHaveBeenCalledWith('SPEC-001', 'TASK-02', 'EV-b.txt', 'h1', 'engano');
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('avisa quando não há outra a conferir e fica na evidência aceita', async () => {
    mockFiles();
    const { onNavigate } = open(first, [first, done]);
    fireEvent.click(screen.getByRole('button', { name: 'Aceitar' }));
    confirm('Aceitar a evidência', 'Aceitar');
    const notice = await screen.findByText('Não há outra evidência a conferir neste filtro.');
    expect(notice.getAttribute('role')).toBe('status');
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('recarrega a evidência quando o arquivo mudou', async () => {
    const message = 'A evidência mudou ou a tarefa não está em conferência: recarregue a página.';
    vi.mocked(acceptEvidence).mockResolvedValue({ ok: false, status: 409, message });
    const calls = mockFiles();
    open(first, [first, last]);
    await waitFor(() => expect(calls).toHaveLength(1));
    fireEvent.click(screen.getByRole('button', { name: 'Aceitar' }));
    confirm('Aceitar a evidência', 'Aceitar');
    expect((await screen.findByRole('alert')).textContent).toBe(message);
    expect(screen.queryByRole('dialog', { name: 'Aceitar a evidência' })).toBeNull();
    await waitFor(() => expect(calls).toHaveLength(2));
  });

  it('mostra o erro do servidor na caixa', async () => {
    vi.mocked(rejectEvidence).mockResolvedValue({ ok: false, status: 403, message: 'O painel recusou a gravação: recarregue a página.' });
    mockFiles();
    const { onNavigate } = open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    type('Recusar a evidência', 'falta');
    confirm('Recusar a evidência', 'Recusar');
    const alert = await within(popover('Recusar a evidência')).findByRole('alert');
    expect(alert.textContent).toBe('O painel recusou a gravação: recarregue a página.');
    const button = within(popover('Recusar a evidência')).getByRole('button', { name: 'Recusar' }) as HTMLButtonElement;
    expect(button.disabled).toBe(false);
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('não navega com as setas no campo da caixa', () => {
    mockFiles();
    const { onNavigate } = open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Aceitar' }));
    fireEvent.keyDown(field('Aceitar a evidência'), { key: 'ArrowRight' });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it('o Esc fecha só a caixa', () => {
    mockFiles();
    const { onClose } = open(first, [first, last]);
    fireEvent.click(screen.getByRole('button', { name: 'Recusar' }));
    fireEvent.keyDown(field('Recusar a evidência'), { key: 'Escape' });
    expect(screen.queryByRole('dialog', { name: 'Recusar a evidência' })).toBeNull();
    expect(screen.getByRole('dialog', { name: 'a.txt' })).toBeTruthy();
    expect(onClose).not.toHaveBeenCalled();
  });
});
