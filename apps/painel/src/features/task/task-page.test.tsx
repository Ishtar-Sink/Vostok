import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { fileUrl } from '../../shared/api';
import cardStyles from '../../ui/card/card.module.css';
import { findLines, TaskPage } from './task-page';
import taskStyles from './task-page.module.css';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const state = (over: object = {}) => ({ id: 'TASK-02', title: 'Criar a tela', description: 'Descrição', status: 'in_progress', wave: '2', profile: 'padrao', persona: 'dev', model: 'm', depends_on: 'TASK-01', covers: 'RF-01', approved_at: '', attempts: '0', tests_result: '', regression_result: '', evidence_result: '', ...over });
const taskDetail = (over: object = {}, evidence: object[] = []) => ({
  spec: 'SPEC-001',
  state: state(over),
  files: { executivo: '## Resumo\nTexto executivo.', tecnico: '## T\nTexto técnico.', criterios: '## C\n- [ ] item', execucao: '## E\nPassos.', verificacao: '## V\nComandos.', registro: '## R\nLinhas.' },
  evidence,
  manual_evidence: [],
});
const specDetail = { meta: { id: 'SPEC-001', title: 'Spec', status: 'planned' }, path: 'p', plan: true, markdown: '# S\n', tasks: [{ id: 'TASK-01', title: 'a', status: 'done', wave: '1', profile: 'padrao', covers: '' }] };

function mockFetch(task: unknown, status = 200) {
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task, status) : json(specDetail))));
}
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = function showModal() { this.setAttribute('open', ''); };
  HTMLDialogElement.prototype.close = function close() { this.removeAttribute('open'); };
  globalThis.IntersectionObserver = class { observe() {} disconnect() {} unobserve() {} takeRecords() { return []; } } as never;
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); history.replaceState(null, '', '/'); });
async function openEvidence() { fireEvent.click(await screen.findByRole('tab', { name: 'Evidências' })); }

