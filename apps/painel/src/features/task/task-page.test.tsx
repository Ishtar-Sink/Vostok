import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
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
  attach_limit_mb: 500,
});
const specDetail = { meta: { id: 'SPEC-001', title: 'Spec', status: 'planned' }, path: 'p', plan: true, markdown: '# S\n', tasks: [{ id: 'TASK-01', title: 'a', status: 'done', wave: '1', profile: 'padrao', covers: '' }] };

function mockFetch(task: unknown, status = 200) {
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task, status) : json(specDetail))));
}
function mockWithOverview(task: unknown, overview: unknown) {
  vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(task) : url.startsWith('/api/visao') ? json(overview) : json(specDetail))));
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

  it('offers Editar on every part but the log', async () => {
    mockFetch(taskDetail());
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Texto executivo.');
    expect(screen.getAllByRole('button', { name: 'Editar' })).toHaveLength(5);
  });

  it('hides Editar in a done task', async () => {
    mockFetch(taskDetail({ status: 'done' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Texto executivo.');
    expect(screen.queryByRole('button', { name: 'Editar' })).toBeNull();
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
  describe('conferência de prints', () => {
    const PRINT_URL = '/api/conferencias?spec=SPEC-001&tarefa=TASK-02&print=tela.png';
    const printDetail = (over: object = {}, registro = '## R\nLinhas.') => {
      const base = taskDetail(over, [{ name: 'tela.png', path: 'x/tela.png', size: 3 }]);
      return { ...base, files: { ...base.files, verificacao: '## V\n- `tela.png` (print): `cmd`\n', registro } };
    };
    function mockPrint(detail: unknown, review: () => Response = () => json({ conferido: 'tela.png', resultado: 'aceito' })) {
      const fn = vi.fn((url: string) =>
        Promise.resolve(
          url.startsWith('/api/tarefas') ? json(detail) : url.startsWith('/arquivos/') ? new Response('img') : url.startsWith('/api/conferencias') ? review() : json(specDetail),
        ),
      );
      vi.stubGlobal('fetch', fn);
      return fn;
    }
    const reviewCalls = (fn: ReturnType<typeof vi.fn>) => fn.mock.calls.filter(([url]) => String(url).startsWith('/api/conferencias'));

    it('mostra o print a conferir com aceitar e rejeitar', async () => {
      mockPrint(printDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      expect(await screen.findByText('a conferir')).toBeTruthy();
      expect(screen.getByRole('button', { name: 'Aceitar tela.png' })).toBeTruthy();
      expect(screen.getByRole('button', { name: 'Rejeitar tela.png' })).toBeTruthy();
    });

    it('aceita o print pelo painel', async () => {
      const fn = mockPrint(printDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Aceitar tela.png' }));
      await waitFor(() => expect(fn).toHaveBeenCalledWith(PRINT_URL + '&resultado=aceito', expect.objectContaining({ method: 'POST' })));
      expect((await screen.findByRole('status')).textContent).toContain('conferido');
    });

    it('rejeita o print depois de confirmar', async () => {
      const fn = mockPrint(printDetail());
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Rejeitar tela.png' }));
      expect(screen.getByRole('alertdialog')).toBeTruthy();
      fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
      expect(screen.queryByRole('alertdialog')).toBeNull();
      expect(reviewCalls(fn)).toHaveLength(0);
      fireEvent.click(screen.getByRole('button', { name: 'Rejeitar tela.png' }));
      fireEvent.click(screen.getByRole('button', { name: 'Rejeitar print' }));
      await waitFor(() => expect(fn).toHaveBeenCalledWith(PRINT_URL + '&resultado=rejeitado', expect.objectContaining({ method: 'POST' })));
    });

    it('mostra o print já conferido sem os botões', async () => {
      const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode('img')));
      const hash = Array.from(digest, (byte) => byte.toString(16).padStart(2, '0')).join('').slice(0, 8);
      mockPrint(printDetail({}, `## R\n| 5 | 2026-11-03T10:00:00Z | conferência | usuário | print tela.png conferido pelo usuário (sha256 ${hash}) |`));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      expect(await screen.findByText('conferido')).toBeTruthy();
      expect(screen.queryByRole('button', { name: 'Aceitar tela.png' })).toBeNull();
    });

    it('mostra a recusa da conferência', async () => {
      mockPrint(printDetail(), () => new Response('não é um print desta tarefa', { status: 422 }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      fireEvent.click(await screen.findByRole('button', { name: 'Aceitar tela.png' }));
      expect((await screen.findByRole('alert')).textContent).toContain('não é um print desta tarefa');
    });

    it('não oferece conferência na tarefa concluída', async () => {
      mockPrint(printDetail({ status: 'done' }));
      render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
      await screen.findByAltText('Evidência tela.png');
      await screen.findByText('a conferir');
      expect(screen.queryByRole('button', { name: 'Aceitar tela.png' })).toBeNull();
    });
  });

  it('pede a finalização da tarefa em validação', async () => {
    mockFetch(taskDetail({ status: 'validating' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Pedir finalização' }));
    fireEvent.click(screen.getAllByRole('button', { name: 'Pedir finalização' }).at(-1) as HTMLElement);
    await waitFor(() => expect(fetch).toHaveBeenCalledWith('/api/pedidos?acao=finalizar-tarefa&spec=SPEC-001&tarefa=TASK-02', expect.objectContaining({ method: 'POST' })));
  });

  it('mostra a finalização em andamento na tarefa', async () => {
    mockWithOverview(taskDetail(), { requests: [{ id: 'PED-4', acao: 'finalizar-tarefa', spec: 'SPEC-001', tarefa: 'TASK-02', estado: 'em_andamento', atualizado_em: '2026-11-03T10:00:00Z' }] });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('Finalizando a TASK-02');
    expect(screen.queryByRole('button', { name: 'Pedir finalização' })).toBeNull();
  });

  it('mostra a espera, o motivo e o link da dependência, sem Decidir', async () => {
    mockFetch(taskDetail({ status: 'blocked', waiting_on: '[TASK-01]', waiting_reason: 'espera o candidato' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    const region = await screen.findByRole('region', { name: 'Espera' });
    expect(within(region).getByText(/Espera/)).toBeTruthy();
    expect(within(region).getByText('espera o candidato')).toBeTruthy();
    expect(within(region).getByRole('link', { name: 'TASK-01' }).getAttribute('href')).toBe('/tarefas/SPEC-001/TASK-01');
    expect(screen.queryByRole('button', { name: 'Decidir' })).toBeNull();
    expect(screen.getByRole('button', { name: 'Ajustar a tarefa' })).toBeTruthy();
  });

  it('bloqueada por falha mostra Decidir, sem a espera', async () => {
    mockFetch(taskDetail({ status: 'blocked', waiting_on: '[]', waiting_reason: '' }));
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    expect(await screen.findByRole('button', { name: 'Decidir' })).toBeTruthy();
    expect(screen.queryByRole('region', { name: 'Espera' })).toBeNull();
  });

  it('mostra a pausa da tarefa com a saída', async () => {
    mockWithOverview({ ...taskDetail(), last_outcome: { id: 'PED-5', acao: 'finalizar-tarefa', desfecho: 'pausado', pergunta: '', saidas: ['ajustar-tarefa'], resultado: 'O print precisa ser refeito.' } }, { requests: [] });
    render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByText('O print precisa ser refeito.');
    expect(within(screen.getByRole('region', { name: 'Estado do pedido' })).getByRole('button', { name: 'Ajustar a tarefa' })).toBeTruthy();
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
    mockEvidence(withEvidence([entry('tela.svg')], '## V\n- `tela.svg` (print): `cmd`\n'));
    const { container } = render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
    await screen.findByRole('link', { name: 'Baixar tela.svg' });
    expect(container.querySelector('img')).toBeNull();
    expect(await screen.findByRole('button', { name: 'Aceitar tela.svg' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Rejeitar tela.svg' })).toBeTruthy();
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
  class FakeXhr {
    static all: FakeXhr[] = [];
    upload: { onprogress: ((event: { lengthComputable: boolean; loaded: number; total: number }) => void) | null } = { onprogress: null };
    method = '';
    url = '';
    headers: Record<string, string> = {};
    body: unknown = null;
    status = 0;
    responseText = '';
    onload: (() => void) | null = null;
    onerror: (() => void) | null = null;
    onabort: (() => void) | null = null;
    constructor() { FakeXhr.all.push(this); }
    open(method: string, url: string) { this.method = method; this.url = url; }
    setRequestHeader(name: string, value: string) { this.headers[name] = value; }
    send(body: unknown) { this.body = body; }
    finish(status: number, text: string) { this.status = status; this.responseText = text; this.onload?.(); }
  }
  const manual = (over: object = {}) => ({ name: 'tela.png', path: 'x/tela.png', size: 0, description: 'Print da tela final.', extension: 'png', attached: false, ...over });
  const attachDetail = (item: object, stateOver: object = {}, limit = 500) => {
    const base = taskDetail(stateOver, (item as { attached: boolean }).attached ? [{ name: (item as { name: string }).name, path: (item as { path: string }).path, size: (item as { size: number }).size }] : []);
    return { ...base, manual_evidence: [item], attach_limit_mb: limit };
  };
  const choose = async (file: File, name = 'tela.png') => {
    const input = await screen.findByLabelText(`Arquivo para ${name}`);
    fireEvent.change(input, { target: { files: [file] } });
  };
  const pngFile = (name = 'foto.PNG', size = 10) => new File([new Uint8Array(size)], name, { type: 'image/png' });
  const open = async (detail: unknown) => {
    vi.stubGlobal('fetch', vi.fn((url: string) => Promise.resolve(url.startsWith('/api/tarefas') ? json(detail) : json(specDetail))));
    return render(<TaskPage spec="SPEC-001" task="TASK-02" version={0} />);
  };

  beforeEach(() => {
    FakeXhr.all = [];
    vi.stubGlobal('XMLHttpRequest', FakeXhr);
  });
  afterEach(() => {
    history.pushState(null, '', '/');
    vi.restoreAllMocks();
  });

  it('mostra o que o harness pede e Anexar na evidência sem arquivo', async () => {
    await open(attachDetail(manual()));
    expect(await screen.findByText('Print da tela final.')).toBeTruthy();
    expect(screen.getByText('falta anexar')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Anexar tela.png' })).toBeTruthy();
    expect(screen.getByRole('list', { name: 'Evidências que o harness pede' })).toBeTruthy();
    expect(screen.queryByText(/aparecem aqui quando a tarefa é finalizada/)).toBeNull();
  });

  it('mostra a imagem e Trocar na evidência com imagem', async () => {
    await open(attachDetail(manual({ attached: true, size: 2048 })));
    expect(await screen.findByAltText('Evidência tela.png')).toBeTruthy();
    expect(screen.getByText('anexado')).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Trocar tela.png' })).toBeTruthy();
    expect(screen.queryByRole('button', { name: 'Anexar tela.png' })).toBeNull();
  });

  it('mostra o link de baixar e Trocar na evidência com zip', async () => {
    await open(attachDetail(manual({ name: 'pacote.zip', path: 'x/pacote.zip', extension: 'zip', attached: true, size: 2 })));
    const link = await screen.findByRole('link', { name: 'Baixar pacote.zip' });
    expect(link.hasAttribute('download')).toBe(true);
    expect(screen.getByRole('button', { name: 'Trocar pacote.zip' })).toBeTruthy();
  });

  it('envia com o progresso, sem desabilitar o botão, e mostra o resultado', async () => {
    const click = vi.spyOn(HTMLInputElement.prototype, 'click');
    await open(attachDetail(manual()));
    const button = (await screen.findByRole('button', { name: 'Anexar tela.png' })) as HTMLButtonElement;
    await choose(pngFile());
    await waitFor(() => expect(FakeXhr.all).toHaveLength(1));
    const xhr = FakeXhr.all[0];
    expect(xhr.method).toBe('PUT');
    expect(xhr.url).toBe('/api/evidencias?spec=SPEC-001&tarefa=TASK-02&evidencia=tela.png&extensao=PNG');
    expect(Object.keys(xhr.headers)).toContain('X-Painel-Token');
    act(() => xhr.upload.onprogress?.({ lengthComputable: true, loaded: 42, total: 100 }));
    const bar = await screen.findByRole('progressbar', { name: 'Enviando tela.png' });
    expect(bar.getAttribute('aria-valuenow')).toBe('42');
    expect(screen.getByText('Enviando… 42%')).toBeTruthy();
    expect(button.disabled).toBe(false);
    fireEvent.click(button);
    expect(click).not.toHaveBeenCalled();
    expect(FakeXhr.all).toHaveLength(1);
    xhr.finish(200, '{}');
    expect(await screen.findByText('foto.PNG anexado como tela.png.')).toBeTruthy();
    expect(screen.queryByRole('progressbar')).toBeNull();
  });

  it('troca a imagem na tela depois do envio', async () => {
    await open(attachDetail(manual({ attached: true, size: 2048 })));
    const before = (await screen.findByAltText('Evidência tela.png')) as HTMLImageElement;
    expect(before.getAttribute('src')).not.toContain('?v=');
    await choose(pngFile(), 'tela.png');
    await waitFor(() => expect(FakeXhr.all).toHaveLength(1));
    FakeXhr.all[0].finish(200, '{}');
    await screen.findByText('foto.PNG anexado como tela.png.');
    const after = (await screen.findByAltText('Evidência tela.png')) as HTMLImageElement;
    expect(after.getAttribute('src')?.endsWith('?v=1')).toBe(true);
  });

  it('não envia acima do limite e avisa', async () => {
    await open(attachDetail(manual(), {}, 1));
    await choose(pngFile('grande.png', 2 * 1024 * 1024));
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('o limite é de 1 MB');
    expect(alert.textContent).toContain('Nada foi enviado.');
    expect(FakeXhr.all).toHaveLength(0);
  });

  it('mostra a recusa e a queda da conexão, mantendo o anexo', async () => {
    await open(attachDetail(manual({ attached: true, size: 2048 })));
    await choose(pngFile());
    await waitFor(() => expect(FakeXhr.all).toHaveLength(1));
    FakeXhr.all[0].finish(400, 'extensão diferente da pedida');
    expect((await screen.findByRole('alert')).textContent).toContain('extensão diferente da pedida');
    expect(screen.getByRole('button', { name: 'Trocar tela.png' })).toBeTruthy();
    await choose(pngFile());
    await waitFor(() => expect(FakeXhr.all).toHaveLength(2));
    act(() => FakeXhr.all[1].onerror?.());
    expect((await screen.findByRole('alert')).textContent).toContain('A conexão caiu');
    expect(screen.getByRole('button', { name: 'Trocar tela.png' })).toBeTruthy();
    expect(screen.getByAltText('Evidência tela.png')).toBeTruthy();
  });

  it('não oferece Anexar fora de execução e validação', async () => {
    for (const status of ['blocked', 'done']) {
      await open(attachDetail(manual(), { status }));
      expect(await screen.findByText('Print da tela final.')).toBeTruthy();
      expect(screen.queryByRole('button', { name: 'Anexar tela.png' })).toBeNull();
      cleanup();
    }
  });

  it('não repete a evidência manual na lista geral', async () => {
    const { container } = await open(attachDetail(manual({ attached: true, size: 2048 })));
    await screen.findByAltText('Evidência tela.png');
    expect(container.querySelectorAll('img')).toHaveLength(1);
  });

  it('aceita jpg e jpeg no seletor', async () => {
    await open(attachDetail(manual({ name: 'foto.jpg', path: 'x/foto.jpg', extension: 'jpg' })));
    expect((await screen.findByLabelText('Arquivo para foto.jpg')).getAttribute('accept')).toBe('.jpg,.jpeg');
    cleanup();
    await open(attachDetail(manual({ name: 'LEIAME', path: 'x/LEIAME', extension: '' })));
    expect((await screen.findByLabelText('Arquivo para LEIAME')).hasAttribute('accept')).toBe(false);
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
