import { describe, expect, it } from 'vitest';
import { flatten, parseConfig, valueOf, writeConfig } from './yaml';

const SAMPLE = `# Configurações de exemplo.

# ID da tarefa.
id:
  prefix: TASK # prefixo
  digits: 2

# Máximo de falhas.
max_attempts: 2

task_types:
  - tarefa # padrão
  - historia

inline: [a, b]

quoted: "texto: com dois pontos"

template: |
  linha 1
    linha 2

last: fim
`;

const read = (content: string, key: string) => valueOf(flatten(parseConfig(content).nodes).get(key));

describe('parseConfig', () => {
  it('reads scalars, groups, lists and blocks with the comment as help', () => {
    const flat = flatten(parseConfig(SAMPLE).nodes);
    expect(valueOf(flat.get('id.prefix'))).toBe('TASK');
    expect(flat.get('max_attempts')?.help).toBe('Máximo de falhas.');
    expect(valueOf(flat.get('task_types'))).toEqual(['tarefa', 'historia']);
    expect(valueOf(flat.get('inline'))).toEqual(['a', 'b']);
    expect(valueOf(flat.get('template'))).toBe('linha 1\n  linha 2');
  });
});

describe('writeConfig', () => {
  it('returns the same text when nothing changes', () => {
    expect(writeConfig(SAMPLE, {})).toBe(SAMPLE);
  });

  it('changes only the edited scalar and keeps comments and blank lines', () => {
    const out = writeConfig(SAMPLE, { max_attempts: '3', 'id.prefix': 'TSK' });
    expect(out).toBe(SAMPLE.replace('max_attempts: 2', 'max_attempts: 3').replace('prefix: TASK # prefixo', 'prefix: TSK # prefixo'));
  });

  it('rewrites a dash list and an inline list', () => {
    const out = writeConfig(SAMPLE, { task_types: ['tarefa', 'bug', 'ideia'], inline: ['x'] });
    expect(read(out, 'task_types')).toEqual(['tarefa', 'bug', 'ideia']);
    expect(read(out, 'inline')).toEqual(['x']);
    expect(out).toContain('  - tarefa # padrão');
    expect(out).toContain('last: fim');
  });

  it('rewrites a block scalar keeping the indentation', () => {
    const out = writeConfig(SAMPLE, { template: 'novo\n  recuado' });
    expect(out).toContain('template: |\n  novo\n    recuado\n\nlast: fim');
    expect(read(out, 'template')).toBe('novo\n  recuado');
  });

  it('quotes values that would break the YAML and keeps existing quotes', () => {
    expect(read(writeConfig(SAMPLE, { last: 'a: b' }), 'last')).toBe('a: b');
    expect(writeConfig(SAMPLE, { quoted: 'outro' })).toContain('quoted: "outro"');
  });

  it('clears a scalar to an empty value', () => {
    expect(read(writeConfig(SAMPLE, { last: '' }), 'last')).toBe('');
  });

  it('ignores keys that do not exist in the file', () => {
    expect(writeConfig(SAMPLE, { nao_existe: 'x' })).toBe(SAMPLE);
  });
});
