import { marked, type Tokens } from 'marked';
import { useLayoutEffect, useMemo, useRef } from 'react';
import { linkReferences, type ReferenceIndex } from './references';
import styles from './markdown.module.css';

export interface Heading {
  id: string;
  text: string;
  depth: number;
}

export function slug(text: string): string {
  return text
    .toLowerCase()
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/[`*_]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}

/** Títulos de nível 2 e 3, para o índice; os IDs batem com os do HTML gerado. */
export function headingsOf(source: string, prefix = ''): Heading[] {
  return marked
    .lexer(source)
    .filter((token): token is Tokens.Heading => token.type === 'heading' && token.depth >= 2 && token.depth <= 3)
    .map((token) => ({ id: prefix + slug(token.text), text: token.text.replace(/[`*]/g, ''), depth: token.depth }));
}

const LINK_PROTOCOLS = ['http:', 'https:', 'mailto:'];
const LOCAL_BASE = 'http://caminho-local.invalid/';

function escapeHtml(text: string): string {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** O `marked` mantém as entidades no destino e o navegador as decodifica: a conferência vale para o destino decodificado. */
function allowedHref(raw: string): string | null {
  const decoder = document.createElement('textarea');
  decoder.innerHTML = raw;
  const href = decoder.value;
  try {
    return LINK_PROTOCOLS.includes(new URL(href, LOCAL_BASE).protocol) ? href : null;
  } catch {
    return null;
  }
}

function render(source: string, prefix: string, untrusted: boolean): string {
  const renderer = new marked.Renderer();
  const table = renderer.table;
  const tablecell = renderer.tablecell;
  renderer.heading = function (this: typeof renderer, { tokens, depth, text }: Tokens.Heading) {
    return `<h${depth} id="${prefix}${slug(text)}">${this.parser.parseInline(tokens)}</h${depth}>`;
  };
  renderer.table = function (this: typeof renderer, token: Tokens.Table) {
    return `<div class="${styles.tableWrap}">${table.call(this, token)}</div>`;
  };
  renderer.tablecell = function (this: typeof renderer, token: Tokens.TableCell) {
    const html = tablecell.call(this, token);
    return !token.header && token.text.trim() === '-' ? html.replace('<td', '<td data-empty=""') : html;
  };
  if (untrusted) {
    const text = renderer.text;
    renderer.html = ({ text: raw, block }: Tokens.HTML | Tokens.Tag) => (block ? `<p>${escapeHtml(raw)}</p>` : escapeHtml(raw));
    // Depois de uma tag como `<pre>` na mesma linha, o `marked` marca o texto como já escapado e o devolve cru.
    renderer.text = function (this: typeof renderer, token: Tokens.Text | Tokens.Escape) {
      return text.call(this, { ...token, escaped: false });
    };
    renderer.image = ({ text: alternative }: Tokens.Image) => escapeHtml(alternative);
    renderer.link = function (this: typeof renderer, { href, raw, tokens }: Tokens.Link) {
      const allowed = allowedHref(href);
      return allowed === null ? escapeHtml(raw) : `<a href="${escapeHtml(allowed)}">${this.parser.parseInline(tokens)}</a>`;
    };
    renderer.heading = function (this: typeof renderer, { tokens, depth }: Tokens.Heading) {
      return `<h${depth}>${this.parser.parseInline(tokens)}</h${depth}>`;
    };
  }
  return marked.parse(source, { async: false, gfm: true, renderer });
}

/** Markdown renderizado. O modo padrão confia no conteúdo e serve aos documentos do harness; texto digitado
 *  ou devolvido pelo agente, que pode repetir um anexo, usa `untrusted`: HTML e link perigoso viram texto, e
 *  imagem vira o texto alternativo. Com um índice de referências, os IDs conhecidos viram botões que o
 *  popover da página abre. */
export function Markdown({ source, prefix = '', references, bind, untrusted = false }: { source: string; prefix?: string; references?: ReferenceIndex; bind?: (element: HTMLElement | null) => void; untrusted?: boolean }) {
  const html = useMemo(() => render(source, prefix, untrusted), [source, prefix, untrusted]);
  const ref = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = html;
    if (references) linkReferences(ref.current, references);
    bind?.(ref.current);
  }, [html, references, bind]);

  return <div ref={ref} className={styles.root} />;
}
