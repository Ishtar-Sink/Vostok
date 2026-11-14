import { describe, expect, it } from 'vitest';
import {
  EVIDENCE_SECTIONS,
  REVIEW_KINDS,
  SETTING_KEYS,
  normalizeExtension,
  normalizeSnippet,
} from './settings-fields';

const allFields = EVIDENCE_SECTIONS.flatMap((section) => section.fields);

describe('settingsFields', () => {
  it('cobre as 12 chaves da lista fechada nas quatro seções', () => {
    expect(EVIDENCE_SECTIONS.map((section) => section.id)).toEqual(['conferencia', 'persistencia', 'documentos', 'regressao']);
    expect(EVIDENCE_SECTIONS.map((section) => section.title)).toEqual([
      'Conferência e geração',
      'Persistência no projeto',
      'Evidências nos documentos',
      'Regressão',
    ]);
    expect(allFields.map((field) => field.key).sort()).toEqual([...SETTING_KEYS].sort());
    expect(new Set(allFields.map((field) => field.key)).size).toBe(12);
  });

  it('usa texto livre só na pasta e no guia da persistência', () => {
    expect(allFields.filter((field) => field.kind === 'text').map((field) => field.key)).toEqual([
      'tasks.persist_dir',
      'tasks.persist_readme',
    ]);
  });

  it('tem uma caixa por tipo de evidência e as faixas do contrato', () => {
    expect(REVIEW_KINDS.map((kind) => kind.value)).toEqual(['automática', 'automática, da verificação', 'diff', 'manual', 'print']);
    expect(REVIEW_KINDS.map((kind) => kind.label)).toEqual(['Automática', 'Automática, da verificação', 'Diff', 'Manual', 'Print']);
    const ranges = Object.fromEntries(
      allFields.flatMap((field) => (field.kind === 'number' ? [[field.key, [field.min, field.max, field.unit]]] : [])),
    );
    expect(ranges).toEqual({
      'tasks.evidence_baseline_timeout': [0, 3600, 's'],
      'tasks.evidence_title_max': [20, 200, 'caracteres'],
      'tasks.persist_max_mb': [0, 1024, 'MB'],
      'documents.embed_max_mb': [0, 100, 'MB'],
    });
  });

  it('normaliza a extensão para minúsculas sem ponto', () => {
    expect(normalizeExtension('.NDJSON')).toBe('ndjson');
    expect(normalizeExtension(' Png ')).toBe('png');
  });

  it('recusa a extensão vazia, com espaço ou com outro símbolo', () => {
    for (const text of ['', '   ', 'nd json', 'tar.gz']) expect(normalizeExtension(text)).toBeNull();
  });

  it('aceita o trecho sem espaços nas pontas e recusa o vazio', () => {
    expect(normalizeSnippet('  Elapsed ')).toBe('Elapsed');
    expect(normalizeSnippet('   ')).toBeNull();
  });
});
