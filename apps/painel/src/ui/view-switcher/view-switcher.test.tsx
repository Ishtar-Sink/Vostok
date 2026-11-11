import { cleanup, fireEvent, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BlocksIcon, ListIcon } from './view-icons';
import { ViewSwitcher } from './view-switcher';

afterEach(cleanup);

const options = [
  { value: 'lista', label: 'Lista', icon: <ListIcon /> },
  { value: 'blocos', label: 'Blocos', icon: <BlocksIcon /> },
];

describe('ViewSwitcher', () => {
  it('marca a visualização ativa com aria-pressed', () => {
    render(<ViewSwitcher label="Visualização" options={options} value="lista" onChange={() => {}} />);
    expect(screen.getByRole('button', { name: 'Lista' }).getAttribute('aria-pressed')).toBe('true');
    expect(screen.getByRole('button', { name: 'Blocos' }).getAttribute('aria-pressed')).toBe('false');
  });

  it('chama onChange com o valor escolhido', () => {
    const onChange = vi.fn();
    render(<ViewSwitcher label="Visualização" options={options} value="lista" onChange={onChange} />);
    fireEvent.click(screen.getByRole('button', { name: 'Blocos' }));
    expect(onChange).toHaveBeenCalledWith('blocos');
  });

  it('mostra só o ícone, nomeado por aria-label e title', () => {
    render(<ViewSwitcher label="Visualização" options={options} value="lista" onChange={() => {}} />);
    const button = screen.getByRole('button', { name: 'Blocos' });
    expect(button.getAttribute('title')).toBe('Blocos');
    expect(button.textContent).toBe('');
  });

  it('põe o ícone direto no botão, sem invólucro', () => {
    render(<ViewSwitcher label="Visualização" options={options} value="lista" onChange={() => {}} />);
    expect(screen.getByRole('button', { name: 'Blocos' }).firstElementChild?.tagName.toLowerCase()).toBe('svg');
  });

  it('passa className, style e atributos data para a raiz', () => {
    render(
      <ViewSwitcher
        label="Visualização"
        options={options}
        value="lista"
        onChange={() => {}}
        className="extra"
        style={{ marginTop: '4px' }}
        data-testid="raiz"
      />,
    );
    const root = screen.getByRole('group', { name: 'Visualização' });
    expect(root.classList.contains('extra')).toBe(true);
    expect(root.style.marginTop).toBe('4px');
    expect(root.getAttribute('data-testid')).toBe('raiz');
  });

  it('desenha os ícones como decorativos', () => {
    render(<ViewSwitcher label="Visualização" options={options} value="lista" onChange={() => {}} />);
    const icons = screen.getByRole('group', { name: 'Visualização' }).querySelectorAll('svg');
    expect(icons).toHaveLength(2);
    icons.forEach((icon) => {
      expect(icon.getAttribute('aria-hidden')).toBe('true');
      expect(icon.getAttribute('focusable')).toBe('false');
      expect(icon.getAttribute('width')).toBe('18');
      expect(icon.getAttribute('height')).toBe('18');
    });
  });
});
