import { cleanup, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { DocumentsPage } from './documents-page';

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
const group = { meta: { id: 'SPEC-001', title: 'Primeira', updated_at: '2026-01-03' }, docs: [{ name: 'executivo', path: 'd/executivo.html', size: 1 }, { name: 'tecnico', path: 'd/tecnico.html', size: 1 }, { name: 'evidencias', path: 'd/evidencias.html', size: 1 }] };
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

describe('DocumentsPage', () => {
  it('lists the archived specs with the three documents', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json([group]))));
    render(<DocumentsPage version={0} />);
    expect(await screen.findByText('Primeira')).toBeTruthy();
    for (const name of ['Executivo', 'Técnico', 'Evidências']) expect(screen.getByRole('link', { name })).toBeTruthy();
  });

  it('shows the empty state', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json([]))));
    render(<DocumentsPage version={0} />);
    expect(await screen.findByText('Nenhuma spec finalizada')).toBeTruthy();
  });

  it('shows the error state', async () => {
    vi.stubGlobal('fetch', vi.fn(() => Promise.resolve(json({ erro: 'x' }, 500))));
    render(<DocumentsPage version={0} />);
    expect(await screen.findByText('Não foi possível ler os documentos')).toBeTruthy();
  });
});
