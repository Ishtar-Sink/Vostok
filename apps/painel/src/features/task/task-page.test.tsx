import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { TaskPage } from './task-page';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const state = (over: object = {}) => ({ id: 'TASK-02', title: 'Criar a tela', description: 'Descrição', status: 'in_progress', wave: '2', profile: 'padrao', persona: 'dev', model: 'm', depends_on: 'TASK-01', covers: 'RF-01', approved_at: '', attempts: '0', tests_result: '', regression_result: '', evidence_result: '', ...over });
const taskDetail = (over: object = {}, evidence: object[] = []) => ({
  spec: 'SPEC-001',
  state: state(over),
  files: { executivo: '## Resumo\nTexto executivo.', tecnico: '## T\nTexto técnico.', criterios: '## C\n- [ ] item', execucao: '## E\nPassos.', verificacao: '## V\nComandos.', registro: '## R\nLinhas.' },
  evidence,
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
});
