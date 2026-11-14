import { useId } from 'react';
import { Checkbox } from '../../ui/checkbox/checkbox';
import { NumberField } from '../../ui/number-field/number-field';
import { Switch } from '../../ui/switch/switch';
import { TagField } from '../../ui/tag-field/tag-field';
import { EVIDENCE_SECTIONS, type DraftValue, type Setting, type SettingKey } from './settings-fields';
import styles from './settings-page.module.css';

export interface EvidenceSettingsProps {
  values: Record<SettingKey, DraftValue>;
  persist: boolean;
  errors: Partial<Record<string, string>>;
  onChange: (key: SettingKey, value: DraftValue) => void;
  onPersistChange: (on: boolean) => void;
}

type FieldProps = Pick<EvidenceSettingsProps, 'values' | 'errors' | 'onChange'>;

function TextSetting({ setting, value, error, onChange }: {
  setting: Extract<Setting, { kind: 'text' }>;
  value: string;
  error?: string;
  onChange: (value: string) => void;
}) {
  const base = useId();
  const inputId = `${base}-input`;
  const helpId = `${base}-help`;
  const errorId = `${base}-error`;
  return (
    <div className={styles.text}>
      <label htmlFor={inputId}>{setting.label}</label>
      <input
        id={inputId}
        type="text"
        spellCheck={false}
        value={value}
        placeholder={setting.placeholder}
        aria-describedby={error ? `${helpId} ${errorId}` : helpId}
        aria-invalid={error ? true : undefined}
        onInput={(event) => onChange(event.currentTarget.value)}
      />
      <p id={helpId}>{setting.help}</p>
      {error ? <p id={errorId} className={styles.error}>{error}</p> : null}
    </div>
  );
}

function ChecksSetting({ setting, selected, error, onChange }: {
  setting: Extract<Setting, { kind: 'checks' }>;
  selected: string[];
  error?: string;
  onChange: (value: string[]) => void;
}) {
  const base = useId();
  const helpId = `${base}-help`;
  const errorId = `${base}-error`;
  return (
    <fieldset className={`${styles.checks} ${styles.wide}`} aria-describedby={error ? `${helpId} ${errorId}` : helpId}>
      <legend>{setting.label}</legend>
      <p id={helpId}>{setting.help}</p>
      <div className={styles.checkGrid}>
        {setting.options.map((option) => (
          <Checkbox
            key={option.value}
            label={option.label}
            help={option.help}
            checked={selected.includes(option.value)}
            onChange={(checked) =>
              onChange(
                setting.options
                  .map((item) => item.value)
                  .filter((value) => (value === option.value ? checked : selected.includes(value))),
              )
            }
          />
        ))}
      </div>
      {selected.length === 0 ? <p className={styles.warning}>Nenhuma evidência vai pedir conferência.</p> : null}
      {error ? <p id={errorId} className={styles.error}>{error}</p> : null}
    </fieldset>
  );
}

function Field({ setting, values, errors, onChange }: FieldProps & { setting: Setting }) {
  const { key } = setting;
  const value = values[key];
  const error = errors[key];
  if (setting.kind === 'checks') {
    return <ChecksSetting setting={setting} selected={value as string[]} error={error} onChange={(next) => onChange(key, next)} />;
  }
  if (setting.kind === 'number') {
    return (
      <NumberField
        label={setting.label}
        help={setting.help}
        unit={setting.unit}
        min={setting.min}
        max={setting.max}
        value={value as number | null}
        error={error}
        onChange={(next) => onChange(key, next)}
      />
    );
  }
  if (setting.kind === 'tags') {
    return (
      <TagField
        className={styles.wide}
        label={setting.label}
        help={setting.help}
        values={value as string[]}
        suggestions={setting.suggestions}
        normalize={setting.normalize}
        error={error}
        onChange={(next) => onChange(key, next)}
      />
    );
  }
  return <TextSetting setting={setting} value={value as string} error={error} onChange={(next) => onChange(key, next)} />;
}

export function EvidenceSettings({ values, persist, errors, onChange, onPersistChange }: EvidenceSettingsProps) {
  return (
    <>
      {EVIDENCE_SECTIONS.map((section) => (
        <section key={section.id} aria-labelledby={`secao-${section.id}`} className={styles.section}>
          <h3 id={`secao-${section.id}`} className={styles.sectionTitle}>{section.title}</h3>
          <p className={styles.sectionHint}>{section.description}</p>
          {section.toggle ? (
            <Switch label={section.toggle.label} help={section.toggle.help} checked={persist} onChange={onPersistChange} />
          ) : null}
          {!section.toggle || persist ? (
            <div className={styles.fields}>
              {section.fields.map((setting) => (
                <Field key={setting.key} setting={setting} values={values} errors={errors} onChange={onChange} />
              ))}
            </div>
          ) : null}
        </section>
      ))}
    </>
  );
}
