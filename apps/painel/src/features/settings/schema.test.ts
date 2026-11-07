import { describe, expect, it } from 'vitest';
import { FILES, humanize } from './schema';

describe('schema', () => {
  it('gives an interface name to every mapped key', () => {
    const fields = FILES.flatMap((file) => file.sections.flatMap((section) => section.fields));
    expect(fields.length).toBeGreaterThan(20);
    for (const field of fields) expect(field.label.trim()).not.toBe('');
    expect(FILES.find((file) => file.name === 'main.yml')?.sections[0].fields[0].label).toBe('Ferramenta de IA');
  });

  it('mapeia os limites novos e o tempo do aviso', () => {
    const fieldOf = (file: string, key: string) =>
      FILES.find((item) => item.name === file)
        ?.sections.flatMap((section) => section.fields)
        .find((field) => field.key === key);
    expect(fieldOf('main.yml', 'toast_dismiss_seconds')).toMatchObject({
      label: 'Tempo do aviso na tela',
      control: { type: 'number', min: 0, unit: 's' },
    });
    expect(fieldOf('main.yml', 'request_note_warning_minutes')).toMatchObject({
      label: 'Aviso de falta de nota',
      control: { type: 'number', min: 1, unit: 'min' },
    });
    expect(fieldOf('tasks.yml', 'manual_attachment_max_mb')).toMatchObject({
      label: 'Limite de anexo pelo painel',
      control: { type: 'number', min: 1, unit: 'MB' },
    });
  });

  it('derives a readable name for an unmapped key', () => {
    expect(humanize('record_section')).toBe('Record section');
  });
});
