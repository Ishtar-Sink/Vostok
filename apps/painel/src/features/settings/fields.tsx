import { useId, useState } from 'react';
import type { ReactNode } from 'react';
import { Checkbox } from '../../ui/checkbox/checkbox';
import { ColorField } from '../../ui/color-field/color-field';
import type { ColorFieldLabels } from '../../ui/color-field/color-picker';
import { SegmentedField } from '../../ui/segmented-field/segmented-field';
import { SelectField } from '../../ui/select-field/select-field';
import { TagField } from '../../ui/tag-field/tag-field';
import { TextField } from '../../ui/text-field/text-field';
import type { Control, Option } from './schema';
import type { ConfigValue } from './yaml';
import styles from './fields.module.css';

const COLOR_LABELS: ColorFieldLabels = {
  title: 'Escolher a cor',
  wheel: 'Roda de cores',
  lightness: 'Luminosidade',
  hex: 'Hex',
  rgb: 'RGB',
  red: 'Vermelho',
  green: 'Verde',
  blue: 'Azul',
  hsl: 'HSL',
  hue: 'Matiz',
  saturation: 'Saturação',
  cancel: 'Cancelar',
  apply: 'Aplicar',
};

export interface ControlProps {
  label: string;
  help?: string;
  control: Control;
  value: ConfigValue;
  onChange: (value: ConfigValue) => void;
  /** Valor de outro campo, para controles que dependem dele (dígitos do ID, próxima sequência). */
  lookup: (key: string) => ConfigValue;
  setOther: (key: string, value: ConfigValue) => void;
}

const asText = (value: ConfigValue) => (Array.isArray(value) ? value.join(', ') : value);
const asList = (value: ConfigValue) => (Array.isArray(value) ? value : value ? [value] : []);

export function ConfigControl(props: ControlProps) {
  const { label, help, control, value, onChange } = props;
  switch (control.type) {
    case 'text':
    case 'path':
      return (
        <TextField
          label={label}
          help={help}
          value={asText(value)}
          placeholder={control.type === 'text' ? control.placeholder : undefined}
          className={control.type === 'path' || control.mono ? styles.mono : undefined}
          spellCheck={false}
          onChange={(event) => onChange(event.currentTarget.value)}
        />
      );
    case 'number':
      return (
        <Shell label={label} help={help}>
          {(id) => (
            <span className={styles.number}>
              <input id={id} type="number" min={control.min} max={control.max} value={asText(value)} onChange={(event) => onChange(event.currentTarget.value)} />
              {control.unit ? <span className={styles.unit}>{control.unit}</span> : null}
            </span>
          )}
        </Shell>
      );
    case 'range':
      return (
        <Shell label={label} help={help}>
          {(id) => (
            <span className={styles.range}>
              <input id={id} type="range" min={control.min} max={control.max} step={1} value={asText(value) || control.min} onChange={(event) => onChange(event.currentTarget.value)} />
              <output htmlFor={id}>
                {asText(value)} {control.unit}
              </output>
            </span>
          )}
        </Shell>
      );
    case 'select': {
      const options = withCurrent(control.empty ? [{ value: '', label: control.empty }, ...control.options] : control.options, asText(value));
      return <SelectField label={label} help={help} options={options} value={asText(value)} onChange={onChange} />;
    }
    case 'segmented':
      return <SegmentedField label={label} help={help} options={control.options} value={asText(value)} onChange={onChange} />;
    case 'color':
      return <ColorInput {...props} empty={control.empty} />;
    case 'tags':
      return (
        <TagField
          label={label}
          help={help}
          value={asList(value)}
          onChange={onChange}
          placeholder={control.placeholder ?? 'Novo item e Enter'}
          list_label={`${label} adicionadas`}
          remove_label={(tag) => `Remover ${tag}`}
        />
      );
    case 'paths':
      return <PathList label={label} help={help} value={asList(value)} onChange={onChange} />;
    case 'checklist':
      return <CheckList label={label} help={help} options={control.options} value={asList(value)} onChange={onChange} />;
    case 'combos':
      return <Combos label={label} help={help} options={control.options} value={asList(value)} onChange={onChange} />;
    case 'textarea':
      return (
        <Shell label={label} help={help}>
          {(id) => <textarea id={id} className={[styles.textarea, control.mono ? styles.monoText : ''].join(' ')} rows={control.mono ? 7 : 2} value={asText(value)} onChange={(event) => onChange(event.currentTarget.value)} />}
        </Shell>
      );
    case 'font':
      return (
        <Shell label={label} help={help}>
          {(id) => (
            <span className={styles.font}>
              <input id={id} className={styles.monoInput} value={asText(value)} spellCheck={false} onChange={(event) => onChange(event.currentTarget.value)} />
              <span className={styles.sample} style={{ fontFamily: asText(value) }} aria-hidden="true">
                Painel do Vostok: 0123456789
              </span>
            </span>
          )}
        </Shell>
      );
    case 'id':
      return <IdFormat {...props} digitsKey={control.digits} sequence={control.sequence} />;
  }
}

