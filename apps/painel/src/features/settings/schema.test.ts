import { describe, expect, it } from 'vitest';
import { FILES, humanize } from './schema';

describe('schema', () => {
  it('gives an interface name to every mapped key', () => {
    const fields = FILES.flatMap((file) => file.sections.flatMap((section) => section.fields));
    expect(fields.length).toBeGreaterThan(20);
    for (const field of fields) expect(field.label.trim()).not.toBe('');
    expect(FILES.find((file) => file.name === 'main.yml')?.sections[0].fields[0].label).toBe('Ferramenta de IA');
  });

  it('derives a readable name for an unmapped key', () => {
    expect(humanize('record_section')).toBe('Record section');
  });
});
