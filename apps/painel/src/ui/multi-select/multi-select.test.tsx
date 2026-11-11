import { cleanup, fireEvent, render, screen, within } from '@testing-library/preact';
import { useState } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { MultiSelect } from './multi-select';

const OPTIONS = [{ value: 'a', label: 'Alfa' }, { value: 'b', label: 'Beta' }, { value: 'c', label: 'Gama' }];
const SHORTCUTS = [{ label: 'Ativas', values: ['a'] }, { label: 'Todas', values: ['a', 'b', 'c'] }];

function Harness({ onChange = () => undefined }: { onChange?: (values: string[]) => void }) {
  const [selected, setSelected] = useState(['a']);
  const change = (values: string[]) => { setSelected(values); onChange(values); };
  return <MultiSelect label="Etapas" options={OPTIONS} selected={selected} onChange={change} summary={`${selected.length} de 3`} shortcuts={SHORTCUTS} />;
}

afterEach(cleanup);

describe('MultiSelect', () => {
  it('abre e fecha pelo gatilho, com o resumo no nome', () => {
    render(<Harness />);
    const trigger = screen.getByRole('button', { name: 'Etapas: 1 de 3' });
    expect(trigger.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByRole('group', { name: 'Etapas' })).toBeNull();
    fireEvent.click(trigger);
    expect(trigger.getAttribute('aria-expanded')).toBe('true');
    const menu = within(screen.getByRole('group', { name: 'Etapas' }));
    expect(menu.getAllByRole('checkbox')).toHaveLength(3);
    expect(menu.getAllByRole('button').map((button) => button.textContent)).toEqual(['Ativas', 'Todas']);
    fireEvent.click(trigger);
    expect(screen.queryByRole('group', { name: 'Etapas' })).toBeNull();
  });

  it('marca e desmarca pelo clique e pelo teclado', () => {
    const onChange = vi.fn();
    render(<Harness onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: /^Etapas/ }));
    const beta = screen.getByRole('checkbox', { name: 'Beta' }) as HTMLInputElement;
    expect((screen.getByRole('checkbox', { name: 'Alfa' }) as HTMLInputElement).checked).toBe(true);
    expect(beta.checked).toBe(false);
    fireEvent.click(beta);
    expect(onChange).toHaveBeenLastCalledWith(['a', 'b']);
    fireEvent.click(screen.getByRole('checkbox', { name: 'Alfa' }));
    expect(onChange).toHaveBeenLastCalledWith(['b']);
    screen.getByRole('checkbox', { name: 'Gama' }).focus();
    fireEvent.keyDown(screen.getByRole('checkbox', { name: 'Gama' }), { key: 'Enter' });
    expect(onChange).toHaveBeenLastCalledWith(['b', 'c']);
  });

  it('chama o atalho com os valores dele', () => {
    const onChange = vi.fn();
    render(<Harness onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: /^Etapas/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Todas' }));
    expect(onChange).toHaveBeenLastCalledWith(['a', 'b', 'c']);
    expect(screen.getByRole('button', { name: 'Etapas: 3 de 3' })).toBeTruthy();
  });

  it('fecha com Esc devolvendo o foco ao gatilho e fecha com o clique fora', () => {
    render(<Harness />);
    const trigger = screen.getByRole('button', { name: /^Etapas/ });
    fireEvent.click(trigger);
    fireEvent.keyDown(screen.getByRole('group', { name: 'Etapas' }), { key: 'Escape' });
    expect(screen.queryByRole('group', { name: 'Etapas' })).toBeNull();
    expect(document.activeElement).toBe(trigger);
    fireEvent.click(trigger);
    fireEvent.pointerDown(document.body);
    expect(screen.queryByRole('group', { name: 'Etapas' })).toBeNull();
  });

  it('move o foco com as setas, de atalho em atalho e de caixa em caixa, e fecha com Tab', () => {
    render(<Harness />);
    fireEvent.click(screen.getByRole('button', { name: /^Etapas/ }));
    const menu = screen.getByRole('group', { name: 'Etapas' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Ativas' }));
    fireEvent.keyDown(menu, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Todas' }));
    fireEvent.keyDown(menu, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getByRole('checkbox', { name: 'Alfa' }));
    fireEvent.keyDown(menu, { key: 'ArrowUp' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Todas' }));
    fireEvent.keyDown(menu, { key: 'Tab' });
    expect(screen.queryByRole('group', { name: 'Etapas' })).toBeNull();
  });
});
