import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { fileUrl } from '../../shared/api';
import { TaskPage } from './task-page';

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
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('TaskPage', () => {
  it('shows the human parts before the agent parts', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    const headings = (await screen.findAllByRole('heading', { level: 2 })).map((h) => h.textContent);
    expect(headings.indexOf('Resumo executivo')).toBeLessThan(headings.indexOf('Evidências'));
    expect(headings.indexOf('Evidências')).toBeLessThan(headings.indexOf('Para o agente'));
    expect(screen.getByRole('heading', { level: 3, name: 'Execução' })).toBeTruthy();
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
      { name: 'nota.diff', kind: 'diff', status: 'rejeitada', reason: 'falta o total' },
      { name: 'tela.png', kind: 'print', status: 'a conferir', reason: '' },
      { name: 'extra.txt', kind: 'automática', status: 'sem aval', reason: '' },
    ];
    const reviewsDetail = () => ({ ...taskDetail({}, ['saida.txt', 'nota.diff', 'tela.png', 'extra.txt'].map(entry)), reviews });
    const itemOf = (name: string) => within(screen.getByText(name).closest('li') as HTMLElement);

    it('mostra os quatro estados da conferência', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('saida.txt');
      expect(itemOf('saida.txt').getByText('conferida')).toBeTruthy();
      expect(itemOf('nota.diff').getByText('rejeitada')).toBeTruthy();
      expect(itemOf('tela.png').getByText('a conferir')).toBeTruthy();
      expect(itemOf('extra.txt').getByText('sem aval')).toBeTruthy();
      for (const text of ['conferida', 'rejeitada', 'a conferir', 'sem aval']) expect(screen.getAllByText(text)).toHaveLength(1);
    });

    it('mostra o motivo da rejeição', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('nota.diff');
      expect(itemOf('nota.diff').getByText('Motivo: falta o total')).toBeTruthy();
      expect(screen.getAllByText(/Motivo:/)).toHaveLength(1);
    });

    it('não mostra botão de conferir nem de rejeitar', async () => {
      mockFetch(reviewsDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('saida.txt');
      expect(screen.queryByRole('button', { name: /Aceitar|Rejeitar|Conferir/ })).toBeNull();
    });

    it('sem reviews não mostra selo de conferência', async () => {
      mockFetch(taskDetail({}, ['saida.txt', 'tela.png'].map(entry)));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByText('saida.txt');
      for (const text of ['conferida', 'rejeitada', 'a conferir', 'sem aval']) expect(screen.queryByText(text)).toBeNull();
    });

    it('mostra o selo da evidência manual anexada', async () => {
      const manualItem = { name: 'tela.png', path: 'x/tela.png', size: 3, description: 'Print final.', extension: 'png', attached: true };
      mockFetch({ ...taskDetail({}, [entry('tela.png')]), manual_evidence: [manualItem], reviews: [reviews[2]] });
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      const list = await screen.findByRole('list', { name: 'Evidências que o harness pede' });
      expect(within(list).getByText('a conferir')).toBeTruthy();
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
    await screen.findByRole('link', { name: 'Baixar tela.svg' });
    expect(container.querySelector('img')).toBeNull();
    expect(await screen.findByText('a conferir')).toBeTruthy();
    expect(screen.queryByRole('button', { name: /Aceitar|Rejeitar/ })).toBeNull();
  });

  it('pede só o primeiro 1 MB do texto e avisa quando há mais', async () => {
    const calls = mockEvidence(withEvidence([entry('grande.log', 3 * MB)]), partial('linha', `bytes 0-${MB - 1}/${3 * MB}`));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
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
    fireEvent.click(await screen.findByRole('button', { name: /pequeno\.txt/ }));
    expect(await screen.findByText('tudo')).toBeTruthy();
    expect(screen.queryByText(/Mostrando o primeiro/)).toBeNull();
    expect(screen.queryByRole('link', { name: 'Baixar o arquivo inteiro' })).toBeNull();
  });

  it('aceita a resposta sem Content-Range', async () => {
    mockEvidence(withEvidence([entry('nota.txt', 4)]), () => new Response('nota'));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: /nota\.txt/ }));
    expect(await screen.findByText('nota')).toBeTruthy();
    expect(screen.queryByText(/Mostrando o primeiro/)).toBeNull();
  });

  it('mostra o caractere de substituição no texto cortado', async () => {
    mockEvidence(withEvidence([entry('corte.txt', 2)]), partial(new Uint8Array([0x61, 0xc3]), null));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: /corte\.txt/ }));
    expect(await screen.findByText('a�')).toBeTruthy();
  });

  it('não pede o conteúdo de um texto vazio', async () => {
    const calls = mockEvidence(withEvidence([entry('vazio.txt', 0)]));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: /vazio\.txt/ }));
    await waitFor(() => expect(screen.getByRole('button', { name: /vazio\.txt/ }).getAttribute('aria-expanded')).toBe('true'));
    expect(screen.queryByText('Carregando…')).toBeNull();
    expect(filesOf(calls)).toHaveLength(0);
  });

  it('mostra a falha ao ler o texto', async () => {
    mockEvidence(withEvidence([entry('nota.txt', 4)]), () => new Response('erro', { status: 500 }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: /nota\.txt/ }));
    expect(await screen.findByText('Não foi possível ler o arquivo.')).toBeTruthy();
    expect(screen.getByRole('button', { name: /nota\.txt/ })).toBeTruthy();
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
    return render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
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

  it('leva o foco a Evidências com #evidencias', async () => {
    const scroll = vi.fn();
    Element.prototype.scrollIntoView = scroll;
    history.pushState(null, '', '/tarefas/SPEC-001/TASK-02#evidencias');
    const { rerender } = await open(attachDetail(manual()));
    const heading = await screen.findByRole('heading', { level: 2, name: 'Evidências' });
    await waitFor(() => expect(document.activeElement).toBe(heading));
    expect(scroll).toHaveBeenCalledTimes(1);
    const taskFetches = () => vi.mocked(fetch).mock.calls.filter(([url]) => String(url).startsWith('/api/tarefas')).length;
    const before = taskFetches();
    rerender(<TaskPage spec="SPEC-001" task="TASK-02" version={1} />);
    await waitFor(() => expect(taskFetches()).toBeGreaterThan(before));
    expect(scroll).toHaveBeenCalledTimes(1);
  });
});
