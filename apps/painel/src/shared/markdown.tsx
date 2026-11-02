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

function render(source: string, prefix: string): string {
  const renderer = new marked.Renderer();
  renderer.heading = function (this: typeof renderer, { tokens, depth, text }: Tokens.Heading) {
    return `<h${depth} id="${prefix}${slug(text)}">${this.parser.parseInline(tokens)}</h${depth}>`;
  };
  return marked.parse(source, { async: false, gfm: true, renderer });
}

/** Markdown do harness renderizado; o conteúdo vem só dos arquivos locais da pasta. Com um índice
 *  de referências, os IDs conhecidos viram botões que o popover da página abre. */
export function Markdown({ source, prefix = '', references, bind }: { source: string; prefix?: string; references?: ReferenceIndex; bind?: (element: HTMLElement | null) => void }) {
  const html = useMemo(() => render(source, prefix), [source, prefix]);
  const ref = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!ref.current) return;
    ref.current.innerHTML = html;
    if (references) linkReferences(ref.current, references);
    bind?.(ref.current);
  }, [html, references, bind]);

  return <div ref={ref} className={styles.root} />;
}
