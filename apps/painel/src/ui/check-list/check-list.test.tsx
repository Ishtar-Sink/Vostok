import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { CheckList, type CheckItem } from './check-list';

afterEach(cleanup);

const ITEMS: CheckItem[] = [
  { id: '0', label: 'test a ... ok', result: 'apareceu · 1 linha', tone: 'good' },
  { id: '1', label: 'Test Files  1 passed', result: 'não apareceu', tone: 'bad' },
];

describe('CheckList', () => {
  it('mostra um item por linha com o texto e o resultado', () => {
    render(<CheckList label="Trecho esperado" items={ITEMS} onSelect={() => undefined} />);
    const list = screen.getByRole('list', { name: 'Trecho esperado' });
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    const first = within(list).getByRole('button', { name: 'test a ... ok apareceu · 1 linha' });
    expect(first.querySelector('code')?.textContent).toBe('test a ... ok');
  });

  it('marca o tom bom e ruim no item', () => {
    render(<CheckList label="Trecho esperado" items={ITEMS} onSelect={() => undefined} />);
    const buttons = screen.getAllByRole('button');
    expect(buttons[0]?.getAttribute('data-tone')).toBe('good');
    expect(buttons[1]?.getAttribute('data-tone')).toBe('bad');
    const icons = document.querySelectorAll('svg');
    expect(icons).toHaveLength(2);
    icons.forEach((icon) => expect(icon.getAttribute('aria-hidden')).toBe('true'));
  });

  it('chama onSelect com o id e marca o ativo com aria-pressed', () => {
    const onSelect = vi.fn();
    render(<CheckList label="Trecho esperado" items={ITEMS} active="1" onSelect={onSelect} />);
    const buttons = screen.getAllByRole('button');
    expect(buttons[1]?.getAttribute('aria-pressed')).toBe('true');
    expect(buttons[0]?.getAttribute('aria-pressed')).toBe('false');
    fireEvent.click(buttons[0] as HTMLElement);
    expect(onSelect).toHaveBeenCalledWith('0');
  });

  it('mostra os itens sem botão quando não há onSelect', () => {
    render(<CheckList label="Trecho esperado" items={ITEMS} />);
    expect(screen.queryAllByRole('button')).toHaveLength(0);
    expect(screen.getByText('test a ... ok')).toBeTruthy();
    expect(screen.getByText('não apareceu')).toBeTruthy();
  });

  it('mostra a dica no title dos botões', () => {
    render(
      <CheckList
        label="Trecho esperado"
        items={ITEMS}
        itemTitle="Clique para destacar no log e copiar"
        onSelect={() => undefined}
      />,
    );
    screen
      .getAllByRole('button')
      .forEach((button) =>
        expect(button.getAttribute('title')).toBe('Clique para destacar no log e copiar'),
      );
  });

  it('passa className e atributos para a raiz', () => {
    render(<CheckList label="Trecho esperado" items={ITEMS} className="extra" data-testid="lista" />);
    const list = screen.getByTestId('lista');
    expect(list.tagName).toBe('UL');
    expect(list.className).toContain('extra');
  });
});
