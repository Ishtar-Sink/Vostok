/** Leitura do YAML do harness para a tela de configurações: o subconjunto que o vostok usa
 *  (escalares, grupos por recuo, listas `- item` ou `[a, b]` e blocos `|`), com os comentários
 *  acima de cada chave como ajuda. */
export interface ConfigNode {
  key: string;
  help: string;
  value?: string;
  inlineComment?: string;
  list?: string[];
  /** Comentário ao lado de cada item da lista, na mesma ordem (vazio quando não há). */
  listComments?: string[];
  children?: ConfigNode[];
}

/** Valor editável de um campo: texto ou lista. */
export type ConfigValue = string | string[];

interface Line {
  indent: number;
  text: string;
}

export function parseConfig(content: string): { intro: string; nodes: ConfigNode[] } {
  const lines: Line[] = content.split('\n').map((raw) => ({ indent: raw.length - raw.trimStart().length, text: raw.trim() }));
  let index = 0;
  const intro: string[] = [];
  while (index < lines.length && (lines[index].text.startsWith('#') || lines[index].text === '')) {
    if (lines[index].text === '' && intro.length > 0) {
      index++;
      break;
    }
    if (lines[index].text.startsWith('#')) intro.push(lines[index].text.replace(/^#\s?/, ''));
    index++;
  }

  function block(indent: number): ConfigNode[] {
    const nodes: ConfigNode[] = [];
    let help: string[] = [];
    while (index < lines.length) {
      const line = lines[index];
      if (line.text === '') {
        help = [];
        index++;
        continue;
      }
      if (line.indent < indent) break;
      if (line.text.startsWith('#')) {
        help.push(line.text.replace(/^#\s?/, ''));
        index++;
        continue;
      }
      const match = /^([A-Za-z0-9_.-]+):\s*(.*)$/.exec(line.text);
      index++;
      if (!match) continue;
      const [, key, rest] = match;
      const [rawValue, comment] = splitComment(rest);
      const node: ConfigNode = { key, help: help.join(' ').replace(/^---\s*|\s*---$/g, '') };
      help = [];
      if (rawValue === '|') {
        node.value = blockScalar(line.indent);
      } else if (/^\[.*\]$/.test(rawValue)) {
        node.list = inlineList(rawValue);
      } else if (rawValue !== '') {
        node.value = unquote(rawValue);
        if (comment) node.inlineComment = comment;
      } else if (index < lines.length && lines[index].text.startsWith('- ')) {
        node.list = [];
        node.listComments = [];
        while (index < lines.length && lines[index].text.startsWith('- ')) {
          const [item, itemComment] = splitComment(lines[index++].text.slice(2));
          node.list.push(unquote(item));
          node.listComments.push(itemComment);
        }
      } else {
        const next = lines.slice(index).find((item) => item.text !== '' && !item.text.startsWith('#'));
        node.children = next && next.indent > line.indent ? block(next.indent) : [];
      }
      nodes.push(node);
    }
    return nodes;
  }

  /** Bloco `|`: as linhas mais recuadas que a chave, sem o recuo comum. */
  function blockScalar(keyIndent: number): string {
    const body: Line[] = [];
    while (index < lines.length && (lines[index].text === '' || lines[index].indent > keyIndent)) body.push(lines[index++]);
    while (body.length > 0 && body[body.length - 1].text === '') body.pop();
    const common = Math.min(...body.filter((item) => item.text !== '').map((item) => item.indent));
    return body.map((item) => (item.text === '' ? '' : ' '.repeat(item.indent - common) + item.text)).join('\n');
  }

  return { intro: intro.join(' '), nodes: block(0) };
}

/** Todos os campos com valor, por caminho com pontos (`paths.specs`). Grupos vazios entram como `''`. */
export function flatten(nodes: ConfigNode[], prefix = ''): Map<string, ConfigNode> {
  const map = new Map<string, ConfigNode>();
  for (const node of nodes) {
    const path = prefix ? `${prefix}.${node.key}` : node.key;
    if (node.children && node.children.length > 0) {
      for (const [childPath, child] of flatten(node.children, path)) map.set(childPath, child);
    } else {
      map.set(path, node);
    }
  }
  return map;
}

export function valueOf(node: ConfigNode | undefined): ConfigValue {
  if (!node) return '';
  return node.list ?? node.value ?? '';
}

function inlineList(text: string): string[] {
  return text.slice(1, -1).split(',').map((item) => unquote(item.trim())).filter(Boolean);
}

function splitComment(text: string): [string, string] {
  let quote = '';
  for (let position = 0; position < text.length; position++) {
    const char = text[position];
    if ((char === '"' || char === "'") && (quote === '' || quote === char)) quote = quote ? '' : char;
    if (!quote && char === '#' && (position === 0 || text[position - 1] === ' ')) {
      return [text.slice(0, position).trim(), text.slice(position + 1).trim()];
    }
  }
  return [text.trim(), ''];
}

function unquote(value: string): string {
  return /^".*"$|^'.*'$/.test(value) ? value.slice(1, -1) : value;
}

interface Region {
  /** Primeira linha da chave e quantas linhas o valor ocupa a partir dela. */
  start: number;
  length: number;
  indent: number;
  kind: 'scalar' | 'inline-list' | 'dash-list' | 'block' | 'empty';
  rawRest: string;
}

/** Localiza cada chave folha do texto, por caminho com pontos, sem tocar em nada. */
function locate(lines: string[]): Map<string, Region> {
  const regions = new Map<string, Region>();
  const stack: { indent: number; key: string }[] = [];
  const indentOf = (text: string) => text.length - text.trimStart().length;
  for (let index = 0; index < lines.length; index++) {
    const text = lines[index].trim();
    if (text === '' || text.startsWith('#')) continue;
    const match = /^([A-Za-z0-9_.-]+):\s*(.*)$/.exec(text);
    if (!match) continue;
    const indent = indentOf(lines[index]);
    while (stack.length > 0 && stack[stack.length - 1].indent >= indent) stack.pop();
    const path = [...stack.map((item) => item.key), match[1]].join('.');
    const [value] = splitComment(match[2]);
    const next = lines.slice(index + 1).find((line) => line.trim() !== '' && !line.trim().startsWith('#'));
    let kind: Region['kind'] = 'scalar';
    let length = 1;
    if (value === '|') {
      kind = 'block';
      while (index + length < lines.length && (lines[index + length].trim() === '' || indentOf(lines[index + length]) > indent)) length++;
      while (length > 1 && lines[index + length - 1].trim() === '') length--;
    } else if (/^\[.*\]$/.test(value)) {
      kind = 'inline-list';
    } else if (value === '' && next?.trim().startsWith('- ')) {
      kind = 'dash-list';
      while (index + length < lines.length && lines[index + length].trim().startsWith('- ')) length++;
    } else if (value === '' && next && indentOf(next) > indent) {
      stack.push({ indent, key: match[1] });
      continue;
    } else if (value === '') {
      kind = 'empty';
    }
    regions.set(path, { start: index, length, indent, kind, rawRest: match[2] });
    index += length - 1;
  }
  return regions;
}

function formatScalar(value: string, original: string): string {
  if (value === '') return original.startsWith('"') || original.startsWith("'") ? original.slice(0, 1) + original.slice(0, 1) : '';
  const quote = original.startsWith("'") ? "'" : original.startsWith('"') ? '"' : '';
  const risky = /^[\s\-?:,[\]{}#&*!|>'"%@`]|[:#]\s|\s#|\s$|^(true|false|null|yes|no)$/i.test(value) || value.includes('\n');
  if (quote === "'") return `'${value.replace(/'/g, "''")}'`;
  if (quote === '"' || risky) return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
  return value;
}

function formatListItem(value: string): string {
  return /[,\]#]|^\s|\s$/.test(value) ? `"${value.replace(/"/g, '\\"')}"` : value;
}

/** Reescreve só as chaves editadas, mantendo comentários, linhas em branco e a ordem do arquivo. */
export function writeConfig(content: string, edits: Record<string, ConfigValue>): string {
  const lines = content.split('\n');
  const regions = locate(lines);
  const replacements = [...Object.entries(edits)]
    .map(([key, value]) => ({ region: regions.get(key), value }))
    .filter((item): item is { region: Region; value: ConfigValue } => !!item.region)
    .sort((a, b) => b.region.start - a.region.start);
  for (const { region, value } of replacements) {
    const pad = ' '.repeat(region.indent);
    const key = lines[region.start].trim().split(':')[0];
    const [, comment] = splitComment(region.rawRest);
    const tail = comment && region.kind !== 'block' ? ` # ${comment}` : '';
    let rewritten: string[];
    if (Array.isArray(value)) {
      if (region.kind === 'inline-list' || region.kind === 'scalar' || region.kind === 'empty') {
        rewritten = [`${pad}${key}: [${value.map(formatListItem).join(', ')}]${tail}`];
      } else {
        const comments = lines.slice(region.start + 1, region.start + region.length).map((line) => splitComment(line.trim().slice(2))[1]);
        const itemPad = ' '.repeat(indentWidth(lines[region.start + 1]));
        rewritten = [`${pad}${key}:`, ...value.map((item, position) => `${itemPad}- ${formatListItem(item)}${comments[position] ? ` # ${comments[position]}` : ''}`)];
      }
    } else if (region.kind === 'block' || value.includes('\n')) {
      const body = lines.slice(region.start + 1, region.start + region.length).find((line) => line.trim() !== '');
      const bodyPad = ' '.repeat(body ? indentWidth(body) : region.indent + 2);
      rewritten = [`${pad}${key}: |`, ...value.split('\n').map((line) => (line === '' ? '' : bodyPad + line))];
    } else {
      const [original] = splitComment(region.rawRest);
      const formatted = formatScalar(value, region.kind === 'scalar' ? original : '');
      rewritten = [`${pad}${key}:${formatted ? ` ${formatted}` : ''}${tail}`];
    }
    lines.splice(region.start, region.length, ...rewritten);
  }
  return lines.join('\n');
}

function indentWidth(line: string): number {
  return line.length - line.trimStart().length;
}
