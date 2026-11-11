import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { applyRedirect, installScrollRestoration, navigate, parseRoute, readSearchParam, redirectOf, restoreScroll, writeSearchParam } from './router';

describe('parseRoute', () => {
  it.each([
    ['/', { page: 'dashboard' }],
    ['/pendencias', { page: 'missing' }],
    ['/specs', { page: 'specs' }],
    ['/specs/SPEC-029', { page: 'spec', id: 'SPEC-029', view: 'documento' }],
    ['/specs/SPEC-029/quadro', { page: 'tasks' }],
    ['/specs/SPEC-029/plano', { page: 'spec', id: 'SPEC-029', view: 'plano' }],
    ['/planos/SPEC-029', { page: 'spec', id: 'SPEC-029', view: 'plano' }],
    ['/tarefas', { page: 'tasks' }],
    ['/tarefas/SPEC-029/TASK-03', { page: 'task', spec: 'SPEC-029', task: 'TASK-03' }],
    ['/documentos', { page: 'documents' }],
    ['/atividades', { page: 'missing' }],
    ['/perguntas', { page: 'missing' }],
    ['/configuracoes', { page: 'missing' }],
  ])('maps %s to the right page', (path, route) => {
    expect(parseRoute(path)).toEqual(route);
  });

  it('leva as páginas removidas para não encontrada', () => {
    for (const path of ['/pendencias', '/perguntas', '/atividades', '/configuracoes']) {
      expect(parseRoute(path)).toEqual({ page: 'missing' });
    }
  });

  it('falls back to the document view for an unknown spec view', () => {
    expect(parseRoute('/specs/SPEC-029/outra')).toEqual({ page: 'spec', id: 'SPEC-029', view: 'documento' });
  });

  it.each(['/nada', '/tarefas/SPEC-029', '/pendencias/x', '/planos', '/specs/a/b/c'])('treats %s as missing', (path) => {
    expect(parseRoute(path)).toEqual({ page: 'missing' });
  });
});

describe('Quadro único', () => {
  afterEach(() => history.replaceState(null, '', '/'));

  it('aponta o quadro antigo da spec para o menu Tarefas', () => {
    expect(redirectOf('/specs/SPEC-001/quadro')).toBe('/tarefas?spec=SPEC-001');
    expect(redirectOf('/specs/SPEC-001')).toBeNull();
    expect(redirectOf('/tarefas')).toBeNull();
    expect(redirectOf('/specs/SPEC-001/plano')).toBeNull();
  });

  it('troca o endereço sem criar entrada no histórico', () => {
    history.pushState(null, '', '/specs/SPEC-001/quadro');
    const before = history.length;
    applyRedirect();
    expect(location.pathname + location.search).toBe('/tarefas?spec=SPEC-001');
    expect(history.length).toBe(before);
  });

  it('lê e grava a spec do endereço', () => {
    history.replaceState(null, '', '/tarefas?spec=SPEC-002');
    const before = history.length;
    expect(readSearchParam('spec')).toBe('SPEC-002');
    writeSearchParam('spec', 'SPEC-001');
    expect(location.pathname + location.search).toBe('/tarefas?spec=SPEC-001');
    expect(history.length).toBe(before);
    writeSearchParam('spec', null);
    expect(location.pathname + location.search).toBe('/tarefas');
  });
});

describe('Posição de rolagem', () => {
  let uninstall: () => void;

  function back() {
    const popped = new Promise((resolve) => window.addEventListener('popstate', resolve, { once: true }));
    history.back();
    return popped;
  }

  beforeEach(() => {
    window.scrollTo = vi.fn();
    Object.defineProperty(history, 'scrollRestoration', { value: 'auto', configurable: true, writable: true });
    Object.defineProperty(window, 'scrollY', { value: 640, configurable: true, writable: true });
    history.replaceState(null, '', '/tarefas?spec=SPEC-001');
    uninstall = installScrollRestoration();
  });

  afterEach(() => {
    uninstall();
    vi.useRealTimers();
    history.replaceState(null, '', '/');
  });

  it('guarda a rolagem da página ao navegar', async () => {
    navigate('/tarefas/SPEC-001/TASK-01');
    await back();
    expect((history.state as { scroll: number }).scroll).toBe(640);
    expect(location.pathname + location.search).toBe('/tarefas?spec=SPEC-001');
  });

  it('restaura a rolagem ao voltar', async () => {
    navigate('/tarefas/SPEC-001/TASK-01');
    await back();
    expect(window.scrollTo).toHaveBeenCalledWith(0, 640);
  });

  it('tenta de novo até a página ter altura para a rolagem', () => {
    navigate('/tarefas/SPEC-001/TASK-01');
    vi.useFakeTimers();
    (window.scrollTo as ReturnType<typeof vi.fn>).mockClear();
    window.scrollY = 0;
    restoreScroll(640);
    vi.advanceTimersByTime(100);
    expect(window.scrollTo).toHaveBeenCalledTimes(3);
    window.scrollY = 640;
    vi.advanceTimersByTime(500);
    expect(window.scrollTo).toHaveBeenCalledTimes(4);
  });

  it('desiste depois de 40 tentativas com a página sem altura', () => {
    vi.useFakeTimers();
    window.scrollY = 0;
    restoreScroll(640);
    vi.advanceTimersByTime(5000);
    expect(window.scrollTo).toHaveBeenCalledTimes(40);
  });

  it('não restaura sem posição guardada', () => {
    window.dispatchEvent(new PopStateEvent('popstate', { state: null }));
    expect(window.scrollTo).not.toHaveBeenCalled();
  });

  it('mantém a posição guardada ao trocar a spec do endereço', () => {
    history.replaceState({ scroll: 300 }, '', '/tarefas');
    writeSearchParam('spec', 'SPEC-002');
    expect((history.state as { scroll: number }).scroll).toBe(300);
  });

  it('desliga a restauração automática do navegador', () => {
    expect(history.scrollRestoration).toBe('manual');
  });
});