/** Rótulo e ajuda no estilo dos campos do Atria, para os controles que o Atria não tem. */
function Shell({ label, help, children, group }: { label: string; help?: string; children: (id: string) => ReactNode; group?: boolean }) {
  const id = useId();
  const content = (
    <>
      {group ? <legend className={styles.label}>{label}</legend> : <label htmlFor={id} className={styles.label}>{label}</label>}
      {children(id)}
      {help ? <p className={styles.help}>{help}</p> : null}
    </>
  );
  return group ? <fieldset className={styles.shell}>{content}</fieldset> : <div className={styles.shell}>{content}</div>;
}

function withCurrent(options: Option[], value: string): Option[] {
  return value && !options.some((option) => option.value === value) ? [...options, { value, label: value }] : options;
}

function ColorInput({ label, help, value, onChange, empty }: ControlProps & { empty?: string }) {
  const text = asText(value);
  if (!text) {
    return (
      <Shell label={label} help={help}>
        {(id) => (
          <span className={styles.emptyColor}>
            <span>{empty ?? 'Sem cor'}</span>
            <button id={id} type="button" className={styles.linkButton} onClick={() => onChange('#6d28d9')}>
              Escolher uma cor
            </button>
          </span>
        )}
      </Shell>
    );
  }
  return (
    <div className={styles.colorRow}>
      <ColorField label={label} help={help} value={text} onChange={onChange} labels={COLOR_LABELS} />
      {empty ? (
        <button type="button" className={styles.linkButton} onClick={() => onChange('')}>
          {empty}
        </button>
      ) : null}
    </div>
  );
}

/** Lista ordenada de caminhos: subir, descer, remover e acrescentar. */
function PathList({ label, help, value, onChange }: { label: string; help?: string; value: string[]; onChange: (value: string[]) => void }) {
  const [draft, setDraft] = useState('');
  const move = (from: number, to: number) => {
    const next = [...value];
    const [item] = next.splice(from, 1);
    next.splice(to, 0, item);
    onChange(next);
  };
  const add = () => {
    const item = draft.trim();
    if (item && !value.includes(item)) onChange([...value, item]);
    setDraft('');
  };
  return (
    <Shell label={label} help={help} group>
      {(id) => (
        <div className={styles.paths}>
          {value.length === 0 ? <p className={styles.empty}>Nenhum arquivo além do obrigatório.</p> : null}
          <ol className={styles.pathList}>
            {value.map((item, index) => (
              <li key={item}>
                <span className={styles.order}>{index + 1}</span>
                <code>{item}</code>
                <span className={styles.rowActions}>
                  <button type="button" aria-label={`Subir ${item}`} disabled={index === 0} onClick={() => move(index, index - 1)}>↑</button>
                  <button type="button" aria-label={`Descer ${item}`} disabled={index === value.length - 1} onClick={() => move(index, index + 1)}>↓</button>
                  <button type="button" aria-label={`Remover ${item}`} onClick={() => onChange(value.filter((other) => other !== item))}>×</button>
                </span>
              </li>
            ))}
          </ol>
          <span className={styles.addRow}>
            <input
              id={id}
              aria-label={`Acrescentar a ${label}`}
              className={styles.monoInput}
              placeholder="rules/project/stack.md"
              value={draft}
              onChange={(event) => setDraft(event.currentTarget.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  add();
                }
              }}
            />
            <button type="button" className={styles.linkButton} onClick={add} disabled={!draft.trim()}>
              Acrescentar
            </button>
          </span>
        </div>
      )}
    </Shell>
  );
}

