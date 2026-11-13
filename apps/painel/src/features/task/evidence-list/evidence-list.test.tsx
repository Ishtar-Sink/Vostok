import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { EvidenceReview, FileRef, ManualEvidence, TaskDetail } from '../../../shared/api';
import { EvidenceList } from './evidence-list';

const STORAGE_KEY = 'vostok.evidencias.visualizacao';

function review(id: string, name: string, status: EvidenceReview['status'], over: Partial<EvidenceReview> = {}): EvidenceReview {
  return { id, name, kind: 'automática', status, reason: '', reviewed_at: '', ...over };
}

function detail(reviews: EvidenceReview[], evidence: FileRef[] = [], verificacao = '', manual: ManualEvidence[] = []): TaskDetail {
  return {
    spec: 'SPEC-001',
    state: { id: 'TASK-02' },
    files: { verificacao },
    evidence: evidence.length ? evidence : reviews.map((item) => ({ name: item.name, path: `x/${item.name}`, size: 1 })),
    manual_evidence: manual,
    reviews,
  };
}

const MIXED = [
  review('EV-01', 'a.txt', 'a conferir'),
  review('EV-02', 'b.txt', 'a conferir'),
  review('EV-03', 'c.txt', 'recusada', { reason: 'falta o total', reviewed_at: '2020-01-03' }),
  review('EV-04', 'saida.txt', 'conferida', { reviewed_at: '2020-01-02' }),
  review('EV-05', 'tela.png', 'sem aval', { kind: 'print' }),
];

const VERIFICATION = ['### Evidências', '', '- `saida.txt` (automática): `cat x`', '  - Título: Testes do quadro passam', ''].join('\n');

afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.unstubAllGlobals();
});

const summary = () => screen.getByRole('group', { name: 'Filtrar por estado' });
const pill = (name: string) => within(summary()).getByRole('button', { name });
const groupHeadings = () => screen.queryAllByRole('heading', { level: 3 }).map((heading) => (heading.textContent ?? '').replace('›', ''));