describe('TaskPage', () => {
  it('divide a tarefa nas abas Resumo, Evidências e Para o agente', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    const tabs = await screen.findAllByRole('tab');
    expect(tabs.map((t) => t.textContent)).toEqual(['Resumo', 'Evidências', 'Para o agente']);
    expect(screen.getByRole('tablist', { name: 'Partes da tarefa' })).toBeTruthy();
    expect(tabs[0].getAttribute('aria-selected')).toBe('true');
    const h2 = () => screen.getAllByRole('heading', { level: 2 }).map((h) => h.textContent);
    expect(h2()).toEqual(['Resumo executivo', 'Detalhes técnicos', 'Critérios de conclusão']);
    fireEvent.click(screen.getByRole('tab', { name: 'Para o agente' }));
    expect(h2()).toEqual(['Execução', 'Verificação', 'Registro de execução']);
    expect(screen.queryByRole('heading', { level: 2, name: 'Resumo executivo' })).toBeNull();
  });

  it('esconde onda, perfil e persona vazios', async () => {
    mockFetch(taskDetail({ wave: '', profile: '', persona: '' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Status');
    expect(screen.queryByText('Onda')).toBeNull();
    expect(screen.queryByText('Perfil')).toBeNull();
    expect(screen.queryByText('Persona')).toBeNull();
    expect(screen.getByText('Status')).toBeTruthy();
    expect(screen.getByText('Modelo')).toBeTruthy();
  });

  it('mostra onda, perfil e persona com valor', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Status');
    expect(screen.getByText('Onda')).toBeTruthy();
    expect(screen.getByText('Perfil')).toBeTruthy();
    expect(screen.getByText('Persona')).toBeTruthy();
  });

  it('shows image evidence', async () => {
    mockFetch(taskDetail({}, [{ name: 'tela.png', path: 'x/tela.png', size: 2048 }]));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    const img = (await screen.findByAltText('Evidência tela.png')) as HTMLImageElement;
    expect(img.getAttribute('src')).toContain('tela.png');
  });

  it('não oferece Editar em nenhuma parte', async () => {
    for (const status of ['in_progress', 'done']) {
      mockFetch(taskDetail({ status }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('Texto executivo.');
      expect(screen.queryByRole('button', { name: 'Editar' })).toBeNull();
      cleanup();
    }
  });

  it('shows the empty evidence note and the error state', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    expect(await screen.findByText(/aparecem aqui quando a tarefa é finalizada/)).toBeTruthy();
    cleanup();
    mockFetch({ erro: 'x' }, 500);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect(await screen.findByText('Não foi possível abrir TASK-02')).toBeTruthy();
  });
  describe('estado da conferência', () => {
    const entry = (name: string) => ({ name, path: `x/${name}`, size: 3 });
    const reviews = [
      { name: 'saida.txt', kind: 'automática', status: 'conferida', reason: '' },
      { name: 'nota.diff', kind: 'diff', status: 'recusada', reason: 'falta o total' },
      { name: 'tela.png', kind: 'print', status: 'a conferir', reason: '' },
      { name: 'extra.txt', kind: 'automática', status: 'sem aval', reason: '' },
    ];
    const reviewsDetail = () => ({ ...taskDetail({}, ['saida.txt', 'nota.diff', 'tela.png', 'extra.txt'].map(entry)), reviews });
    const itemOf = (name: string) => within(screen.getByText(name).closest('li') as HTMLElement);

    it('mostra os quatro estados da conferência', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      await screen.findByText('saida.txt');
      expect(itemOf('saida.txt').getByText('conferida')).toBeTruthy();
      expect(itemOf('nota.diff').getByText('recusada')).toBeTruthy();
      expect(itemOf('tela.png').getByText('a conferir')).toBeTruthy();
      expect(itemOf('extra.txt').getByText('sem aval')).toBeTruthy();
      for (const text of ['conferida', 'recusada', 'a conferir', 'sem aval']) expect(screen.getAllByText(text)).toHaveLength(1);
    });

    it('mostra o motivo da rejeição', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      await screen.findByText('nota.diff');
      expect(itemOf('nota.diff').getByText('Motivo: falta o total')).toBeTruthy();
      expect(screen.getAllByText(/Motivo:/)).toHaveLength(1);
    });

    it('não mostra botão de conferir nem de rejeitar', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      await screen.findByText('saida.txt');
      expect(screen.queryByRole('button', { name: /Aceitar|Rejeitar|Conferir/ })).toBeNull();
    });

    it('sem reviews não mostra selo de conferência', async () => {
      mockFetch(taskDetail({}, ['saida.txt', 'tela.png'].map(entry)));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      await screen.findByText('saida.txt');
      for (const text of ['conferida', 'recusada', 'a conferir', 'sem aval']) expect(screen.queryByText(text)).toBeNull();
    });

    it('mostra o selo da evidência manual anexada', async () => {
      const manualItem = { name: 'tela.png', path: 'x/tela.png', size: 3, description: 'Print final.', extension: 'png', attached: true };
      mockFetch({ ...taskDetail({}, [entry('tela.png')]), manual_evidence: [manualItem], reviews: [reviews[2]] });
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      const list = await screen.findByRole('list', { name: 'Evidências que o harness pede' });
      expect(within(list).getByText('a conferir')).toBeTruthy();
    });
  });

  describe('Data e aval da conferência', () => {
    const entry = (name: string) => ({ name, path: `x/${name}`, size: 3 });
    const itemWith = async (review: Record<string, string>) => {
      mockFetch({ ...taskDetail({}, [entry('saida.txt')]), reviews: [{ name: 'saida.txt', kind: 'automática', ...review }] });
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await openEvidence();
      await screen.findByText('saida.txt');
      return within(screen.getByText('saida.txt').closest('li') as HTMLElement);
    };

    it('mostra a data e o aval da conferência', async () => {
      const item = await itemWith({ status: 'conferida', reason: 'a tela bate', reviewed_at: '2026-11-10' });
      expect(item.getByText('em 10/11/2026')).toBeTruthy();
      expect(item.getByText('Aval: a tela bate')).toBeTruthy();
    });

    it('mostra só a data na conferência sem aval', async () => {
      const item = await itemWith({ status: 'conferida', reason: '', reviewed_at: '2026-11-10' });
      expect(item.getByText('em 10/11/2026')).toBeTruthy();
      expect(item.queryByText(/Aval:/)).toBeNull();
    });

    it('mostra a data da rejeição com o motivo', async () => {
      const item = await itemWith({ status: 'recusada', reason: 'falta o total', reviewed_at: '2026-11-09' });
      expect(item.getByText('em 09/11/2026')).toBeTruthy();
      expect(item.getByText('Motivo: falta o total')).toBeTruthy();
      expect(item.queryByText(/Aval:/)).toBeNull();
    });

    it('não mostra data na evidência a conferir nem na resposta sem reviewed_at', async () => {
      const pending = await itemWith({ status: 'a conferir', reason: '', reviewed_at: '' });
      expect(pending.queryByText(/^em \d/)).toBeNull();
      cleanup();
      const legacy = await itemWith({ status: 'conferida', reason: '' });
      expect(legacy.queryByText(/^em \d/)).toBeNull();
    });
  });

  it('não mostra botão de ação em nenhum status', async () => {
    for (const status of ['pending', 'approved', 'in_progress', 'validating', 'blocked', 'done']) {
      mockFetch(taskDetail({ status }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('Texto executivo.');
      for (const button of screen.queryAllByRole('button')) expect(button.textContent ?? '').not.toMatch(/Aprovar|Executar|Finalizar|Ajustar|Decidir|Editar|Aceitar|Rejeitar|Anexar|Trocar/);
      cleanup();
    }
  });
});

describe('resposta real da API', () => {
  it('mostra a tarefa quando a resposta não traz manual_evidence', async () => {
    const { manual_evidence: _omit, ...real } = taskDetail({}, [{ name: 'painel-binario.txt', path: 'x/painel-binario.txt', size: 1562 }]);
    mockFetch({ ...real, archived: false });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect(await screen.findByRole('heading', { level: 2, name: 'Resumo executivo' })).toBeTruthy();
    expect(screen.queryByText('Carregando TASK-02')).toBeNull();
  });
});

describe('Evidências por tipo', () => {
  const MB = 1024 * 1024;
  type Call = { url: string; init?: RequestInit };
  function mockEvidence(detail: unknown, file: () => Response = () => new Response('conteúdo')) {
    const calls: Call[] = [];
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string, init?: RequestInit) => {
        calls.push({ url, init });
        if (url.startsWith('/api/tarefas')) return Promise.resolve(json(detail));
        if (url.startsWith('/arquivos/')) return Promise.resolve(file());
        return Promise.resolve(json(specDetail));
      }),
    );
    return calls;
  }
  const filesOf = (calls: Call[]) => calls.filter(({ url }) => url.startsWith('/arquivos/'));
  const entry = (name: string, size = 100) => ({ name, path: `x/${name}`, size });
  const withEvidence = (evidence: object[], verificacao?: string) => {
    const base = taskDetail({}, evidence);
    return verificacao ? { ...base, files: { ...base.files, verificacao } } : base;
  };
  const partial = (body: BodyInit, range: string | null) => () =>
    new Response(body, { status: 206, headers: range ? { 'Content-Range': range } : {} });

  it('mostra imagem, vídeo e texto e só dá o link aos demais', async () => {
    const names = ['tela.png', 'a.mp4', 'b.webm', 'c.mov', 'nota.txt', 'pacote.zip', 'desenho.svg', 'pagina.html', 'PAGINA.HTML', 'Misto.SvG', 'doc.pdf', 'semextensao'];
    const calls = mockEvidence(withEvidence(names.map((name) => entry(name))));
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    await screen.findByAltText('Evidência tela.png');
    expect(container.querySelectorAll('img')).toHaveLength(1);
    expect(container.querySelectorAll('video')).toHaveLength(3);
    expect(screen.getByRole('button', { name: /nota\.txt/ }).getAttribute('aria-expanded')).toBe('false');
    for (const name of ['pacote.zip', 'desenho.svg', 'pagina.html', 'PAGINA.HTML', 'Misto.SvG', 'doc.pdf', 'semextensao']) {
      const link = screen.getByRole('link', { name: `Baixar ${name}` });
      expect(link.hasAttribute('download')).toBe(true);
      expect(link.getAttribute('href')).toBe(fileUrl(`x/${name}`));
    }
    expect(filesOf(calls)).toHaveLength(0);
  });

  it('mantém a conferência no print declarado como svg', async () => {
    mockEvidence({ ...withEvidence([entry('tela.svg')], '## V\n- `tela.svg` (print): `cmd`\n'), reviews: [{ name: 'tela.svg', kind: 'print', status: 'a conferir', reason: '' }] });
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    await screen.findByRole('link', { name: 'Baixar tela.svg' });
    expect(container.querySelector('img')).toBeNull();
    expect(await screen.findByText('a conferir')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Aceitar|Rejeitar/ })).toBeNull();
  });

  it('pede só o primeiro 1 MB do texto e avisa quando há mais', async () => {
    const calls = mockEvidence(withEvidence([entry('grande.log', 3 * MB)]), partial('linha', `bytes 0-${MB - 1}/${3 * MB}`));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /grande\.log/ }));
    expect(await screen.findByText(/Mostrando o primeiro 1 MB de 3,0 MB\./)).toBeTruthy();
    expect(screen.getByRole('link', { name: 'Baixar o arquivo inteiro' })).toBeTruthy();
    const requests = filesOf(calls);
    expect(requests).toHaveLength(1);
    expect((requests[0].init?.headers as Record<string, string>).Range).toBe(`bytes=0-${MB - 1}`);
  });

  it('mostra inteiro o texto menor, sem aviso', async () => {
    mockEvidence(withEvidence([entry('pequeno.txt', 10240)]), partial('tudo', 'bytes 0-10239/10240'));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /pequeno\.txt/ }));
    expect(await screen.findByText('tudo')).toBeTruthy();
    expect(screen.queryByText(/Mostrando o primeiro/)).toBeNull();
    expect(screen.queryByRole('link', { name: 'Baixar o arquivo inteiro' })).toBeNull();
  });

  it('aceita a resposta sem Content-Range', async () => {
    mockEvidence(withEvidence([entry('nota.txt', 4)]), () => new Response('nota'));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /nota\.txt/ }));
    expect(await screen.findByText('nota')).toBeTruthy();
    expect(screen.queryByText(/Mostrando o primeiro/)).toBeNull();
  });

  it('mostra o caractere de substituição no texto cortado', async () => {
    mockEvidence(withEvidence([entry('corte.txt', 2)]), partial(new Uint8Array([0x61, 0xc3]), null));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /corte\.txt/ }));
    expect(await screen.findByText('a�')).toBeTruthy();
  });

  it('não pede o conteúdo de um texto vazio', async () => {
    const calls = mockEvidence(withEvidence([entry('vazio.txt', 0)]));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /vazio\.txt/ }));
    await waitFor(() => expect(screen.getByRole('button', { name: 'Ver vazio.txt' }).getAttribute('aria-expanded')).toBe('true'));
    expect(screen.queryByText('Carregando…')).toBeNull();
    expect(filesOf(calls)).toHaveLength(0);
  });

  it('mostra a falha ao ler o texto', async () => {
    mockEvidence(withEvidence([entry('nota.txt', 4)]), () => new Response('erro', { status: 500 }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: /nota\.txt/ }));
    expect(await screen.findByText('Não foi possível ler o arquivo.')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Ver nota.txt' })).toBeTruthy();
  });
});