/** Opções marcáveis que guardam a lista na ordem das opções; valores desconhecidos continuam no fim. */
function CheckList({ label, help, options, value, onChange }: { label: string; help?: string; options: Option[]; value: string[]; onChange: (value: string[]) => void }) {
  const all: Option[] = [...options, ...value.filter((item) => !options.some((option) => option.value === item)).map((item) => ({ value: item, label: item }))];
  const toggle = (item: string, on: boolean) => {
    const set = new Set(value);
    if (on) set.add(item);
    else set.delete(item);
    onChange(all.map((option) => option.value).filter((option) => set.has(option)));
  };
  return (
    <Shell label={label} help={help} group>
      {() => (
        <div className={styles.checks}>
          {all.map((option) => (
            <div key={option.value} className={styles.check}>
              <Checkbox label={option.label} checked={value.includes(option.value)} onChange={(event) => toggle(option.value, event.currentTarget.checked)} />
              {option.help ? <span className={styles.checkHelp}>{option.help}</span> : null}
            </div>
          ))}
        </div>
      )}
    </Shell>
  );
}

/** Combinações de fatos (`[a, b]`): cada linha é uma combinação, com uma caixa por fato. */
function Combos({ label, help, options, value, onChange }: { label: string; help?: string; options: Option[]; value: string[]; onChange: (value: string[]) => void }) {
  const rows = value.map((item) => item.replace(/[[\]]/g, '').split(',').map((part) => part.trim()).filter(Boolean));
  const save = (next: string[][]) => onChange(next.filter((row) => row.length > 0).map((row) => `[${row.join(', ')}]`));
  return (
    <Shell label={label} help={help} group>
      {() => (
        <div className={styles.combos}>
          {rows.map((row, index) => (
            <div key={index} className={styles.combo}>
              <span className={styles.comboTitle}>Combinação {index + 1}</span>
              <div className={styles.comboChecks}>
                {options.map((option) => (
                  <Checkbox
                    key={option.value}
                    label={option.label}
                    checked={row.includes(option.value)}
                    onChange={(event) => save(rows.map((other, at) => (at !== index ? other : event.currentTarget.checked ? [...other, option.value] : other.filter((fact) => fact !== option.value))))}
                  />
                ))}
              </div>
              <button type="button" className={styles.linkButton} onClick={() => save(rows.filter((_, at) => at !== index))}>
                Remover
              </button>
            </div>
          ))}
          <button type="button" className={styles.linkButton} onClick={() => onChange([...value, `[${options[0].value}]`])}>
            Acrescentar combinação
          </button>
        </div>
      )}
    </Shell>
  );
}

/** Prefixo e dígitos do ID, com a prévia do próximo (`SPEC-028`). */
function IdFormat({ label, help, value, onChange, lookup, setOther, digitsKey, sequence }: ControlProps & { digitsKey: string; sequence?: { file: string; key: string } }) {
  const digits = asText(lookup(digitsKey));
  const next = sequence ? Number(asText(lookup(`${sequence.file}:${sequence.key}`))) || 1 : 1;
  const sample = `${asText(value)}-${String(next).padStart(Number(digits) || 1, '0')}`;
  return (
    <Shell label={label} help={help} group>
      {(id) => (
        <span className={styles.idRow}>
          <input id={id} aria-label="Prefixo" className={styles.monoInput} value={asText(value)} onChange={(event) => onChange(event.currentTarget.value.toUpperCase())} />
          <span className={styles.unit}>com</span>
          <span className={styles.number}>
            <input type="number" aria-label="Dígitos" min={1} max={6} value={digits} onChange={(event) => setOther(digitsKey, event.currentTarget.value)} />
            <span className={styles.unit}>dígitos</span>
          </span>
          <span className={styles.preview}>
            {sequence ? 'Próximo: ' : 'Ex.: '}
            <code>{sample}</code>
          </span>
        </span>
      )}
    </Shell>
  );
}
