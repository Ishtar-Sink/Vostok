import { cleanup, render } from '@testing-library/preact';
import { afterEach, describe, expect, it } from 'vitest';
import { Markdown } from './markdown';

afterEach(cleanup);

describe('Markdown', () => {
  it('envolve a tabela num contêiner com rolagem', () => {
    const { container } = render(<Markdown source={'| ID | Requisito |\n|---|---|\n| RF-01 | Exportar |\n'} />);
    const table = container.querySelector('table')!;
    expect(table.parentElement!.className).toMatch(/tableWrap/);
    expect(table.parentElement!.parentElement).toBe(container.firstElementChild);
  });

  it('envolve cada tabela do documento', () => {
    const source = '| A |\n|---|\n| 1 |\n\nTexto\n\n| B |\n|---|\n| 2 |\n';
    const { container } = render(<Markdown source={source} />);
    const tables = container.querySelectorAll('table');
    expect(tables).toHaveLength(2);
    tables.forEach((table) => expect(table.parentElement!.className).toMatch(/tableWrap/));
  });

  it('apaga a célula com só um hífen', () => {
    const { container } = render(<Markdown source={'| # | Pendência | Opções |\n|---|---|---|\n| - | Nenhuma | - |\n'} />);
    const cells = Array.from(container.querySelectorAll('td'));
    cells.filter((cell) => cell.textContent === '-').forEach((cell) => expect(cell.hasAttribute('data-empty')).toBe(true));
    expect(cells.filter((cell) => cell.hasAttribute('data-empty'))).toHaveLength(2);
    expect(cells.find((cell) => cell.textContent === 'Nenhuma')!.hasAttribute('data-empty')).toBe(false);
  });

  it('não apaga o cabeçalho nem o texto com hífen', () => {
    const { container } = render(<Markdown source={'| - | Nome |\n|---|---|\n| RF-01 | a - b |\n'} />);
    container.querySelectorAll('th').forEach((cell) => expect(cell.hasAttribute('data-empty')).toBe(false));
    container.querySelectorAll('td').forEach((cell) => expect(cell.hasAttribute('data-empty')).toBe(false));
  });

  it('mantém os títulos com âncora', () => {
    const { container } = render(<Markdown prefix="x-" source={'## Casos de teste\n\n| ID |\n|---|\n| CT-01 |\n'} />);
    expect(container.querySelector('h2')!.id).toBe('x-casos-de-teste');
  });
});

