import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { OpenSpec } from '../shared/api';
import type { Route } from '../shared/router';
import { Sidebar } from './sidebar';

const task = (id: string, status: string) => ({ id, title: id, status, wave: '1' });
const active = [
  { meta: { id: 'SPEC-001', title: 'Exportar' }, plan: true, plan_outdated: false, tasks: [task('TASK-01', 'done'), task('TASK-02', 'approved'), task('TASK-03', 'pending')] },
] as unknown as OpenSpec[];

const sidebar = (props: { route?: Route; onToggleTheme?: () => void } = {}) =>
  render(
    <Sidebar
      name="demo"
      icon=""
      specs={active}
      route={props.route ?? { page: 'dashboard' }}
      theme="light"
      onToggleTheme={props.onToggleTheme ?? (() => {})}
      drawer={false}
      open={false}
      onClose={() => {}}
    />,
  );

afterEach(() => cleanup());

describe('Sidebar só de leitura', () => {
  it('mostra só Painel, Specs, Tarefas e Documentos no menu', () => {
    sidebar();
    const nav = screen.getByRole('navigation', { name: 'Principal' });
    const labels = within(nav)
      .getAllByRole('link')
      .slice(0, 4)
      .map((link) => link.textContent);
    expect(labels).toEqual(['Painel', 'Specs', 'Tarefas', 'Documentos']);
    for (const name of ['Pendências', 'Perguntas', 'Atividades', 'Configurações']) {
      expect(screen.queryByText(name)).toBeNull();
    }
  });

  it('não mostra contagem nem aviso do agente', () => {
    const { container } = sidebar();
    expect(container.querySelector('aside [role="status"]')).toBeNull();
  });

  it('lista as specs em andamento com o progresso', () => {
    sidebar();
    expect(screen.getByText('Em andamento')).toBeTruthy();
    expect(screen.getByLabelText('1 de 3 tarefas concluídas')).toBeTruthy();
  });

  it('marca a página atual', () => {
    sidebar({ route: { page: 'specs' } });
    expect(screen.getByRole('link', { name: 'Specs' }).getAttribute('aria-current')).toBe('page');
    expect(screen.getByRole('link', { name: 'Painel' }).getAttribute('aria-current')).toBeNull();
  });

  it('troca o tema pelo botão do rodapé', () => {
    const onToggleTheme = vi.fn();
    sidebar({ onToggleTheme });
    fireEvent.click(screen.getByRole('button', { name: 'Tema escuro' }));
    expect(onToggleTheme).toHaveBeenCalledTimes(1);
  });
});
