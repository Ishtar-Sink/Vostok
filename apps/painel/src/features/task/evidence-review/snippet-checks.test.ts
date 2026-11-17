import { describe, expect, it } from 'vitest';
import { checksOf, failureChecksOf, snippetsOf } from './snippet-checks';

const LOG = '$ cmd\n# data: d\n# commit: c\n# código de saída: 0\ntest a ... ok\ntest b ... ok\n0 failed\nFAILED x\nFAILED y';

describe('snippetsOf', () => {
  it('devolve os textos entre crases, na ordem', () => {
    expect(snippetsOf('as linhas `a` e `b c`.')).toEqual(['a', 'b c']);
    expect(snippetsOf(undefined)).toEqual([]);
    expect(snippetsOf('sem crase')).toEqual([]);
  });
});

describe('checksOf', () => {
  it('marca o Trecho achado como bom e o não achado como ruim', () => {
    expect(checksOf(LOG, ['test a ... ok', 'nada'], true)).toEqual([
      { id: '0', label: 'test a ... ok', result: 'apareceu · 1 linha', tone: 'good', lines: [4] },
      { id: '1', label: 'nada', result: 'não apareceu', tone: 'bad', lines: [] },
    ]);
  });

  it('marca a Falha achada como ruim e a não achada como boa', () => {
    expect(checksOf(LOG, ['FAILED', 'panicked'], false)).toEqual([
      { id: '0', label: 'FAILED', result: 'apareceu · 2 linhas', tone: 'bad', lines: [7, 8] },
      { id: '1', label: 'panicked', result: 'não apareceu', tone: 'good', lines: [] },
    ]);
  });

  it('usa o índice como id mesmo com textos repetidos', () => {
    expect(checksOf(LOG, ['FAILED', 'FAILED'], false).map((check) => check.id)).toEqual(['0', '1']);
  });

  it('avisa o texto que não dá para buscar', () => {
    expect(snippetsOf('linhas `+` com `origem \\`painel\\``')).toEqual(['+', 'origem \\']);
    const [plus, escaped, times, found] = checksOf(LOG, ['+', 'origem \\', '×', 'test a ... ok'], true);
    expect(plus).toEqual({ id: '0', label: '+', result: 'não dá para buscar: símbolo solto', tone: 'bad', lines: [] });
    expect(escaped).toEqual({ id: '1', label: 'origem \\', result: 'não dá para buscar: crase dentro do texto', tone: 'bad', lines: [] });
    expect(times.result).toBe('não apareceu');
    expect(found.result).toBe('apareceu · 1 linha');
  });
});

describe('failureChecksOf', () => {
  const log = (line: string) => `$ cmd\n# data: d\n# commit: c\n# código de saída: 0\n${line}`;

  it('mostra a Falha contida no Trecho como não dá para buscar, em tom neutro', () => {
    const checks = failureChecksOf(log('mp4_referencia=152'), 'se `mp4_referencia` diferente de 152', '`mp4_referencia=152`');
    expect(checks).toEqual([
      { id: '0', label: 'mp4_referencia', result: 'não dá para buscar: está no Trecho', tone: 'neutral', lines: [] },
    ]);
  });

  it('mostra a Falha com condição como não dá para buscar', () => {
    const checks = failureChecksOf(log('rm_na_referencia=0'), 'se `rm_na_referencia` for maior que 0', undefined);
    expect(checks[0]).toMatchObject({ result: 'não dá para buscar: condição sobre o valor', tone: 'neutral', lines: [] });
  });

  it('mantém a Falha literal como antes', () => {
    const checks = failureChecksOf(log('tudo certo'), 'se aparecer `FAILED`', '`tudo certo`');
    expect(checks[0]).toMatchObject({ result: 'não apareceu', tone: 'good' });
  });
});
