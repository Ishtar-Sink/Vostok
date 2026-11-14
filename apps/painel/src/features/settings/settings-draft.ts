import { DEFAULTS, SETTING_KEYS, type DraftValue, type SettingKey, type SettingValue } from './settings-fields';

export type SettingValues = Record<SettingKey, SettingValue>;

export interface Draft {
  edits: Partial<Record<SettingKey, DraftValue>>;
  /** `null` até o interruptor ser tocado: vale o que o arquivo tem. */
  persist: boolean | null;
}

export const EMPTY_DRAFT: Draft = { edits: {}, persist: null };

const sameType = (a: SettingValue, b: SettingValue) => (Array.isArray(a) ? Array.isArray(b) : typeof a === typeof b);

export function readValues(valores: Record<string, SettingValue>): SettingValues {
  const values = {} as SettingValues;
  for (const key of SETTING_KEYS) {
    const read = valores[key];
    values[key] = read !== undefined && sameType(read, DEFAULTS[key]) ? read : DEFAULTS[key];
  }
  return values;
}

export const shownValues = (base: SettingValues, draft: Draft): Record<SettingKey, DraftValue> => ({
  ...base,
  ...draft.edits,
});

export const persistOn = (base: SettingValues, draft: Draft): boolean =>
  draft.persist ?? base['tasks.persist_dir'] !== '';

export const changeValue = (draft: Draft, key: SettingKey, value: DraftValue): Draft => ({
  ...draft,
  edits: { ...draft.edits, [key]: value },
});

export const changePersist = (_base: SettingValues, draft: Draft, on: boolean): Draft =>
  on ? { ...draft, persist: true } : { edits: { ...draft.edits, 'tasks.persist_dir': '' }, persist: false };

export function pendingValues(base: SettingValues, draft: Draft): Partial<Record<SettingKey, DraftValue>> {
  const changed: Partial<Record<SettingKey, DraftValue>> = {};
  for (const key of SETTING_KEYS) {
    const value = draft.edits[key];
    if (value !== undefined && JSON.stringify(value) !== JSON.stringify(base[key])) changed[key] = value;
  }
  return changed;
}

export const hasPending = (base: SettingValues, draft: Draft): boolean =>
  Object.keys(pendingValues(base, draft)).length > 0 || persistOn(base, draft) !== (base['tasks.persist_dir'] !== '');

export function localErrors(base: SettingValues, draft: Draft): Partial<Record<SettingKey, string>> {
  const errors: Partial<Record<SettingKey, string>> = {};
  const pending = pendingValues(base, draft);
  for (const key of SETTING_KEYS) if (pending[key] === null) errors[key] = 'Preencha um número.';
  const dir = shownValues(base, draft)['tasks.persist_dir'];
  if (persistOn(base, draft) && String(dir ?? '').trim() === '') errors['tasks.persist_dir'] = 'Preencha a pasta de destino.';
  return errors;
}
