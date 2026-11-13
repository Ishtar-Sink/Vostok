import { describe, expect, it } from 'vitest';
import type { TaskDetail } from '../../../shared/api';
import { categoryOf, evidenceEntries, evidenceNotes, kindLabel } from './evidence-items';

const verification = (lines: string[]) => ['### Evidências', '', ...lines, '', '### Outra', ''].join('\n');

function detail(over: Partial<TaskDetail> = {}): TaskDetail {
  return {
    spec: 'SPEC-001',
    state: { id: 'TASK-02' },
    files: { verificacao: '' },
    evidence: [],
    manual_evidence: [],
    reviews: [],
    ...over,
  };
}

describe('evidenceNotes', () => {
  it('lê Título, Prova, Espera, Trecho e Falha de cada evidência', () => {
    const notes = evidenceNotes(
      verification([
        '- `saida.txt` (automática, da verificação)',
        '  - Título: Testes do quadro passam',
        '  - Prova: prova da saída',
        '  - Espera: texto esperado',
        '  - Trecho: linha do trecho',
        '  - Falha: linha da falha',
      ]),
    );
    expect(notes.get('saida.txt')).toEqual({
      title: 'Testes do quadro passam',
      proof: 'prova da saída',
      expect: 'texto esperado',
      excerpt: 'linha do trecho',
      failure: 'linha da falha',
    });
  });

  it('ignora os subitens fora da seção Evidências', () => {
    const notes = evidenceNotes(['## V', '- `fora.txt` (automática): `x`', '  - Título: Fora', '', verification(['- `dentro.txt` (automática): `y`'])].join('\n'));
    expect(notes.has('fora.txt')).toBe(false);
    expect(notes.get('dentro.txt')).toEqual({});
  });
});

describe('kindLabel', () => {
  it('rotula Log, Diff, Print, Vídeo e Manual', () => {
    expect(kindLabel('automática', 'a.txt')).toBe('Log');
    expect(kindLabel('automática, da verificação', 'a.txt')).toBe('Log');
    expect(kindLabel('diff', 'a.diff')).toBe('Diff');
    expect(kindLabel('print', 'a.png')).toBe('Print');
    expect(kindLabel('print', 'a.webm')).toBe('Vídeo');
    expect(kindLabel('manual', 'a.mp4')).toBe('Vídeo');
    expect(kindLabel('manual', 'a.zip')).toBe('Manual');
    expect(kindLabel('', 'a.png')).toBe('Print');
    expect(kindLabel('', 'a.txt')).toBe('Log');
  });
});

describe('categoryOf', () => {
  it('põe cada estado na sua categoria', () => {
    expect(categoryOf('a conferir')).toBe('a-conferir');
    expect(categoryOf('recusada')).toBe('recusadas');
    expect(categoryOf('conferida')).toBe('conferidas');
    expect(categoryOf('sem aval')).toBe('conferidas');
    expect(categoryOf(undefined)).toBe('conferidas');
  });
});

describe('evidenceEntries', () => {
  it('segue a ordem das conferências e põe depois os arquivos sem conferência', () => {
    const entries = evidenceEntries(
      detail({
        evidence: [
          { name: 'a.png', path: 'x/a.png', size: 1 },
          { name: 'extra.txt', path: 'x/extra.txt', size: 1 },
          { name: 'b.txt', path: 'x/b.txt', size: 1 },
        ],
        reviews: [
          { id: 'EV-01', name: 'b.txt', kind: 'automática', status: 'a conferir', reason: '' },
          { id: 'EV-02', name: 'a.png', kind: 'print', status: 'conferida', reason: '' },
        ],
      }),
    );
    expect(entries.map((entry) => entry.name)).toEqual(['b.txt', 'a.png', 'extra.txt']);
    expect(entries.map((entry) => entry.id)).toEqual(['EV-01', 'EV-02', '']);
    expect(entries[1].file?.path).toBe('x/a.png');
  });

  it('usa o Título e, sem ele, o nome do arquivo', () => {
    const entries = evidenceEntries(
      detail({
        files: { verificacao: verification(['- `a.txt` (automática): `x`', '  - Título: Título da A', '- `b.txt` (automática): `y`']) },
        reviews: [
          { id: 'EV-01', name: 'a.txt', kind: 'automática', status: 'a conferir', reason: '' },
          { id: 'EV-02', name: 'b.txt', kind: 'automática', status: 'a conferir', reason: '' },
        ],
      }),
    );
    expect(entries.map((entry) => entry.title)).toEqual(['Título da A', 'b.txt']);
  });

  it('não repete a evidência manual anexada', () => {
    const entries = evidenceEntries(
      detail({
        evidence: [{ name: 'foto.png', path: 'x/foto.png', size: 9 }],
        manual_evidence: [{ name: 'foto.png', path: 'x/foto.png', size: 9, description: 'Foto.', extension: 'png', attached: true }],
      }),
    );
    expect(entries).toHaveLength(1);
    expect(entries[0].manual?.attached).toBe(true);
    expect(entries[0].file?.name).toBe('foto.png');
    expect(entries[0].kind).toBe('manual');
  });

  it('mostra a manual sem anexo sem arquivo', () => {
    const entries = evidenceEntries(
      detail({ manual_evidence: [{ name: 'tela.png', path: '', size: 0, description: 'Print da tela.', extension: 'png', attached: false }] }),
    );
    expect(entries).toHaveLength(1);
    expect(entries[0].file).toBeUndefined();
    expect(entries[0].manual?.attached).toBe(false);
  });
});