const ALLOWED_TAGS = new Set(['DIV', 'P', 'A', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6', 'STRONG', 'EM', 'DEL', 'CODE', 'PRE', 'BLOCKQUOTE', 'UL', 'OL', 'LI', 'INPUT', 'HR', 'BR', 'TABLE', 'THEAD', 'TBODY', 'TR', 'TH', 'TD']);
const ALLOWED_PROTOCOLS = ['http:', 'https:', 'mailto:'];

describe('Markdown em modo não confiável', () => {
  it.each([
    '<img src="x" onerror="window.__ataque=1">',
    '<script>window.__ataque=2</script>',
    '[clique](javascript:window.__ataque=3)',
    '[a](JaVaScRiPt:window.__ataque=4)',
    '[a](&#106;avascript:window.__ataque=5)',
    '[a](&#x6A;avascript:window.__ataque=6)',
    '[a](&#106avascript:window.__ataque=7)',
    '[a](javascript&colon;window.__ataque=8)',
    '[a](java&#9;script:window.__ataque=9)',
    '<javascript:window.__ataque=10>',
    '[a][ref]\n\n[ref]: javascript:window.__ataque=11',
    '[a](data:text/html,x)',
    '[a](vbscript:x)',
    '[a](file:///etc/passwd)',
    '<iframe src="http://fora.invalid/"></iframe>',
    '<svg onload="window.__ataque=12"></svg>',
    'texto <b onclick="window.__ataque=13">x</b> texto',
    '<!-- c --><img src="x" onerror="window.__ataque=14">',
    '![foto](http://fora.invalid/a.png)',
    '[![foto](http://fora.invalid/a.png)](http://fora.invalid/)',
    'x <pre> <img src=x onerror=window.__ataque=15 ',
    'x <script> <svg onload=window.__ataque=16 ',
  ])('não deixa elemento perigoso: %s', (source) => {
    const { container } = render(<Markdown untrusted source={source} />);
    container.querySelectorAll('*').forEach((element) => {
      expect(ALLOWED_TAGS.has(element.tagName), element.tagName).toBe(true);
      Array.from(element.attributes).forEach((attribute) => expect(attribute.name.toLowerCase().startsWith('on'), attribute.name).toBe(false));
    });
    container.querySelectorAll('a').forEach((link) => {
      expect(ALLOWED_PROTOCOLS).toContain(new URL(link.getAttribute('href')!, 'http://local.invalid/').protocol);
    });
  });

  it('mostra HTML como texto', () => {
    const { container } = render(<Markdown untrusted source={'<img src="x" onerror="y">\n\ntexto <b onclick="z">x</b> fim'} />);
    expect(container.textContent).toContain('<img src="x" onerror="y">');
    expect(container.textContent).toContain('texto <b onclick="z">x</b> fim');
  });

  it('mostra como texto o que vem depois de uma tag de bloco cru na mesma linha', () => {
    const { container } = render(<Markdown untrusted source="x <pre> <img src=x onerror=y " />);
    expect(container.querySelector('img')).toBeNull();
    expect(container.textContent).toContain('<pre> <img src=x onerror=y');
  });

  it('mostra o link recusado como texto', () => {
    const { container } = render(<Markdown untrusted source="[clique](javascript:x)" />);
    expect(container.querySelector('a')).toBeNull();
    expect(container.textContent).toContain('[clique](javascript:x)');
  });

  it.each([
    ['[a](http://x.invalid/?a=1&b=2)', 'http://x.invalid/?a=1&b=2'],
    ['[a](https://x.invalid/)', 'https://x.invalid/'],
    ['[a](mailto:a@b.invalid)', 'mailto:a@b.invalid'],
    ['[a](/pendencias)', '/pendencias'],
    ['[a](./b.md)', './b.md'],
    ['[a](#c)', '#c'],
    ['<https://x.invalid/>', 'https://x.invalid/'],
  ])('mantém o link %s', (source, href) => {
    const { container } = render(<Markdown untrusted source={source} />);
    expect(container.querySelector('a')!.getAttribute('href')).toBe(href);
  });

  it('troca a imagem pelo texto alternativo', () => {
    const { container } = render(<Markdown untrusted source="![foto <b>x</b>](http://fora.invalid/a.png)" />);
    expect(container.querySelector('img')).toBeNull();
    expect(container.querySelector('b')).toBeNull();
    expect(container.textContent).toContain('foto <b>x</b>');
  });

  it('mantém a formatação comum', () => {
    const source = '## Título\n\n**forte** e `<b>código</b>`\n\n- item\n\n| A |\n|---|\n| 1 |\n';
    const { container } = render(<Markdown untrusted source={source} />);
    expect(container.querySelector('h2')!.textContent).toBe('Título');
    expect(container.querySelector('h2')!.hasAttribute('id')).toBe(false);
    expect(container.querySelector('strong')!.textContent).toBe('forte');
    expect(container.querySelector('code')!.textContent).toBe('<b>código</b>');
    expect(container.querySelector('li')!.textContent).toBe('item');
    expect(container.querySelector('table')!.parentElement!.className).toMatch(/tableWrap/);
  });

  it('não muda o modo padrão', () => {
    const { container } = render(<Markdown source={'## T\n\n<img src="x">\n\n[a](javascript:x)\n\n![foto](http://fora.invalid/a.png)\n'} />);
    expect(container.querySelector('h2')!.id).toBe('t');
    expect(container.querySelectorAll('img')).toHaveLength(2);
    expect(container.querySelector('a')!.getAttribute('href')).toBe('javascript:x');
  });
});