describe('anexo de evidência', () => {
  const manual = (over: object = {}) => ({ name: 'tela.png', path: 'x/tela.png', size: 0, description: 'Print da tela final.', extension: 'png', attached: false, ...over });
  const attachDetail = (item: object, stateOver: object = {}) => {
    const base = taskDetail(stateOver, (item as { attached: boolean }).attached ? [{ name: (item as { name: string }).name, path: (item as { path: string }).path, size: (item as { size: number }).size }] : []);
    return { ...base, manual_evidence: [item] };
  };
  const open = async (detail: unknown) => {
    vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(detail) : json(specDetail))));
    const view = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    return view;
  };

  afterEach(() => {
    history.pushState(null, '', '/');
    vi.restoreAllMocks();
  });

  it('mostra o que o harness pede, sem Anexar', async () => {
    await open(attachDetail(manual()));
    expect(await screen.findByText('Print da tela final.')).toBeTruthy();
    expect(screen.getByText('falta anexar')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Anexar/ })).toBeNull();
    expect(screen.getByRole('list', { name: 'Evidências que o harness pede' })).toBeTruthy();
    expect(screen.queryByText(/aparecem aqui quando a tarefa é finalizada/)).toBeNull();
  });

  it('mostra a imagem, sem Trocar', async () => {
    await open(attachDetail(manual({ attached: true, size: 2048 })));
    expect(await screen.findByAltText('Evidência tela.png')).toBeTruthy();
    expect(screen.getByText('anexado')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Trocar|Anexar/ })).toBeNull();
  });

  it('mostra o link de baixar, sem Trocar', async () => {
    await open(attachDetail(manual({ name: 'pacote.zip', path: 'x/pacote.zip', extension: 'zip', attached: true, size: 2 })));
    const link = await screen.findByRole('link', { name: 'Baixar pacote.zip' });
    expect(link.hasAttribute('download')).toBe(true);
    expect(screen.queryByRole('button', { name: /Trocar|Anexar/ })).toBeNull();
  });

  it('não repete a evidência manual na lista geral', async () => {
    const { container } = await open(attachDetail(manual({ attached: true, size: 2048 })));
    await screen.findByAltText('Evidência tela.png');
    expect(container.querySelectorAll('img')).toHaveLength(1);
  });

  it('abre a aba Evidências com #evidencias', async () => {
    history.pushState(null, '', '/tarefas/SPEC-001/TASK-02#evidencias');
    const { rerender } = await open(attachDetail(manual()));
    const tab = () => screen.getByRole('tab', { name: 'Evidências' });
    expect(tab().getAttribute('aria-selected')).toBe('true');
    expect(await screen.findByText('Print da tela final.')).toBeTruthy();
    const taskFetches = () => vi.mocked(fetch).mock.calls.filter(([url]) => String(url).startsWith('/api/tarefas')).length;
    const before = taskFetches();
    rerender(<TaskPage spec="SPEC-001" task="TASK-02" version={1} />);
    await waitFor(() => expect(taskFetches()).toBeGreaterThan(before));
    expect(tab().getAttribute('aria-selected')).toBe('true');
  });
});

