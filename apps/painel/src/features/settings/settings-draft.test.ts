import { describe, expect, it } from 'vitest';
import {
  EMPTY_DRAFT,
  changePersist,
  changeValue,
  hasPending,
  localErrors,
  pendingValues,
  persistOn,
  readValues,
  shownValues,
} from './settings-draft';
import { DEFAULTS } from './settings-fields';

const base = readValues({ ...DEFAULTS });

describe('settingsDraft', () => {
  it('preenche com o padrão a chave ausente ou de tipo errado', () => {
    const values = readValues({ 'tasks.evidence_title_max': 80, 'tasks.regression_alert': 'FAILED', 'outra.chave': 1 });
    expect(values['tasks.evidence_review_kinds']).toEqual(['automática', 'automática, da verificação', 'diff', 'manual', 'print']);
    expect(values['tasks.evidence_title_max']).toBe(80);
    expect(values['tasks.regression_alert']).toEqual(DEFAULTS['tasks.regression_alert']);
    expect('outra.chave' in values).toBe(false);
  });

  it('manda só as chaves que mudaram', () => {
    const extensions = [...(DEFAULTS['documents.text_extensions'] as string[]), 'ndjson'];
    const draft = changeValue(changeValue(EMPTY_DRAFT, 'tasks.evidence_title_max', 80), 'documents.text_extensions', extensions);
    expect(pendingValues(base, draft)).toEqual({ 'tasks.evidence_title_max': 80, 'documents.text_extensions': extensions });
    expect(hasPending(base, draft)).toBe(true);
  });

  it('não tem pendência quando o valor volta ao lido', () => {
    const draft = changeValue(changeValue(EMPTY_DRAFT, 'tasks.evidence_title_max', 80), 'tasks.evidence_title_max', 60);
    expect(pendingValues(base, draft)).toEqual({});
    expect(hasPending(base, draft)).toBe(false);
  });

  it('desligar a persistência esvazia a pasta', () => {
    const withDir = readValues({ ...DEFAULTS, 'tasks.persist_dir': 'docs/evidencias/<SPEC>/<TASK>' });
    expect(persistOn(withDir, EMPTY_DRAFT)).toBe(true);
    const off = changePersist(withDir, EMPTY_DRAFT, false);
    expect(persistOn(withDir, off)).toBe(false);
    expect(pendingValues(withDir, off)).toEqual({ 'tasks.persist_dir': '' });
  });

  it('ligar a persistência sem a pasta pede a pasta', () => {
    const on = changePersist(base, EMPTY_DRAFT, true);
    expect(hasPending(base, on)).toBe(true);
    expect(pendingValues(base, on)).toEqual({});
    expect(localErrors(base, on)).toEqual({ 'tasks.persist_dir': 'Preencha a pasta de destino.' });
    const filled = changeValue(on, 'tasks.persist_dir', 'docs/evidencias/<SPEC>/<TASK>');
    expect(localErrors(base, filled)).toEqual({});
  });

  it('campo numérico vazio pede um número', () => {
    expect(localErrors(base, changeValue(EMPTY_DRAFT, 'tasks.persist_max_mb', null))).toEqual({
      'tasks.persist_max_mb': 'Preencha um número.',
    });
  });

  it('mostra as edições sobre os valores lidos', () => {
    const shown = shownValues(base, changeValue(EMPTY_DRAFT, 'tasks.evidence_title_max', 80));
    expect(shown['tasks.evidence_title_max']).toBe(80);
    expect(shown['tasks.evidence_baseline_timeout']).toBe(120);
  });
});
