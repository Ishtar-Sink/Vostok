import { useEffect, useId, useRef, useState } from 'react';
import { Button } from '../button/button';
import { hexToRgb, rgbToHex } from './color';
import { ColorPicker, type ColorFieldLabels } from './color-picker';
import styles from './color-field.module.css';

export type { ColorFieldLabels } from './color-picker';

export interface ColorFieldProps {
  label: string;
  /** HEX atual. */
  value: string;
  onChange: (hex: string) => void;
  onBlur?: () => void;
  labels: ColorFieldLabels;
  error?: string;
  help?: string;
  id?: string;
  className?: string;
}

export function ColorField({
  label,
  value,
  onChange,
  onBlur,
  labels,
  error,
  help,
  id,
  className,
}: ColorFieldProps) {
  const generated = useId();
  const controlId = id ?? generated;
  const labelId = `${controlId}-label`;
  const valueId = `${controlId}-value`;
  const titleId = `${controlId}-title`;
  const helpId = `${controlId}-help`;
  const errorId = `${controlId}-error`;
  const describedBy = [help ? helpId : null, error ? errorId : null].filter(Boolean).join(' ');
  const trigger = useRef<HTMLButtonElement>(null);
  const dialog = useRef<HTMLDialogElement>(null);
  const [open, setOpen] = useState(false);
  const draft = useRef(value);

  const rgb = hexToRgb(value);
  const shown = rgb ? rgbToHex(rgb) : value;

  useEffect(() => {
    if (!open) return;
    dialog.current?.showModal();
    dialog.current?.querySelector<HTMLElement>('[role="slider"]')?.focus();
  }, [open]);

  function close() {
    setOpen(false);
    trigger.current?.focus();
    onBlur?.();
  }

  return (
    <div className={[styles.root, className].filter(Boolean).join(' ')}>
      <span id={labelId} className={styles.label}>
        {label}
      </span>
      {/* eslint-disable-next-line jsx-a11y/role-supports-aria-props -- mesmo contrato do SelectField */}
      <button
        ref={trigger}
        type="button"
        id={controlId}
        className={styles.trigger}
        aria-haspopup="dialog"
        aria-labelledby={`${labelId} ${valueId}`}
        aria-invalid={error ? true : undefined}
        aria-describedby={describedBy || undefined}
        onClick={() => {
          draft.current = value;
          setOpen(true);
        }}
      >
        <span className={styles.swatch} data-swatch="" style={{ backgroundColor: value }} />
        <span id={valueId} className={styles.hexText}>
          {shown}
        </span>
      </button>
      {open ? (
        <dialog
          ref={dialog}
          aria-labelledby={titleId}
          className={styles.dialog}
          onCancel={(event) => {
            event.preventDefault();
            close();
          }}
        >
          <h2 id={titleId} className={styles.title}>
            {labels.title}
          </h2>
          <ColorPicker
            value={value}
            labels={labels}
            onChange={(hex) => {
              draft.current = hex;
            }}
          />
          <div className={styles.buttons}>
            <Button variant="ghost" onClick={close}>
              {labels.cancel}
            </Button>
            <Button
              variant="primary"
              onClick={() => {
                onChange(draft.current);
                close();
              }}
            >
              {labels.apply}
            </Button>
          </div>
        </dialog>
      ) : null}
      {help ? (
        <p id={helpId} className={styles.help}>
          {help}
        </p>
      ) : null}
      {error ? (
        <p id={errorId} role="alert" className={styles.error}>
          {error}
        </p>
      ) : null}
    </div>
  );
}