describe('TaskPage em abas', () => {
  const tabNames = ['Resumo', 'Evidências', 'Para o agente'];

  it('guarda a aba aberta no hash', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('tab', { name: 'Para o agente' }));
    expect(window.location.hash).toBe('#agente');
  });

  it('abre Resumo com hash desconhecido', async () => {
    history.pushState(null, '', '/tarefas/SPEC-001/TASK-02#nada');
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect((await screen.findByRole('tab', { name: 'Resumo' })).getAttribute('aria-selected')).toBe('true');
  });

  it('não mostra o índice nem os cards e mantém o aside', async () => {
    mockFetch(taskDetail());
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByRole('tab', { name: 'Resumo' });
    expect(screen.queryByRole('navigation', { name: 'Nesta tarefa' })).toBeNull();
    for (const name of tabNames) {
      fireEvent.click(screen.getByRole('tab', { name }));
      expect(container.querySelector('.' + cardStyles.root)).toBeNull();
    }
    const aside = within(screen.getByRole('complementary'));
    for (const label of ['Status', 'Onda', 'Depende de', 'Cobre', 'Perfil', 'Persona', 'Modelo', 'Aprovada', 'Tentativas', 'Validação']) expect(aside.getByText(label)).toBeTruthy();
  });
});

describe('ordem da evidência', () => {
  afterEach(() => history.pushState(null, '', '/'));
  const item = '- `saida.txt` (automática, da verificação)';
  const fields = ['  - Título: Testes do quadro passam', '  - Prova: prova da saída', '  - Espera: texto esperado', '  - Trecho: linha do trecho', '  - Falha: linha da falha'];
  const verification = (lines: string[]) => ['### Evidências', '', item, ...lines, '', '### Outra', ''].join('\n');
  const reviewList = [{ name: 'saida.txt', kind: 'automática', status: 'a conferir', reason: '' }];
  const detailWith = (lines: string[], extra: object = { reviews: reviewList }) => {
    const base = taskDetail({}, [{ name: 'saida.txt', path: 'x/saida.txt', size: 3 }]);
    return { ...base, files: { ...base.files, verificacao: verification(lines) }, ...extra };
  };
  async function open(task: unknown) {
    mockFetch(task);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
  }

  it('mostra a evidência na ordem de leitura', async () => {
    await open(detailWith(fields));
    const li = (await screen.findByText('Testes do quadro passam')).closest('li') as HTMLElement;
    const text = li.textContent ?? '';
    const order = ['Testes do quadro passam', 'prova da saída', 'texto esperado', 'linha do trecho', 'linha da falha', 'a conferir', 'Ver conteúdo', 'saida.txt'].map((part) => text.indexOf(part));
    expect(order.every((index) => index >= 0)).toBe(true);
    expect([...order].sort((a, b) => a - b)).toEqual(order);
    const button = within(li).getByRole('button', { name: 'Ver saida.txt' });
    const badge = within(li).getByText('a conferir');
    const name = li.querySelector('.' + taskStyles.evidenceFile) as HTMLElement;
    expect(badge.compareDocumentPosition(button) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(button.compareDocumentPosition(name) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    expect(within(li).getAllByText('saida.txt')).toHaveLength(1);
  });

  it('mostra o nome do arquivo no lugar do título na evidência antiga', async () => {
    await open(detailWith(fields.slice(1)));
    const heading = await screen.findByRole('heading', { level: 3, name: 'saida.txt' });
    expect(heading).toBeTruthy();
    expect(screen.getAllByText('saida.txt')).toHaveLength(1);
  });

  it('não mostra a linha de conferência sem reviews', async () => {
    await open(detailWith(fields, {}));
    await screen.findByText('Testes do quadro passam');
    expect(screen.getByText('prova da saída')).toBeTruthy();
    expect(screen.getByText('linha do trecho')).toBeTruthy();
    expect(screen.getByText('linha da falha')).toBeTruthy();
    for (const label of ['conferida', 'recusada', 'a conferir', 'sem aval']) expect(screen.queryByText(label)).toBeNull();
  });

  it('mostra só o nome do arquivo para evidência fora da verificação', async () => {
    const base = detailWith(fields);
    await open({ ...base, files: { ...base.files, verificacao: '## V\nComandos.' } });
    expect(await screen.findByRole('heading', { level: 3, name: 'saida.txt' })).toBeTruthy();
    for (const label of ['Prova', 'Trecho', 'Falha']) expect(screen.queryByText(label)).toBeNull();
  });
});

describe('Esperas no aside', () => {
  const waitsSpec = { ...specDetail, tasks: [{ id: 'TASK-01', title: 'a', status: 'done', wave: '1', profile: 'padrao', covers: '' }, { id: 'TASK-12', title: 'b', status: 'in_progress', wave: '1', profile: 'padrao', covers: '' }] };
  const mockWaits = (task: unknown) => vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task) : json(waitsSpec))));
  const aside = () => within(screen.getByRole('complementary'));
  const row = (label: string) => aside().getByText(label, { selector: 'dt' }).parentElement;

  it('mostra Espera, Fecha depois de e Depende de no aside', async () => {
    mockWaits(taskDetail({ waiting_on: '[TASK-03]', waiting_reason: 'motivo', closes_after: '[TASK-12]', depends_on: '[TASK-01]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Fecha depois de', { selector: 'dt' });
    expect(row('Espera')?.textContent).toContain('TASK-03: motivo');
    expect(row('Fecha depois de')?.textContent).toContain('TASK-12');
    expect(row('Depende de')?.textContent).toContain('TASK-01');
  });

  it('esconde Fecha depois de quando a tarefa já está concluída', async () => {
    mockWaits(taskDetail({ closes_after: '[TASK-01]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Depende de', { selector: 'dt' });
    await waitFor(() => expect(aside().getByText('TASK-01', { selector: 'a' })).toBeTruthy());
    expect(aside().queryByText('Fecha depois de', { selector: 'dt' })).toBeNull();
  });

  it('não mostra espera sem waiting_on', async () => {
    mockWaits(taskDetail({ status: 'done', waiting_on: '[]' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Depende de', { selector: 'dt' });
    expect(aside().queryByText('Espera', { selector: 'dt' })).toBeNull();
    expect(row('Depende de')?.textContent).toContain('TASK-01');
  });
});

describe('Componentes de mídia na evidência', () => {
  const entry = (name: string) => ({ name, path: `x/${name}`, size: 100 });
  const LOG = '$ cmd\n# data: d\n# commit: c\n# código de saída: 0\nsaída';
  function mockEvidence(names: string[], file = 'conteúdo') {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string) => {
        if (url.startsWith('/api/tarefas')) return Promise.resolve(json(taskDetail({}, names.map(entry))));
        if (url.startsWith('/arquivos/')) return Promise.resolve(new Response(file));
        return Promise.resolve(json(specDetail));
      }),
    );
  }
  function stubClipboard(writeText: () => Promise<void>) {
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
  }

  it('abre a imagem da evidência no visualizador', async () => {
    mockEvidence(['tela.png']);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: 'Ampliar Evidência tela.png' }));
    expect(screen.getByRole('dialog', { name: 'Evidência tela.png' })).toBeTruthy();
  });

  it('toca o vídeo da evidência no player', async () => {
    mockEvidence(['a.mp4']);
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    const group = await screen.findByRole('group', { name: 'Vídeo a.mp4' });
    expect(within(group).getByRole('button', { name: 'Tocar' })).toBeTruthy();
    expect(container.querySelector('video')?.hasAttribute('controls')).toBe(false);
  });

  it('copia o conteúdo inteiro do texto com o cabeçalho', async () => {
    const writeText = vi.fn(() => Promise.resolve());
    stubClipboard(writeText);
    mockEvidence(['nota.txt'], LOG);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    fireEvent.click(await screen.findByRole('button', { name: 'Ver nota.txt' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Copiar nota.txt' }));
    expect(writeText).toHaveBeenCalledWith(LOG);
  });

  it('não mostra o botão de copiar antes de o texto carregar', async () => {
    mockEvidence(['nota.txt'], LOG);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
    await screen.findByRole('button', { name: 'Ver nota.txt' });
    expect(screen.queryByRole('button', { name: 'Copiar nota.txt' })).toBeNull();
  });
});

describe('ID das evidências', () => {
  const entry = (name: string) => ({ name, path: `x/${name}`, size: 3 });
  const verification = (name: string, title: boolean) => ['### Evidências', '', `- \`${name}\` (automática, da verificação)`, ...(title ? ['  - Título: Testes do quadro passam'] : []), '', '### Outra', ''].join('\n');
  async function open(name: string, title: boolean, review: object) {
    const base = taskDetail({}, [entry(name)]);
    mockFetch({ ...base, files: { ...base.files, verificacao: verification(name, title) }, reviews: [{ name, kind: 'automática', status: 'a conferir', reason: '', ...review }] });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
  }

  it('mostra o título com o ID como prefixo', async () => {
    await open('saida.txt', true, { id: 'EV-01' });
    expect(await screen.findByRole('heading', { level: 3, name: '[EV-01] Testes do quadro passam' })).toBeTruthy();
  });

  it('mostra o ID com o nome do arquivo sem Título', async () => {
    await open('tela.png', false, { id: 'EV-02' });
    expect(await screen.findByRole('heading', { level: 3, name: '[EV-02] tela.png' })).toBeTruthy();
  });

  it('mostra o título sem prefixo quando a resposta não traz o ID', async () => {
    await open('saida.txt', true, {});
    expect(await screen.findByRole('heading', { level: 3, name: 'Testes do quadro passam' })).toBeTruthy();
  });
});

describe('Filtro de evidências', () => {
  const entry = (name: string) => ({ name, path: `x/${name}`, size: 3 });
  const names = ['saida.txt', 'nota.diff', 'tela.png', 'extra.txt'];
  const statuses = ['conferida', 'recusada', 'a conferir', 'sem aval'];
  const reviews = (list: string[]) => names.map((name, index) => ({ name, kind: 'automática', status: list[index], reason: '' }));
  async function open(task: unknown) {
    mockFetch(task);
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
  }
  const group = () => screen.getByRole('group', { name: 'Filtrar evidências' });

  it('filtra por todas, a conferir e conferidas com a contagem', async () => {
    await open({ ...taskDetail({}, names.map(entry)), reviews: reviews(statuses) });
    await screen.findByText('saida.txt');
    const all = within(group()).getByRole('button', { name: 'Todas 4' });
    expect(all.getAttribute('aria-pressed')).toBe('true');
    fireEvent.click(within(group()).getByRole('button', { name: 'A conferir 1' }));
    expect(screen.queryByText('saida.txt')).toBeNull();
    expect(screen.getByText('tela.png')).toBeTruthy();
    expect(screen.getAllByRole('heading', { level: 3 })).toHaveLength(1);
    fireEvent.click(within(group()).getByRole('button', { name: 'Conferidas 1' }));
    expect(screen.getByText('saida.txt')).toBeTruthy();
    expect(screen.queryByText('tela.png')).toBeNull();
    fireEvent.click(within(group()).getByRole('button', { name: 'Todas 4' }));
    expect(screen.getAllByRole('heading', { level: 3 })).toHaveLength(4);
  });

  it('avisa quando nenhuma evidência está no filtro', async () => {
    await open({ ...taskDetail({}, names.map(entry)), reviews: reviews(['recusada', 'recusada', 'a conferir', 'sem aval']) });
    await screen.findByText('saida.txt');
    fireEvent.click(within(group()).getByRole('button', { name: 'Conferidas 0' }));
    expect(screen.getByText('Nenhuma evidência neste filtro.')).toBeTruthy();
  });

  it('não mostra o filtro sem evidências', async () => {
    await open(taskDetail());
    await screen.findByText('Elas aparecem aqui quando a tarefa é finalizada.');
    expect(screen.queryByRole('group', { name: 'Filtrar evidências' })).toBeNull();
  });
});

describe('Destaque do Trecho', () => {
  const LOG = '$ pnpm test\n# data: 2026-11-11T10:00:00Z\n# commit: abc1234\n# código de saída: 0\n✓ a > um\n✓ a > dois\ntest result: ok. 3 passed\ntest result: ok\nTest Files  1 passed (1)\npnpm fim';

  it('prefere a linha igual à linha que contém o texto', () => {
    expect(findLines(LOG, 'test result: ok')).toEqual([7]);
  });
  it('usa a linha que contém o texto quando nenhuma é igual', () => {
    expect(findLines(LOG, 'passed')).toEqual([6, 8]);
  });
  it('normaliza os espaços na busca', () => {
    expect(findLines(LOG, 'Test Files 1 passed')).toEqual([8]);
  });
  it('destaca todas as linhas do trecho genérico', () => {
    expect(findLines(LOG, '✓')).toEqual([4, 5]);
  });
  it('ignora as quatro linhas de cabeçalho do log', () => {
    expect(findLines(LOG, 'pnpm')).toEqual([9]);
    expect(findLines(LOG, '# commit')).toEqual([]);
  });
  it('devolve vazio para o trecho que não aparece', () => {
    expect(findLines(LOG, 'FAILED')).toEqual([]);
  });

  describe('reticências no trecho', () => {
    const LOG_RETICENCIAS = '$ pnpm test\n# data: d\n# commit: c\n# código de saída: 0\n✓ src/features/board/board.test.tsx > Board > Filtro de status, ordem e faixa > esconde a raia desmarcada e lembra a seleção ao recarregar\n✓ src/features/board/board.test.tsx > Board > Filtro de status, ordem e faixa > mostra as raias da última etapa para a primeira por padrão\nTests  10 passed (10)\nTests  0 passed (0)';

    it('casa os textos da TASK-08 com reticências no meio', () => {
      expect(findLines(LOG_RETICENCIAS, '✓ ... Filtro de status, ordem e faixa > esconde a raia desmarcada e lembra a seleção ao recarregar')).toEqual([4]);
      expect(findLines(LOG_RETICENCIAS, '... > mostra as raias da última etapa para a primeira por padrão')).toEqual([5]);
    });
    it('casa os textos da EV-02 da TASK-08 com o tempo no fim da linha', () => {
      const log = '$ pnpm test\n# data: d\n# commit: c\n# código de saída: 0\n✓ src/shared/router.test.ts > Quadro único > troca o endereço sem criar entrada no histórico 3ms\n✓ src/features/spec/spec-page.test.tsx > SpecPage > aponta o quadro para o menu Tarefas, sem a visão Quadro 54ms\n✓ src/layout/topbar.test.tsx > Topbar > leva a migalha da tarefa ao quadro filtrado pela spec 8ms\n✓ src/features/dashboard/dashboard-page.test.tsx > DashboardPage > leva o quadro de cada spec ao menu Tarefas filtrado 12ms';
      expect(findLines(log, '✓ ... Quadro único > troca o endereço sem criar entrada no histórico')).toEqual([4]);
      expect(findLines(log, '✓ ... SpecPage > aponta o quadro para o menu Tarefas, sem a visão Quadro')).toEqual([5]);
      expect(findLines(log, '✓ ... Topbar > leva a migalha da tarefa ao quadro filtrado pela spec')).toEqual([6]);
      expect(findLines(log, '✓ ... leva o quadro de cada spec ao menu Tarefas filtrado')).toEqual([7]);
    });
    it('aceita as reticências em caractere único', () => {
      expect(findLines(LOG_RETICENCIAS, '✓ … Board … esconde')).toEqual([4]);
    });
    it('casa as reticências no início do texto', () => {
      expect(findLines(LOG_RETICENCIAS, '... passed')).toEqual([6, 7]);
    });
    it('mantém o limite de palavra só nas pontas sem reticências', () => {
      expect(findLines(LOG_RETICENCIAS, '... passe')).toEqual([]);
      expect(findLines(LOG_RETICENCIAS, 'Tests ... 0 passed')).toEqual([6, 7]);
      expect(findLines(LOG_RETICENCIAS, '0 passed')).toEqual([7]);
    });
    it('não casa quando as partes aparecem fora de ordem', () => {
      expect(findLines(LOG_RETICENCIAS, 'esconde ... Filtro de status')).toEqual([]);
    });
    it('não casa um texto só de reticências', () => {
      expect(findLines(LOG_RETICENCIAS, '...')).toEqual([]);
      expect(findLines(LOG_RETICENCIAS, '…')).toEqual([]);
      expect(findLines(LOG_RETICENCIAS, ' ... … ')).toEqual([]);
    });
  });

  describe('limite de palavra', () => {
    const LOG_LIMITE = '$ pnpm test\n# data: d\n# commit: c\n# código de saída: 0\nTests  10 passed (10)\nTests  0 passed\nSnapshots  0 passed (0)\nação concluída\nTests  2 failed';

    it('não casa o texto no meio de um número', () => {
      expect(findLines(LOG_LIMITE, '0 passed')).toEqual([5, 6]);
    });
    it('casa o texto no fim de uma linha com outras palavras', () => {
      expect(findLines('Tests 0 passed', '0 passed')).toEqual([0]);
      expect(findLines(LOG_LIMITE, 'Tests 0 passed')).toEqual([5]);
    });
    it('não casa dentro de uma palavra', () => {
      expect(findLines(LOG_LIMITE, 'ção')).toEqual([]);
      expect(findLines(LOG_LIMITE, 'ação')).toEqual([7]);
    });
    it('respeita acentos como letras', () => {
      expect(findLines('não\nnão é', 'ão')).toEqual([]);
    });
  });

  const verification = (lines: string[]) => ['### Evidências', '', '- `saida.txt` (automática, da verificação)', ...lines, '', '### Outra', ''].join('\n');
  async function open(lines: string[]) {
    const base = taskDetail({}, [{ name: 'saida.txt', path: 'x/saida.txt', size: LOG.length }]);
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string) => {
        if (url.startsWith('/api/tarefas')) return Promise.resolve(json({ ...base, files: { ...base.files, verificacao: verification(lines) } }));
        if (url.startsWith('/arquivos/')) return Promise.resolve(new Response(LOG));
        return Promise.resolve(json(specDetail));
      }),
    );
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await openEvidence();
  }
  const trecho = () => screen.findByRole('button', { name: 'Destacar test result: ok no log' });
  const hits = (container: HTMLElement) => container.querySelectorAll('.' + taskStyles.hit);

  describe('cópia do trecho', () => {
    const lines = ['  - Trecho: a linha `test result: ok`', '  - Falha: `FAILED`'];
    const stubClipboard = (value: unknown) => Object.defineProperty(navigator, 'clipboard', { value, configurable: true });
    const click = async (name: string) => fireEvent.click(await screen.findByRole('button', { name }));

    it('copia o texto do trecho sem as crases ao clicar e confirma por cerca de 1,5 s', async () => {
      Element.prototype.scrollIntoView = vi.fn();
      const writeText = vi.fn(() => Promise.resolve());
      stubClipboard({ writeText });
      await open(lines);
      await click('Destacar test result: ok no log');
      expect(writeText).toHaveBeenCalledWith('test result: ok');
      await waitFor(() => expect(screen.getAllByText('Copiado').length).toBeGreaterThan(0));
      expect(screen.getAllByRole('status').some((node) => node.textContent === 'Copiado')).toBe(true);
      await waitFor(() => expect(hits(document.body)).toHaveLength(1));
      await waitFor(() => expect(screen.queryByText('Copiado')).toBeNull(), { timeout: 3000 });
    });

    it('copia também o texto da Falha', async () => {
      stubClipboard({ writeText: vi.fn(() => Promise.resolve()) });
      await open(lines);
      await click('Destacar FAILED no log');
      expect((navigator.clipboard.writeText as unknown as ReturnType<typeof vi.fn>)).toHaveBeenCalledWith('FAILED');
    });

    it('não copia ao passar o mouse nem ao focar', async () => {
      Element.prototype.scrollIntoView = vi.fn();
      const writeText = vi.fn(() => Promise.resolve());
      stubClipboard({ writeText });
      await open(lines);
      const button = await trecho();
      fireEvent.mouseEnter(button);
      fireEvent.focusIn(button);
      await waitFor(() => expect(hits(document.body)).toHaveLength(1));
      expect(writeText).not.toHaveBeenCalled();
    });

    it('avisa Não foi possível copiar e mantém o destaque quando a cópia é recusada', async () => {
      Element.prototype.scrollIntoView = vi.fn();
      stubClipboard({ writeText: vi.fn(() => Promise.reject(new Error('negado'))) });
      await open(lines);
      await click('Destacar test result: ok no log');
      await waitFor(() => expect(screen.getAllByText('Não foi possível copiar').length).toBeGreaterThan(0));
      expect(screen.getAllByRole('status').some((node) => node.textContent === 'Não foi possível copiar')).toBe(true);
      await waitFor(() => expect(hits(document.body)).toHaveLength(1));
    });

    it('avisa Não foi possível copiar quando não há área de transferência', async () => {
      Element.prototype.scrollIntoView = vi.fn();
      stubClipboard(undefined);
      await open(lines);
      await click('Destacar test result: ok no log');
      await waitFor(() => expect(screen.getAllByText('Não foi possível copiar').length).toBeGreaterThan(0));
      await waitFor(() => expect(hits(document.body)).toHaveLength(1));
    });
  });

  it('abre o log, destaca as linhas e rola até a primeira ao passar o mouse', async () => {
    const scroll = vi.fn();
    Element.prototype.scrollIntoView = scroll;
    await open(['  - Trecho: a linha `test result: ok`']);
    fireEvent.mouseEnter(await trecho());
    const toggle = await screen.findByRole('button', { name: 'Ver saida.txt' });
    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    await waitFor(() => expect(hits(document.body)).toHaveLength(1));
    const hit = hits(document.body)[0] as HTMLElement;
    expect(hit.textContent).toBe('test result: ok');
    await waitFor(() => expect(scroll).toHaveBeenCalled());
    expect(scroll.mock.contexts[0]).toBe(hit);
    expect(screen.getByText('1 linha')).toBeTruthy();
  });

  it('destaca também pelo foco do teclado', async () => {
    Element.prototype.scrollIntoView = vi.fn();
    await open(['  - Trecho: a linha `test result: ok`']);
    const button = await trecho();
    fireEvent.focusIn(button);
    await waitFor(() => expect(hits(document.body)).toHaveLength(1));
    fireEvent.focusOut(button);
    expect(hits(document.body)).toHaveLength(0);
  });

  it('tira o destaque ao sair do trecho', async () => {
    Element.prototype.scrollIntoView = vi.fn();
    await open(['  - Trecho: a linha `test result: ok`']);
    const button = await trecho();
    fireEvent.mouseEnter(button);
    await waitFor(() => expect(hits(document.body)).toHaveLength(1));
    fireEvent.mouseLeave(button);
    expect(hits(document.body)).toHaveLength(0);
    expect(screen.getByRole('button', { name: 'Ver saida.txt' }).getAttribute('aria-expanded')).toBe('true');
  });

  it('marca o trecho não encontrado como alerta e a falha não encontrada como bom', async () => {
    await open(['  - Trecho: `nada aqui`', '  - Falha: `FAILED`']);
    fireEvent.click(await screen.findByRole('button', { name: 'Ver saida.txt' }));
    const badges = await screen.findAllByText('não encontrado');
    expect(badges).toHaveLength(2);
    expect(badges[0].className).toMatch(/warning/);
    expect(badges[1].className).toMatch(/success/);
  });
});

