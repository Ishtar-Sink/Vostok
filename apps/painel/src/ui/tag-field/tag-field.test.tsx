import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { TagField } from './tag-field';

afterEach(cleanup);

const SUGGESTIONS = ['txt', 'json', 'ndjson', 'log'];
const LABEL = 'Extensões de texto';

const extension = (text: string) => {
  const value = text.trim().replace(/^\.+/, '').toLowerCase();
  return /^[a-z0-9]+$/.test(value) ? value : null;
};

function setup(props: Partial<Parameters<typeof TagField>[0]> = {}) {
  const onChange = vi.fn();
  render(
    <TagField
      label={LABEL}
      values={['txt']}
      suggestions={SUGGESTIONS}
      normalize={extension}
      onChange={onChange}
      {...props}
    />,
  );
  const input = screen.getByRole('combobox', { name: LABEL }) as HTMLInputElement;
  return { onChange, input };
}

describe('TagField', () => {
  it('liga o rótulo ao combobox', () => {
    const { input } = setup();
    expect(input.getAttribute('aria-autocomplete')).toBe('list');
    expect(input.getAttribute('aria-expanded')).toBe('false');
  });

  it('mostra as tags com o botão de remover', () => {
    setup({ values: ['txt', 'log'] });
    const list = screen.getByRole('list', { name: `${LABEL} adicionadas` });
    expect(list.querySelectorAll('li').length).toBe(2);
    expect(screen.getByRole('button', { name: 'Remover txt' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Remover log' })).toBeTruthy();
  });

  it('filtra as sugestões pelo texto sem repetir as adicionadas', () => {
    const { input } = setup();
    fireEvent.input(input, { target: { value: 'js' } });
    expect(input.getAttribute('aria-expanded')).toBe('true');
    const listbox = screen.getByRole('listbox', { name: `Sugestões para ${LABEL}` });
    const names = Array.from(listbox.querySelectorAll('[role="option"]')).map((o) => o.textContent);
    expect(names).toEqual(['json', 'ndjson']);
    expect(names).not.toContain('txt');
  });

  it('marca a sugestão com ArrowDown e a acrescenta com Enter', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'js' } });
    fireEvent.keyDown(input, { key: 'ArrowDown' });
    const option = screen.getByRole('option', { name: 'json' });
    expect(input.getAttribute('aria-activedescendant')).toBe(option.id);
    expect(option.getAttribute('aria-selected')).toBe('true');
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith(['txt', 'json']);
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(input.value).toBe('');
  });

  it('volta ao texto com ArrowUp', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'js' } });
    fireEvent.keyDown(input, { key: 'ArrowDown' });
    fireEvent.keyDown(input, { key: 'ArrowUp' });
    expect(input.hasAttribute('aria-activedescendant')).toBe(false);
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith(['txt', 'js']);
  });

  it('acrescenta o texto normalizado com Enter', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: '.NDJSON' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith(['txt', 'ndjson']);
  });

  it('acrescenta com vírgula', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'csv' } });
    fireEvent.keyDown(input, { key: ',' });
    expect(onChange).toHaveBeenCalledWith(['txt', 'csv']);
  });

  it('separa o texto colado com vírgulas', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'csv, yaml,' } });
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(['txt', 'csv', 'yaml']);
  });

  it('recusa o texto quando normalize devolve null', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'a b' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).not.toHaveBeenCalled();
    expect(input.value).toBe('a b');
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(screen.getByRole('alert').textContent).toBe('Valor não aceito.');
    fireEvent.input(input, { target: { value: 'a' } });
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('não repete uma tag já adicionada', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'TXT' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).not.toHaveBeenCalled();
    expect(input.value).toBe('');
  });

  it('remove a última tag com Backspace no campo vazio', () => {
    const { input, onChange } = setup({ values: ['txt', 'log'] });
    fireEvent.keyDown(input, { key: 'Backspace' });
    expect(onChange).toHaveBeenCalledWith(['txt']);
  });

  it('não remove tag com Backspace quando há texto', () => {
    const { input, onChange } = setup({ values: ['txt', 'log'] });
    fireEvent.input(input, { target: { value: 'md' } });
    fireEvent.keyDown(input, { key: 'Backspace' });
    expect(onChange).not.toHaveBeenCalled();
  });

  it('remove pelo botão e devolve o foco ao campo', () => {
    const { input, onChange } = setup({ values: ['txt', 'log'] });
    fireEvent.click(screen.getByRole('button', { name: 'Remover txt' }));
    expect(onChange).toHaveBeenCalledWith(['log']);
    expect(document.activeElement).toBe(input);
  });

  it('fecha as sugestões com Escape sem limpar o texto', () => {
    const { input } = setup();
    fireEvent.input(input, { target: { value: 'js' } });
    fireEvent.keyDown(input, { key: 'Escape' });
    expect(screen.queryByRole('listbox')).toBeNull();
    expect(input.getAttribute('aria-expanded')).toBe('false');
    expect(input.value).toBe('js');
  });

  it('acrescenta a sugestão clicada', () => {
    const { input, onChange } = setup();
    fireEvent.input(input, { target: { value: 'js' } });
    const option = screen.getByRole('option', { name: 'ndjson' });
    fireEvent.mouseDown(option);
    fireEvent.click(option);
    expect(onChange).toHaveBeenCalledWith(['txt', 'ndjson']);
  });

  it('aceita qualquer texto sem normalize', () => {
    const { input, onChange } = setup({ normalize: undefined });
    fireEvent.input(input, { target: { value: '  Time:  ' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onChange).toHaveBeenCalledWith(['txt', 'Time:']);
  });

  it('liga a ajuda por aria-describedby', () => {
    const { input } = setup({ help: 'Sem ponto, em minúsculas.' });
    const id = input.getAttribute('aria-describedby') ?? '';
    expect(document.getElementById(id)?.textContent).toBe('Sem ponto, em minúsculas.');
  });

  it('mostra o erro com role alert e marca o campo inválido', () => {
    const error = 'A extensão png já está nas de imagem.';
    const { input } = setup({ error });
    expect(screen.getByRole('alert').textContent).toBe(error);
    expect(input.getAttribute('aria-invalid')).toBe('true');
  });

  it('passa className e atributos para a raiz', () => {
    const { input } = setup({ className: 'extra', 'data-testid': 'tags' } as never);
    const root = screen.getByTestId('tags');
    expect(root.classList.contains('extra')).toBe(true);
    expect(root.contains(input)).toBe(true);
  });
});