describe('EvidenceList', () => {
  it('mostra o resumo com a contagem de cada estado', () => {
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    for (const name of ['2 a conferir', '1 recusada', '2 conferidas']) {
      expect(pill(name).getAttribute('aria-pressed')).toBe('false');
    }
  });

  it('separa as evidências em A conferir, Recusadas e Conferidas, nessa ordem', () => {
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    const headings = groupHeadings();
    expect(headings).toHaveLength(3);
    expect(headings[0]).toContain('A conferir');
    expect(headings[0]).toContain('2');
    expect(headings[1]).toContain('Recusadas');
    expect(headings[2]).toContain('Conferidas');
    const toConfer = screen.getByRole('region', { name: /A conferir/ });
    expect(within(toConfer).getByText('a.txt')).toBeTruthy();
    expect(within(toConfer).queryByText('c.txt')).toBeNull();
  });

  it('recolhe e abre cada categoria', () => {
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    const toggle = screen.getByRole('button', { name: /^A conferir/ });
    expect(toggle.getAttribute('aria-expanded')).toBe('true');
    fireEvent.click(toggle);
    expect(toggle.getAttribute('aria-expanded')).toBe('false');
    expect(screen.getByText('a.txt').closest('[hidden]')).not.toBeNull();
    fireEvent.click(toggle);
    expect(screen.getByText('a.txt').closest('[hidden]')).toBeNull();
  });

  it('mostra na lista o ID, o título, o arquivo, o tipo, o estado e a data', () => {
    render(<EvidenceList detail={detail(MIXED, [], VERIFICATION)} headingId="evidencias" />);
    const row = screen.getByText('Testes do quadro passam').closest('button') as HTMLElement;
    for (const text of ['EV-04', 'saida.txt', 'Log', 'Conferida', '02/01/2020']) {
      expect(within(row).getByText(text)).toBeTruthy();
    }
  });

  it('mostra o nome do arquivo quando não há Título', () => {
    render(<EvidenceList detail={detail([review('EV-01', 'a.txt', 'a conferir')])} headingId="evidencias" />);
    const row = screen.getByText('EV-01').closest('button') as HTMLElement;
    expect(within(row).getAllByText(/a\.txt/).length).toBeGreaterThan(0);
  });

  it('mostra os cards com ID, tipo, título, estado e data, sem preview', () => {
    render(<EvidenceList detail={detail(MIXED, [], VERIFICATION)} headingId="evidencias" />);
    fireEvent.click(screen.getByRole('button', { name: 'Cards' }));
    const card = screen.getByText('Testes do quadro passam').closest('button') as HTMLElement;
    for (const text of ['EV-04', 'Log', 'Conferida', '02/01/2020']) expect(within(card).getByText(text)).toBeTruthy();
    const container = card.parentElement as HTMLElement;
    expect(container.querySelector('img, video, pre')).toBeNull();
  });

  it('lembra a visualização escolhida ao recarregar', () => {
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    fireEvent.click(screen.getByRole('button', { name: 'Cards' }));
    expect(localStorage.getItem(STORAGE_KEY)).toBe('cards');
    cleanup();
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    expect(screen.getByRole('button', { name: 'Cards' }).getAttribute('aria-pressed')).toBe('true');
  });

  it('abre na lista com valor salvo inválido ou sem localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'quadro');
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    expect(screen.getByRole('button', { name: 'Lista' }).getAttribute('aria-pressed')).toBe('true');
    cleanup();
    vi.stubGlobal('localStorage', {
      getItem: () => {
        throw new Error('bloqueado');
      },
      setItem: () => {
        throw new Error('bloqueado');
      },
      clear: () => undefined,
    });
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    expect(screen.getByRole('button', { name: 'Lista' }).getAttribute('aria-pressed')).toBe('true');
    expect(() => fireEvent.click(screen.getByRole('button', { name: 'Cards' }))).not.toThrow();
  });

  it('filtra pelas pills, várias de uma vez, e mostra todas sem nenhuma', () => {
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" />);
    fireEvent.click(pill('2 a conferir'));
    expect(pill('2 a conferir').getAttribute('aria-pressed')).toBe('true');
    expect(groupHeadings().map((text) => text.split(/\d/)[0].trim())).toEqual(['A conferir']);
    fireEvent.click(pill('1 recusada'));
    expect(groupHeadings()).toHaveLength(2);
    fireEvent.click(pill('2 a conferir'));
    fireEvent.click(pill('1 recusada'));
    expect(groupHeadings()).toHaveLength(3);
  });

  it('esconde a categoria vazia e mantém o 0 no resumo', () => {
    render(<EvidenceList detail={detail(MIXED.filter((item) => item.status !== 'recusada'))} headingId="evidencias" />);
    expect(pill('0 recusadas')).toBeTruthy();
    expect(screen.queryByRole('heading', { level: 3, name: /Recusadas/ })).toBeNull();
    fireEvent.click(pill('0 recusadas'));
    expect(screen.getByText('Nenhuma evidência nos estados escolhidos.')).toBeTruthy();
  });

  it('mostra a manual sem anexo como falta anexar', () => {
    const manual: ManualEvidence = { name: 'foto.png', path: '', size: 0, description: 'Foto do aparelho.', extension: 'png', attached: false };
    render(<EvidenceList detail={detail([], [], '', [manual])} headingId="evidencias" />);
    const row = screen.getByText(/falta anexar/).closest('button') as HTMLElement;
    expect(within(row).getByText('Manual')).toBeTruthy();
  });

  it('põe na categoria Conferidas a evidência sem aval e a sem conferência', () => {
    render(<EvidenceList detail={detail([review('EV-01', 'tela.png', 'sem aval', { kind: 'print' })], [{ name: 'tela.png', path: 'x', size: 1 }, { name: 'solta.txt', path: 'y', size: 1 }])} headingId="evidencias" />);
    const done = screen.getByRole('region', { name: /Conferidas/ });
    expect(within(done).getAllByText('Sem aval')).toHaveLength(2);
    expect(within(done).getByText('solta.txt')).toBeTruthy();
  });

  it('chama onOpen com o arquivo ao clicar na linha e no card', () => {
    const onOpen = vi.fn();
    render(<EvidenceList detail={detail(MIXED)} headingId="evidencias" onOpen={onOpen} />);
    fireEvent.click(screen.getByText('a.txt').closest('button') as HTMLElement);
    expect(onOpen).toHaveBeenLastCalledWith('a.txt');
    fireEvent.click(screen.getByRole('button', { name: 'Cards' }));
    fireEvent.click(screen.getByText('EV-02').closest('button') as HTMLElement);
    expect(onOpen).toHaveBeenLastCalledWith('b.txt');
  });

  it('mostra o aviso sem evidências e sem resumo', () => {
    render(<EvidenceList detail={detail([])} headingId="evidencias" />);
    expect(screen.getByText('Elas aparecem aqui quando a tarefa é finalizada.')).toBeTruthy();
    expect(screen.queryByRole('group', { name: 'Filtrar por estado' })).toBeNull();
  });
});
