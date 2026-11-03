import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Activity } from '../../shared/api';
import { ActivityPage } from './activity-page';

const activity = (overrides: Partial<Activity>): Activity => ({ date: '2026-11-03', type: 'tarefa', spec: 'SPEC-001', task: 'TASK-01', target: 'TASK-01', step: 'início', result: 'in_progress', detail: 'detalhe', ...overrides });
const ALL = [
  activity({ detail: 'primeira tarefa' }),
  activity({ type: 'pedido', spec: 'SPEC-002', task: '', target: 'PED-1', step: 'pedido', result: 'done', detail: 'um pedido' }),
];

function stubFetch() {
  const calls: string[] = [];
  vi.stubGlobal('fetch', vi.fn((url: string) => {
    calls.push(url);
    const params = new URL(url, 'http://x').searchParams;
    const rows = ALL.filter((item) => (!params.get('tipo') || item.type === params.get('tipo')) && (!params.get('spec') || item.spec === params.get('spec')));
    return Promise.resolve(new Response(JSON.stringify(rows), { status: 200 }));
  }));
  return calls;
}

async function choose(label: string, option: string) {
  fireEvent.click(screen.getByRole('button', { name: new RegExp(label) }));
  fireEvent.click(await screen.findByRole('option', { name: option }));
}

Element.prototype.scrollIntoView = vi.fn();

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('ActivityPage', () => {
  it('lista todas as atividades', async () => {
    const calls = stubFetch();
    render(<ActivityPage version={0} />);
    expect(await screen.findByText('primeira tarefa')).toBeTruthy();
    expect(screen.getByText('um pedido')).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Atividades' })).toBeTruthy();
    expect(calls[0]).toContain('limite=1000');
  });

  it('filtra por tipo', async () => {
    const calls = stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Tipo', 'pedido');
    await waitFor(() => expect(screen.queryByText('primeira tarefa')).toBeNull());
    expect(screen.getByText('um pedido')).toBeTruthy();
    expect(calls.at(-1)).toContain('tipo=pedido');
  });

  it('filtra por spec', async () => {
    const calls = stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Spec', 'SPEC-002');
    await waitFor(() => expect(screen.queryByText('primeira tarefa')).toBeNull());
    expect(calls.at(-1)).toContain('spec=SPEC-002');
  });

  it('mostra a lista vazia com o filtro visível', async () => {
    stubFetch();
    render(<ActivityPage version={0} />);
    await screen.findByText('primeira tarefa');
    await choose('Tipo', 'aval');
    expect(await screen.findByText('Nenhuma atividade com esses filtros.')).toBeTruthy();
    expect(screen.getByRole('button', { name: /Tipo/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Spec/ })).toBeTruthy();
  });
});
