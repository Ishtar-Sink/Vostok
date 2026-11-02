import { describe, expect, it } from 'vitest';
import { parseRoute } from './router';

describe('parseRoute', () => {
  it.each([
    ['/', { page: 'dashboard' }],
    ['/pendencias', { page: 'inbox' }],
    ['/specs', { page: 'specs' }],
    ['/specs/SPEC-029', { page: 'spec', id: 'SPEC-029', view: 'documento' }],
    ['/specs/SPEC-029/quadro', { page: 'spec', id: 'SPEC-029', view: 'quadro' }],
    ['/specs/SPEC-029/plano', { page: 'spec', id: 'SPEC-029', view: 'plano' }],
    ['/planos/SPEC-029', { page: 'spec', id: 'SPEC-029', view: 'plano' }],
    ['/tarefas', { page: 'tasks' }],
    ['/tarefas/SPEC-029/TASK-03', { page: 'task', spec: 'SPEC-029', task: 'TASK-03' }],
    ['/documentos', { page: 'documents' }],
    ['/configuracoes', { page: 'settings' }],
  ])('maps %s to the right page', (path, route) => {
    expect(parseRoute(path)).toEqual(route);
  });

  it('falls back to the document view for an unknown spec view', () => {
    expect(parseRoute('/specs/SPEC-029/outra')).toEqual({ page: 'spec', id: 'SPEC-029', view: 'documento' });
  });

  it.each(['/nada', '/tarefas/SPEC-029', '/pendencias/x', '/planos', '/specs/a/b/c'])('treats %s as missing', (path) => {
    expect(parseRoute(path)).toEqual({ page: 'missing' });
  });
});