const summary = (id: string, wave: string) => ({ id, title: id, status: 'done', wave, profile: 'padrao', covers: '' });
const waveSpec = { ...specDetail, tasks: [summary('TASK-01', '1'), summary('TASK-02', '1'), summary('TASK-03', '1'), summary('TASK-04', '2')] };
const toReview = [{ name: 'a.txt', kind: 'automática', status: 'a conferir', reason: '' }];
const reviewed = [{ name: 'a.txt', kind: 'automática', status: 'conferida', reason: '' }];

function mockRoutes(details: Record<string, unknown> = {}, tasks = waveSpec) {
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    if (!url.startsWith('/api/tarefas')) return Promise.resolve(json(tasks));
    const id = url.split('/').pop() as string;
    const body = details[id] ?? taskDetail({ id, wave: '1' });
    return Promise.resolve(typeof body === 'number' ? json({}, body) : json(body));
  }));
}
const withReviews = (id: string, reviews: object[]) => ({ ...taskDetail({ id, wave: '1' }), reviews });
const renderTask = async (task = 'TASK-02') => {
  render(<TaskPage spec="SPEC-001" task={task} version={0} />);
  return screen.findByRole('navigation', { name: 'Tarefas da onda' });
};

describe('Barra da onda', () => {
  beforeEach(() => { window.scrollTo = vi.fn(); });

  it('volta por padrão ao quadro filtrado pela spec', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    expect(screen.getByRole('link', { name: /Quadro de SPEC-001|Voltar/ }).getAttribute('href')).toBe('/tarefas?spec=SPEC-001');
  });

  it('mostra a anterior, a próxima e a posição na onda', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    const bar = await renderTask();
    expect(bar.textContent).toContain('TASK-02 · Onda 1 · 2 de 3');
    expect(within(bar).getByRole('link', { name: 'Tarefa anterior: TASK-01' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
    expect(within(bar).getByRole('link', { name: 'Próxima tarefa: TASK-03' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-03');
  });

  it('abre a próxima com → e a anterior com ←', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    await waitFor(() => {
      fireEvent.keyDown(document.body, { key: 'ArrowRight' });
      expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-03');
    });
    history.replaceState(null, '', '/');
    fireEvent.keyDown(document.body, { key: 'ArrowLeft' });
    expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-01');
  });

  it('não troca de tarefa com as setas nas abas, num campo ou com modificador', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }) });
    await renderTask();
    fireEvent.keyDown(screen.getByRole('tab', { name: 'Resumo' }), { key: 'ArrowRight' });
    const input = document.body.appendChild(document.createElement('input'));
    fireEvent.keyDown(input, { key: 'ArrowRight' });
    fireEvent.keyDown(document.body, { key: 'ArrowRight', altKey: true });
    input.remove();
    expect(window.location.pathname).toBe('/');
  });

  it('mostra só a posição na tarefa única na onda ou sem onda', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '2' }) }, { ...waveSpec, tasks: [summary('TASK-01', '1'), summary('TASK-02', '2')] });
    const alone = await renderTask();
    expect(alone.textContent).toContain('TASK-02 · Onda 2 · 1 de 1');
    expect(within(alone).queryAllByRole('link')).toHaveLength(0);
    expect(within(alone).queryByRole('button', { name: 'Próxima a conferir' })).toBeNull();
    cleanup();
    mockRoutes({ 'TASK-02': taskDetail({ wave: '' }) });
    const none = await renderTask();
    expect(none.textContent).toContain('TASK-02 · Sem onda');
    expect(within(none).queryAllByRole('link')).toHaveLength(0);
    expect(within(none).queryByRole('button', { name: 'Próxima a conferir' })).toBeNull();
  });
});

