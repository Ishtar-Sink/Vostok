import { useEffect, useRef, useState } from 'react';
import type { KeyboardEvent, PointerEvent } from 'react';
import { clamp, hexToRgb, hslToRgb, rgbToHex, rgbToHsl, type Rgb } from './color';
import styles from './color-field.module.css';

export interface ColorFieldLabels {
  title: string;
  lightness: string;
  hex: string;
  rgb: string;
  red: string;
  green: string;
  blue: string;
  hsl: string;
  hue: string;
  saturation: string;
  cancel: string;
  apply: string;
  /** Nome da roda de matiz e saturação. */
  wheel: string;
}

export interface ColorPickerProps {
  /** Cor inicial (HEX); o seletor guarda o próprio estado depois disso. */
  value: string;
  onChange: (hex: string) => void;
  labels: ColorFieldLabels;
}

interface State {
  h: number;
  s: number;
  l: number;
  rgb: Rgb;
}

const WHEEL = 220;

function fromRgb(rgb: Rgb): State {
  const [h, s, l] = rgbToHsl(...rgb);
  return { h, s, l, rgb };
}

function fromHsl(h: number, s: number, l: number): State {
  return { h, s, l, rgb: hslToRgb(h, s, l) };
}

export function ColorPicker({ value, onChange, labels }: ColorPickerProps) {
  const [state, setState] = useState<State>(() => fromRgb(hexToRgb(value) ?? [124, 58, 237]));
  const [editing, setEditing] = useState<{ key: string; text: string } | null>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const hex = rgbToHex(state.rgb);

  function update(next: State) {
    setState(next);
    onChange(rgbToHex(next.rgb));
  }

  useEffect(() => {
    const context = canvas.current?.getContext('2d');
    if (!context) return;
    const r = WHEEL / 2;
    const image = context.createImageData(WHEEL, WHEEL);
    for (let y = 0; y < WHEEL; y++) {
      for (let x = 0; x < WHEEL; x++) {
        const dx = x - r;
        const dy = y - r;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const i = (y * WHEEL + x) * 4;
        if (dist > r) continue;
        const rgb = hslToRgb(
          ((Math.atan2(dy, dx) * 180) / Math.PI + 360) % 360,
          (dist / r) * 100,
          state.l,
        );
        image.data[i] = rgb[0];
        image.data[i + 1] = rgb[1];
        image.data[i + 2] = rgb[2];
        image.data[i + 3] = 255;
      }
    }
    context.putImageData(image, 0, 0);
  }, [state.l]);

  function pick(event: PointerEvent<HTMLCanvasElement>) {
    const box = event.currentTarget.getBoundingClientRect();
    const x = ((event.clientX - box.left) / box.width) * 2 - 1;
    const y = ((event.clientY - box.top) / box.height) * 2 - 1;
    const h = Math.round(((Math.atan2(y, x) * 180) / Math.PI + 360) % 360);
    const s = Math.round(clamp(Math.sqrt(x * x + y * y), 0, 1) * 100);
    update(fromHsl(h, s, state.l));
  }

  function onWheelKey(event: KeyboardEvent<HTMLCanvasElement>) {
    const step = event.shiftKey ? 10 : 1;
    const { key } = event;
    if (key === 'ArrowRight' || key === 'ArrowLeft') {
      const h = (state.h + (key === 'ArrowRight' ? step : -step) + 360) % 360;
      update(fromHsl(h, state.s, state.l));
    } else if (key === 'ArrowUp' || key === 'ArrowDown') {
      update(
        fromHsl(state.h, clamp(state.s + (key === 'ArrowUp' ? step : -step), 0, 100), state.l),
      );
    } else {
      return;
    }
    event.preventDefault();
  }

  const radius = WHEEL / 2;
  const angle = (state.h * Math.PI) / 180;
  const dist = (state.s / 100) * radius;
  const marker = {
    left: `${String(((radius + Math.cos(angle) * dist) / WHEEL) * 100)}%`,
    top: `${String(((radius + Math.sin(angle) * dist) / WHEEL) * 100)}%`,
    backgroundColor: hex,
  };

  const hex_invalid = editing?.key === 'hex' && hexToRgb(editing.text) === null;
  const num = (text: string, max: number) => clamp(Math.round(Number(text) || 0), 0, max);

  function shown(key: string, derived: string | number): string {
    return editing?.key === key ? editing.text : String(derived);
  }

  function numberField(
    key: string,
    label: string,
    derived: number,
    apply: (n: number) => void,
    max: number,
  ) {
    return (
      <label className={styles.numberLabel}>
        <span>{label}</span>
        <input
          type="number"
          className={styles.input}
          min={0}
          max={max}
          value={shown(key, derived)}
          onChange={(event) => {
            setEditing({ key, text: event.target.value });
            apply(num(event.target.value, max));
          }}
          onBlur={() => {
            setEditing(null);
          }}
        />
      </label>
    );
  }

  const [r, g, b] = state.rgb;
  const setRgb = (index: 0 | 1 | 2) => (n: number) => {
    const next: Rgb = [...state.rgb];
    next[index] = n;
    update(fromRgb(next));
  };

  return (
    <div className={styles.picker}>
      <div className={styles.wheelWrap}>
        <canvas
          ref={canvas}
          width={WHEEL}
          height={WHEEL}
          className={styles.wheel}
          role="slider"
          tabIndex={0}
          aria-label={labels.wheel}
          aria-valuemin={0}
          aria-valuemax={360}
          aria-valuenow={state.h}
          aria-valuetext={`${labels.hue} ${String(state.h)} graus, ${labels.saturation.toLowerCase()} ${String(state.s)}%`}
          onPointerDown={(event) => {
            event.currentTarget.setPointerCapture(event.pointerId);
            pick(event);
          }}
          onPointerMove={(event) => {
            if (event.currentTarget.hasPointerCapture(event.pointerId)) pick(event);
          }}
          onKeyDown={onWheelKey}
        />
        <span className={styles.marker} style={marker} aria-hidden="true" />
      </div>
      <div className={styles.controls}>
        <span
          className={styles.preview}
          data-preview=""
          style={{ backgroundColor: hex }}
          aria-hidden="true"
        />
        <label className={styles.numberLabel}>
          <span>{labels.lightness}</span>
          <input
            type="range"
            min={0}
            max={100}
            className={styles.range}
            value={state.l}
            onChange={(event) => {
              update(fromHsl(state.h, state.s, Number(event.target.value)));
            }}
          />
        </label>
        <label className={styles.numberLabel}>
          <span>{labels.hex}</span>
          <input
            type="text"
            className={styles.input}
            autoComplete="off"
            value={shown('hex', hex)}
            aria-invalid={hex_invalid ? true : undefined}
            onChange={(event) => {
              setEditing({ key: 'hex', text: event.target.value });
              const rgb = hexToRgb(event.target.value);
              if (rgb) update(fromRgb(rgb));
            }}
            onBlur={() => {
              setEditing(null);
            }}
          />
        </label>
        <fieldset className={styles.group}>
          <legend>{labels.rgb}</legend>
          {numberField('r', labels.red, r, setRgb(0), 255)}
          {numberField('g', labels.green, g, setRgb(1), 255)}
          {numberField('b', labels.blue, b, setRgb(2), 255)}
        </fieldset>
        <fieldset className={styles.group}>
          <legend>{labels.hsl}</legend>
          {numberField(
            'h',
            labels.hue,
            state.h,
            (n) => {
              update(fromHsl(n % 360, state.s, state.l));
            },
            360,
          )}
          {numberField(
            's',
            labels.saturation,
            state.s,
            (n) => {
              update(fromHsl(state.h, n, state.l));
            },
            100,
          )}
          {numberField(
            'l',
            labels.lightness,
            state.l,
            (n) => {
              update(fromHsl(state.h, state.s, n));
            },
            100,
          )}
        </fieldset>
      </div>
    </div>
  );
}
