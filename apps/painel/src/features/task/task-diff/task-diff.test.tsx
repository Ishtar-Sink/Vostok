import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { contrastRatio } from '../../../theme';
import { dark, light } from '../../../theme/tokens';
import { TaskDiff, wordMarks } from './task-diff';

const changes = (over: object = {}) => ({
  base: 'abc1234',
  frozen: false,
  reason: null,
  added: 4,
  removed: 1,
  files: [
    { path: 'apps/x.ts', action: 'alterado', added: 3, removed: 1, hunks: ['@@ -1,3 +1,4 @@\n linha comum\n-linha velha\n+linha nova\n+linha extra\n fim'] },
    { path: 'apps/novo.ts', action: 'criado', added: 1, removed: 0, hunks: ['@@ -0,0 +1 @@\n+export const novo = 1;'] },
  ],
  ...over,
});

function stubFetch(...bodies: object[]) {
  let call = 0;
  const fetchMock = vi.fn(() => Promise.resolve(new Response(JSON.stringify(bodies[Math.min(call++, bodies.length - 1)]))));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
}

const view = (status = 'in_progress') => <TaskDiff spec="SPEC-001" task="TASK-01" status={status} version={0} />;

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  vi.useRealTimers();
  localStorage.clear();
});

describe('TaskDiff', () => {
  it('mostra o resumo de linhas e um bloco recolhido por arquivo', async () => {
    const fetchMock = stubFetch(changes());
    const { container } = render(view());
    await screen.findByText('apps/x.ts');
    const summary = within(container.querySelector('[data-diff-summary]') as HTMLElement);
    expect(summary.getByText('2 arquivos')).toBeTruthy();
    expect(summary.getByText('+4')).toBeTruthy();
    expect(summary.getByText('−1')).toBeTruthy();
    const files = container.querySelectorAll('details[data-diff-file]');
    expect(files.length).toBe(2);
    files.forEach((file) => expect(file.hasAttribute('open')).toBe(false));
    expect(within(files[0].querySelector('summary') as HTMLElement).getByText('alterado')).toBeTruthy();
    expect(fetchMock).toHaveBeenCalledWith('/api/tarefas/SPEC-001/TASK-01/mudancas');
  });

  it('abre lado a lado por padrão', async () => {
    stubFetch(changes());
    const { container } = render(view());
    await screen.findByText('apps/x.ts');
    expect(screen.getByRole('button', { name: 'Lado a lado' }).getAttribute('aria-pressed')).toBe('true');
    const first = container.querySelector('details[data-diff-file]') as HTMLDetailsElement;
    first.setAttribute('open', '');
    const old = screen.getByText('linha velha');
    const added = screen.getByText('linha nova');
    expect(old.closest('tr')).toBe(added.closest('tr'));
    expect(old.closest('td')).not.toBe(added.closest('td'));
  });

  it('troca para unificada e lembra a escolha depois de recarregar', async () => {
    stubFetch(changes());
    render(view());
    await screen.findByText('apps/x.ts');
    fireEvent.click(screen.getByRole('button', { name: 'Unificada' }));
    expect(screen.getByRole('button', { name: 'Unificada' }).getAttribute('aria-pressed')).toBe('true');
    expect(localStorage.getItem('vostok.diff.formato')).toBe('unificada');
    cleanup();
    render(view());
    await screen.findByText('apps/x.ts');
    expect(screen.getByRole('button', { name: 'Unificada' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('troca de formato sem armazenamento local, sem lembrar', async () => {
    stubFetch(changes());
    vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => { throw new Error('bloqueado'); });
    vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('bloqueado'); });
    render(view());
    await screen.findByText('apps/x.ts');
    expect(screen.getByRole('button', { name: 'Lado a lado' }).getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(screen.getByRole('button', { name: 'Unificada' }));
    expect(screen.getByRole('button', { name: 'Unificada' }).getAttribute('aria-pressed')).toBe('true');
    cleanup();
    render(view());
    await screen.findByText('apps/x.ts');
    expect(screen.getByRole('button', { name: 'Lado a lado' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('explica a falta de base de comparação sem mostrar diff', async () => {
    stubFetch(changes({ base: null, reason: 'sem_base', files: [] }));
    const { container } = render(view());
    await screen.findByText('Sem base de comparação');
    expect(container.querySelector('details[data-diff-file]')).toBeNull();
  });

  it('diz que a tarefa não começou', async () => {
    stubFetch(changes({ base: null, reason: 'nao_iniciada', files: [] }));
    render(view());
    await screen.findByText('A tarefa ainda não começou');
  });

  it('diz que nada mudou desde a base', async () => {
    stubFetch(changes({ reason: 'sem_mudancas', files: [] }));
    render(view());
    await screen.findByText('Nada mudou desde a base');
  });

  it('explica que a diff não foi gravada na conclusão', async () => {
    stubFetch(changes({ reason: 'nao_gravada', frozen: true, files: [] }));
    render(view('done'));
    await screen.findByText('A diff não foi gravada');
  });

  it('marca o arquivo que já tinha mudanças antes', async () => {
    const body = changes();
    (body.files[0] as { before?: boolean }).before = true;
    stubFetch(body);
    render(view());
    await screen.findByText('já tinha mudanças antes');
  });

  it('mostra o arquivo binário sem trechos', async () => {
    stubFetch(changes({ files: [{ path: 'img.png', action: 'binário', added: 0, removed: 0, hunks: [] }] }));
    const { container } = render(view());
    await screen.findByText('img.png');
    (container.querySelector('details[data-diff-file]') as HTMLDetailsElement).setAttribute('open', '');
    expect(screen.getByText('Arquivo binário: sem trechos para mostrar.')).toBeTruthy();
  });

  it('mostra o erro da rota com Tentar de novo', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(new Response('{}', { status: 500 }))));
    render(view());
    await screen.findByText('Não foi possível carregar as mudanças');
    expect(screen.getByRole('button', { name: 'Tentar de novo' })).toBeTruthy();
  });

  it('pede as mudanças de novo a cada 5 s fora de done', async () => {
    vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval'] });
    const second = changes();
    second.files.push({ path: 'apps/outro.ts', action: 'criado', added: 1, removed: 0, hunks: [] });
    const fetchMock = stubFetch(changes(), second);
    render(view());
    await screen.findByText('apps/x.ts');
    await act(async () => { vi.advanceTimersByTime(5000); });
    await screen.findByText('apps/outro.ts');
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('não pede de novo numa tarefa concluída', async () => {
    vi.useFakeTimers({ toFake: ['setInterval', 'clearInterval'] });
    const fetchMock = stubFetch(changes({ frozen: true }));
    render(view('done'));
    await screen.findByText('Congelada na conclusão');
    await act(async () => { vi.advanceTimersByTime(15000); });
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(1));
  });

  it('linhas acrescentadas e removidas têm contraste AA nos dois temas', () => {
    for (const t of [light, dark]) {
      expect(contrastRatio(t.text, t.successSoft)).toBeGreaterThanOrEqual(4.5);
      expect(contrastRatio(t.text, t.dangerSoft)).toBeGreaterThanOrEqual(4.5);
    }
  });
});

describe('colunas da diff', () => {
  const longLine = 'x'.repeat(400);
  const twoHunks = changes({
    files: [
      {
        path: 'apps/x.ts',
        action: 'alterado',
        added: 2,
        removed: 1,
        hunks: [`@@ -1,2 +1,2 @@\n-${longLine}\n+curta\n fim`, '@@ -40,1 +40,2 @@\n meio\n+nova'],
      },
    ],
  });

  async function openFile() {
    const { container } = render(view());
    await screen.findByText('apps/x.ts');
    (container.querySelector('details[data-diff-file]') as HTMLDetailsElement).setAttribute('open', '');
    return container;
  }

  const columns = (table: Element) => Array.from(table.querySelectorAll('colgroup > col')).map((col) => col.getAttribute('data-col'));

  it('lado a lado tem as mesmas seis colunas fixas em todos os trechos', async () => {
    stubFetch(twoHunks);
    const container = await openFile();
    const tables = container.querySelectorAll('table');
    expect(tables).toHaveLength(2);
    for (const table of tables) {
      expect(columns(table)).toEqual(['number', 'sign', 'code', 'number', 'sign', 'code']);
    }
  });

  it('unificada tem as mesmas quatro colunas fixas em todos os trechos', async () => {
    stubFetch(twoHunks);
    const container = await openFile();
    fireEvent.click(screen.getByRole('button', { name: 'Unificada' }));
    const tables = container.querySelectorAll('table');
    expect(tables).toHaveLength(2);
    for (const table of tables) {
      expect(columns(table)).toEqual(['number', 'number', 'sign', 'code']);
    }
  });

  it('a linha longa fica inteira na célula, sem cortar', async () => {
    stubFetch(twoHunks);
    await openFile();
    expect(screen.getByText(longLine).textContent).toHaveLength(400);
  });
});

describe('negrito por palavra', () => {
  const changed = (parts: { text: string; changed: boolean }[]) => parts.filter((part) => part.changed).map((part) => part.text);
  const listHunk =
    '@@ -1,8 +1,8 @@\n const tipos = [criado, alterado];\n-const nomes = [ana, bia];\n+const nomes = [ana, bia, caio];\n-function soma(a, b) {\n-  const total = a + b;\n-  registrar(total);\n-  avisar(total);\n-  return total;\n+export async function carregar(url) {\n+  const resposta = await fetch(url);\n+  if (!resposta.ok) throw new Error(url);\n+  guardar(resposta);\n+  return resposta.json();\n }';

  it('marca só o valor novo numa linha de lista', () => {
    const marks = wordMarks(['const nomes = [ana, bia];'], ['const nomes = [ana, bia, caio];']);
    expect(marks).not.toBeNull();
    expect(changed(marks!.removed[0])).toEqual([]);
    expect(changed(marks!.added[0])).toEqual([', caio']);
    expect(marks!.removed[0].map((part) => part.text).join('')).toBe('const nomes = [ana, bia];');
    expect(marks!.added[0].map((part) => part.text).join('')).toBe('const nomes = [ana, bia, caio];');
  });

  it('troca de uma linha com 2 de 5 palavras em comum tem negrito', () => {
    const marks = wordMarks(['alfa beta gama delta epsilon'], ['alfa beta zeta eta teta']);
    expect(changed(marks!.removed[0])).toEqual(['gama delta epsilon']);
    expect(changed(marks!.added[0])).toEqual(['zeta eta teta']);
  });

  it('troca de uma linha com 1 de 5 palavras em comum fica sem negrito', () => {
    expect(wordMarks(['alfa beta gama delta epsilon'], ['alfa zeta eta teta iota'])).toBeNull();
  });

  it('troca de várias linhas exige 60% em cada par', () => {
    expect(
      wordMarks(['alfa beta gama delta epsilon', 'um dois tres quatro cinco'], ['alfa beta zeta eta teta', 'um dois seis sete oito']),
    ).toBeNull();
    const marks = wordMarks(['alfa beta gama delta epsilon', 'um dois tres quatro cinco'], ['alfa beta gama zeta eta', 'um dois tres seis sete']);
    expect(marks!.removed.map(changed)).toEqual([['delta epsilon'], ['quatro cinco']]);
    expect(marks!.added.map(changed)).toEqual([['zeta eta'], ['seis sete']]);
  });

  it('um par que falha deixa a troca de 2 linhas sem negrito', () => {
    expect(
      wordMarks(['const nomes = [ana, bia];', 'alfa beta gama delta epsilon'], ['const nomes = [ana, bia, caio];', 'alfa zeta eta teta iota']),
    ).toBeNull();
  });

  it('mais de 2 pedaços trocados deixa a linha sem negrito', () => {
    expect(wordMarks(['um dois tres quatro cinco seis sete'], ['um X tres Y cinco Z sete'])).toBeNull();
  });

  it('quantidades diferentes de linhas ficam sem negrito', () => {
    expect(wordMarks(['a b'], ['a b', 'c'])).toBeNull();
    expect(wordMarks([], ['c'])).toBeNull();
  });

  it('bloco de 5 linhas reescrito fica sem negrito', () => {
    const body = listHunk.split('\n').slice(1);
    const removed = body.filter((line) => line.startsWith('-')).slice(1).map((line) => line.slice(1));
    const added = body.filter((line) => line.startsWith('+')).slice(1).map((line) => line.slice(1));
    expect(removed).toHaveLength(5);
    expect(added).toHaveLength(5);
    expect(wordMarks(removed, added)).toBeNull();
  });

  async function openList() {
    stubFetch(changes({ files: [{ path: 'apps/lista.ts', action: 'alterado', added: 6, removed: 6, hunks: [listHunk] }] }));
    const { container } = render(view());
    await screen.findByText('apps/lista.ts');
    (container.querySelector('details[data-diff-file]') as HTMLDetailsElement).setAttribute('open', '');
    return container;
  }

  function expectOnlyNewValueBold(container: Element) {
    const inserted = container.querySelectorAll('ins');
    expect(inserted).toHaveLength(1);
    expect(inserted[0].textContent).toBe(', caio');
    expect(container.querySelectorAll('del')).toHaveLength(0);
    expect(inserted[0].closest('td')?.getAttribute('data-diff')).toBe('add');
  }

  it('lado a lado: negrito só no valor novo da linha da lista', async () => {
    expectOnlyNewValueBold(await openList());
  });

  it('unificada: negrito só no valor novo da linha da lista', async () => {
    const container = await openList();
    fireEvent.click(screen.getByRole('button', { name: 'Unificada' }));
    expect(screen.getByRole('button', { name: 'Unificada' }).getAttribute('aria-pressed')).toBe('true');
    expectOnlyNewValueBold(container);
  });

  it('o negrito tem contraste AA nos dois temas', () => {
    const mix = (a: string, b: string, share: number) => {
      const channels = (hex: string) => [1, 3, 5].map((index) => parseInt(hex.slice(index, index + 2), 16));
      const [ca, cb] = [channels(a), channels(b)];
      return `#${ca.map((value, index) => Math.round(value * share + cb[index] * (1 - share)).toString(16).padStart(2, '0')).join('')}`;
    };
    for (const t of [light, dark]) {
      expect(contrastRatio(t.text, mix(t.success, t.successSoft, 0.22))).toBeGreaterThanOrEqual(4.5);
      expect(contrastRatio(t.text, mix(t.danger, t.dangerSoft, 0.22))).toBeGreaterThanOrEqual(4.5);
    }
  });
});

describe('frases do agente', () => {
  const withNotes = (notes?: (string | null)[]) =>
    changes({
      files: [
        {
          path: 'apps/x.ts',
          action: 'alterado',
          added: 3,
          removed: 1,
          hunks: ['@@ -1,2 +1,3 @@\n-linha velha\n+linha nova\n+linha extra2\n fim', '@@ -10,1 +10,2 @@\n meio\n+linha extra'],
          ...(notes === undefined ? {} : { notes }),
        },
      ],
    });

  async function open(notes?: (string | null)[]) {
    stubFetch(withNotes(notes));
    const { container } = render(view());
    await screen.findByText('apps/x.ts');
    (container.querySelector('details[data-diff-file]') as HTMLDetailsElement).setAttribute('open', '');
    return container;
  }

  const notesOf = (container: Element) => container.querySelectorAll('[data-diff-note]');

  function expectNoteBeforeFirstHunk(container: Element) {
    const found = notesOf(container);
    expect(found).toHaveLength(1);
    const note = found[0];
    expect(note.textContent).toBe('Troca a linha velha pela nova.');
    expect(note.compareDocumentPosition(screen.getByText('linha velha')) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
    expect(note.compareDocumentPosition(screen.getByText('linha extra')) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
  }

  it('mostra a frase acima do trecho no formato lado a lado', async () => {
    expectNoteBeforeFirstHunk(await open(['Troca a linha velha pela nova.', null]));
  });

  it('mostra a frase acima do trecho no formato unificado', async () => {
    localStorage.setItem('vostok.diff.formato', 'unificada');
    const container = await open(['Troca a linha velha pela nova.', null]);
    expect(screen.getByRole('button', { name: 'Unificada' }).getAttribute('aria-pressed')).toBe('true');
    expectNoteBeforeFirstHunk(container);
  });

  it('troca de formato sem perder a frase', async () => {
    const container = await open(['Troca a linha velha pela nova.', null]);
    fireEvent.click(screen.getByRole('button', { name: 'Unificada' }));
    expect(notesOf(container)).toHaveLength(1);
    expect(notesOf(container)[0].textContent).toBe('Troca a linha velha pela nova.');
  });

  it('trecho com null mostra só a diff', async () => {
    const container = await open([null, null]);
    expect(notesOf(container)).toHaveLength(0);
    expect(screen.getByText('linha velha')).toBeTruthy();
    expect(screen.getByText('linha extra')).toBeTruthy();
  });

  it('arquivo sem notes mostra só a diff', async () => {
    const container = await open();
    expect(notesOf(container)).toHaveLength(0);
    expect(screen.getByText('linha velha')).toBeTruthy();
  });

  it('frase vazia ou só com espaços não aparece', async () => {
    expect(notesOf(await open(['', '   ']))).toHaveLength(0);
  });

  it('lista de notes menor que a de trechos mostra só a frase existente', async () => {
    const container = await open(['Só a primeira.']);
    expect(notesOf(container)).toHaveLength(1);
    expect(notesOf(container)[0].textContent).toBe('Só a primeira.');
  });

  it('código entre crases na frase vira code', async () => {
    const container = await open(['Troca `linha velha` por `linha nova`.', null]);
    const note = notesOf(container)[0];
    const codes = note.querySelectorAll('code');
    expect(codes).toHaveLength(2);
    expect(codes[0].textContent).toBe('linha velha');
    expect(codes[1].textContent).toBe('linha nova');
    expect(note.textContent).toBe('Troca linha velha por linha nova.');
  });

  it('crase sem par fica como texto', async () => {
    const note = notesOf(await open(['Usa a crase ` sozinha.', null]))[0];
    expect(note.querySelectorAll('code')).toHaveLength(0);
    expect(note.textContent).toBe('Usa a crase ` sozinha.');
  });

  it('frase com marcação HTML aparece como texto', async () => {
    const note = notesOf(await open(['<b>negrito</b> não vira tag', null]))[0];
    expect(note.querySelectorAll('b')).toHaveLength(0);
    expect(note.textContent).toBe('<b>negrito</b> não vira tag');
  });
});