describe('Próxima a conferir', () => {
  beforeEach(() => { window.scrollTo = vi.fn(); });

  it('pula para a próxima tarefa da onda com evidência a conferir', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': withReviews('TASK-03', reviewed), 'TASK-01': withReviews('TASK-01', toReview) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(window.location.pathname + window.location.hash).toBe('/tarefas/SPEC-001/TASK-01#evidencias'));
    expect(screen.getByRole('tab', { name: 'Evidências' }).getAttribute('aria-selected')).toBe('true');
  });

  it('avisa e fica na tarefa quando nenhuma tem evidência a conferir', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': withReviews('TASK-03', reviewed), 'TASK-01': withReviews('TASK-01', reviewed) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(screen.getByRole('status').textContent).toBe('Nenhuma outra tarefa da onda tem evidência a conferir.'));
    expect(window.location.pathname).toBe('/');
  });

  it('pula a tarefa que não abre e segue para a próxima', async () => {
    mockRoutes({ 'TASK-02': taskDetail({ wave: '1' }), 'TASK-03': 500, 'TASK-01': withReviews('TASK-01', toReview) });
    const bar = await renderTask();
    fireEvent.click(within(bar).getByRole('button', { name: 'Próxima a conferir' }));
    await waitFor(() => expect(window.location.pathname).toBe('/tarefas/SPEC-001/TASK-01'));
  });
});
