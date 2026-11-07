import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/preact';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { activityLabel } from '../../shared/status';
import { AskPage } from './ask-page';

type Estado = 'pendente' | 'em_andamento' | 'feito' | 'recusado';
const question = (id: string, estado: Estado, over: object = {}) => ({
  id,
  spec: 'SPEC-001',
  estado,
  texto: `Texto de ${id}`,
  resultado: '',
  atualizado_em: '2026-11-03T15:00:00Z',
  lida_em: '',
  ...over,
});
const meta = (id: string, title: string) => ({ id, title, status: 'planned', type: 'feature', author: '', created_at: '', updated_at: '', refined_at: '', approved_at: '', adjustments: '' });
const overview = {
  project: 'p',
  specs: [
    { meta: meta('SPEC-001', 'Primeira'), plan: false, plan_outdated: false, tasks: [], last_outcome: null },
    { meta: meta('SPEC-002', 'Segunda'), plan: false, plan_outdated: false, tasks: [], last_outcome: null },
  ],
  archived: [],
  activity: [],
  questions: [],
  requests: [],
  loop: null,
};
const four = [
  question('PED-001', 'pendente', { spec: '' }),
  question('PED-002', 'em_andamento'),
  question('PED-003', 'recusado', { resultado: 'Recusei: pede uma alteração.' }),
  question('PED-004', 'feito', { resultado: '**Use ponto e vírgula.**' }),
];

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
type Handler = (url: string, init?: RequestInit) => Response | Promise<Response> | undefined;

function mockFetch(list: unknown[] | (() => unknown[]), extra: Handler = () => undefined) {
  const fn = vi.fn((url: string, init?: RequestInit) => {
    const custom = extra(url, init);
    if (custom) return Promise.resolve(custom);
    if (url === '/api/perguntas-livres') return Promise.resolve(json(typeof list === 'function' ? list() : list));
    if (url === '/api/visao') return Promise.resolve(json(overview));
    return Promise.resolve(json({}));
  });
  vi.stubGlobal('fetch', fn);
  return fn;
}
const posts = (fn: ReturnType<typeof mockFetch>, part: string) => fn.mock.calls.filter(([url, init]) => String(url).includes(part) && init?.method === 'POST');

beforeEach(() => {
  FakeXhr.log = [];
  FakeXhr.requests = [];
  FakeXhr.outcomes = [];
  vi.stubGlobal('XMLHttpRequest', FakeXhr);
  Element.prototype.scrollIntoView = vi.fn();
  document.head.innerHTML = '<meta name="painel-token" content="tok-1">';
  window.history.pushState(null, '', '/perguntas');
});
afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

async function ready() {
  render(<AskPage version={0} />);
  await screen.findByRole('heading', { name: 'Perguntas enviadas' });
}
const field = () => screen.getByLabelText('Sua pergunta') as HTMLTextAreaElement;
const send = () => screen.getByRole('button', { name: 'Enviar pergunta' }) as HTMLButtonElement;

type Outcome = { status: number; text: string } | 'error';
class FakeXhr {
  static log: string[] = [];
  static requests: FakeXhr[] = [];
  static outcomes: Outcome[] = [];
  url = '';
  headers: Record<string, string> = {};
  status = 0;
  responseText = '';
  listeners: Record<string, () => void> = {};
  progress: ((event: unknown) => void) | undefined;
  upload = {
    addEventListener: (_type: string, listener: (event: unknown) => void) => {
      this.progress = listener;
    },
  };
  open(_method: string, url: string) {
    this.url = url;
  }
  setRequestHeader(name: string, value: string) {
    this.headers[name] = value;
  }
  addEventListener(type: string, listener: () => void) {
    this.listeners[type] = listener;
  }
  send() {
    const index = FakeXhr.requests.length;
    FakeXhr.requests.push(this);
    FakeXhr.log.push(`PUT ${this.url}`);
    const outcome = FakeXhr.outcomes[index] ?? { status: 200, text: '{}' };
    setTimeout(() => {
      this.progress?.({ lengthComputable: true, loaded: 1, total: 2 });
      if (outcome === 'error') {
        this.listeners.error?.();
        return;
      }
      this.status = outcome.status;
      this.responseText = outcome.text;
      this.listeners.load?.();
    }, 0);
  }
}
const puts = () => FakeXhr.requests.map((request) => request.url);
const draftOf = (url: string) => new URL(url, 'http://x').searchParams.get('rascunho');

function chooseFiles(files: File[]) {
  const input = document.querySelector('input[type=file]') as HTMLInputElement;
  Object.defineProperty(input, 'files', { value: files, configurable: true });
  fireEvent.change(input);
}
const upload = (name: string) => new File([name], name, { type: name.endsWith('.png') ? 'image/png' : '' });
const isPost = (url: unknown, init?: RequestInit) => init?.method === 'POST' && String(url).startsWith('/api/pedidos?');
function postWith(extra: Handler = () => undefined) {
  return mockFetch([], (url, init) => {
    if (isPost(url, init)) FakeXhr.log.push(`POST ${url}`);
    if (url === '/api/pedidos/anexos/limites') return json({ max_mb: 20, max_arquivos: 10, nomes_recusados: [], nomes_aceitos: [] });
    return extra(url, init) ?? (isPost(url, init) ? json({ id: 'PED-005' }) : undefined);
  });
}
async function typeAndAttach(names: string[]) {
  await ready();
  fireEvent.input(field(), { target: { value: 'Pergunta com anexos' } });
  chooseFiles(names.map(upload));
}

describe('AskPage', () => {
  it('envia a pergunta com a spec', async () => {
    const fn = mockFetch([], (url, init) => (init?.method === 'POST' && url.startsWith('/api/pedidos?') ? json({ id: 'PED-005' }) : undefined));
    await ready();
    fireEvent.input(field(), { target: { value: '  Essa spec muda o login?  ' } });
    fireEvent.click(screen.getByRole('button', { name: /Spec \(opcional\)/ }));
    fireEvent.click(await screen.findByRole('option', { name: 'SPEC-002 · Segunda' }));
    fireEvent.click(send());
    await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=perguntar-agente&spec=SPEC-002', expect.objectContaining({ method: 'POST', body: 'Essa spec muda o login?', headers: expect.objectContaining({ 'X-Painel-Token': 'tok-1' }) })));
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(field().value).toBe('');
  });

  it('envia a pergunta sem spec', async () => {
    const fn = mockFetch([], (url, init) => (init?.method === 'POST' && url.startsWith('/api/pedidos?') ? json({ id: 'PED-005' }) : undefined));
    await ready();
    fireEvent.input(field(), { target: { value: 'Qual o formato?' } });
    fireEvent.click(send());
    await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=perguntar-agente', expect.objectContaining({ method: 'POST', body: 'Qual o formato?', headers: expect.objectContaining({ 'X-Painel-Token': 'tok-1' }) })));
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(field().value).toBe('');
  });

  it('o pedido grava o texto digitado em Markdown', async () => {
    const fn = mockFetch([], (url, init) => (init?.method === 'POST' && url.startsWith('/api/pedidos?') ? json({ id: 'PED-005' }) : undefined));
    await ready();
    fireEvent.input(field(), { target: { value: '**forte** e _leve_' } });
    fireEvent.click(screen.getByRole('tab', { name: 'Prévia' }));
    expect(document.querySelector('[role=tabpanel] strong')?.textContent).toBe('forte');
    fireEvent.click(send());
    await waitFor(() => expect(fn).toHaveBeenCalledWith('/api/pedidos?acao=perguntar-agente', expect.objectContaining({ method: 'POST', body: '**forte** e _leve_' })));
  });

  it('desabilita Enviar sem texto', async () => {
    await (async () => {
      mockFetch([]);
      await ready();
    })();
    expect(send().disabled).toBe(true);
    expect(screen.getByText('Escreva a pergunta: o texto continua obrigatório, mesmo com anexos.')).toBeTruthy();
    fireEvent.input(field(), { target: { value: '   ' } });
    expect(send().disabled).toBe(true);
    fireEvent.input(field(), { target: { value: 'oi' } });
    expect(send().disabled).toBe(false);
  });

  it('mantém o texto na recusa', async () => {
    mockFetch([], (url, init) => (init?.method === 'POST' && url.startsWith('/api/pedidos?') ? new Response('Pergunta vazia demais.', { status: 400 }) : undefined));
    await ready();
    fireEvent.input(field(), { target: { value: 'Minha dúvida' } });
    fireEvent.click(send());
    expect((await screen.findByRole('alert')).textContent).toContain('Pergunta vazia demais.');
    expect(field().value).toBe('Minha dúvida');
  });

  it('mantém o texto na falha de rede', async () => {
    mockFetch([], (url, init) => {
      if (init?.method === 'POST') throw new Error('sem rede');
      return undefined;
    });
    await ready();
    fireEvent.input(field(), { target: { value: 'Minha dúvida' } });
    fireEvent.click(send());
    expect((await screen.findByRole('alert')).textContent).toContain('sem rede');
    expect(field().value).toBe('Minha dúvida');
  });

  it('mostra os quatro estados', async () => {
    mockFetch(four);
    await ready();
    const list = await screen.findByRole('list');
    for (const label of ['Na fila', 'Respondendo', 'Respondida', 'Recusada']) expect(within(list).getByText(label)).toBeTruthy();
  });

  it('mostra Não lida só na resposta sem leitura', async () => {
    mockFetch([...four, question('PED-006', 'feito', { resultado: 'ok', lida_em: '2026-11-03T16:00:00Z' })]);
    await ready();
    await screen.findByRole('list');
    expect(screen.getAllByText('Não lida')).toHaveLength(2);
    expect(within(document.getElementById('pergunta-PED-006')!).queryByText('Não lida')).toBeNull();
    expect(within(document.getElementById('pergunta-PED-001')!).queryByText('Não lida')).toBeNull();
  });

  it('marca como lida uma vez ao abrir', async () => {
    const fn = mockFetch(four, (url, init) => (init?.method === 'POST' && url.endsWith('/lida') ? json({}) : undefined));
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /Texto de PED-004/ }));
    await waitFor(() => expect(posts(fn, '/api/pedidos/PED-004/lida')).toHaveLength(1));
    expect(screen.getByRole('heading', { name: 'Resposta' })).toBeTruthy();
    expect(document.querySelector('strong')?.textContent).toBe('Use ponto e vírgula.');
  });

  it('não chama lida na segunda abertura', async () => {
    const fn = mockFetch(four, (url, init) => (init?.method === 'POST' && url.endsWith('/lida') ? json({}) : undefined));
    await ready();
    const button = await screen.findByRole('button', { name: /Texto de PED-004/ });
    fireEvent.click(button);
    await waitFor(() => expect(posts(fn, '/lida')).toHaveLength(1));
    fireEvent.click(button);
    expect(button.getAttribute('aria-expanded')).toBe('false');
    fireEvent.click(button);
    expect(button.getAttribute('aria-expanded')).toBe('true');
    await new Promise((resolve) => setTimeout(resolve, 20));
    expect(posts(fn, '/lida')).toHaveLength(1);
  });

  it('não chama lida na pergunta já lida', async () => {
    const fn = mockFetch([question('PED-004', 'feito', { resultado: 'ok', lida_em: '2026-11-03T16:00:00Z' })]);
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /Texto de PED-004/ }));
    expect(await screen.findByText('Resposta')).toBeTruthy();
    expect(posts(fn, '/lida')).toHaveLength(0);
  });

  it('não chama lida na pergunta na fila', async () => {
    const fn = mockFetch(four);
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /Texto de PED-001/ }));
    expect(await screen.findByText('O agente ainda não respondeu.')).toBeTruthy();
    expect(posts(fn, '/lida')).toHaveLength(0);
  });

  it('mostra o motivo da recusada', async () => {
    mockFetch(four, (url, init) => (init?.method === 'POST' ? json({}) : undefined));
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /Texto de PED-003/ }));
    expect(await screen.findByRole('heading', { name: 'Motivo da recusa' })).toBeTruthy();
    expect(screen.getByText('Recusei: pede uma alteração.')).toBeTruthy();
  });

  it('mostra a resposta como texto não confiável', async () => {
    const hostile = '<img src="x" onerror="y">';
    mockFetch([question('PED-004', 'feito', { texto: hostile, resultado: hostile })], (url, init) => (init?.method === 'POST' ? json({}) : undefined));
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /img src/ }));
    await screen.findByRole('heading', { name: 'Resposta' });
    expect(document.querySelector('main img, img')).toBeNull();
    expect(screen.getAllByText(hostile, { exact: false }).length).toBeGreaterThanOrEqual(2);
  });

  it('avisa se a leitura não for gravada', async () => {
    mockFetch(four, (url, init) => (init?.method === 'POST' && url.endsWith('/lida') ? new Response('', { status: 500 }) : undefined));
    await ready();
    const button = await screen.findByRole('button', { name: /Texto de PED-004/ });
    fireEvent.click(button);
    expect((await screen.findByRole('alert')).textContent).toContain('Não foi possível marcar como lida; o aviso continua.');
    expect(button.getAttribute('aria-expanded')).toBe('true');
  });

  it('abre pela âncora do endereço', async () => {
    const fn = mockFetch(four, (url, init) => (init?.method === 'POST' ? json({}) : undefined));
    window.history.pushState(null, '', '/perguntas#PED-004');
    await ready();
    const button = await screen.findByRole('button', { name: /Texto de PED-004/ });
    await waitFor(() => expect(button.getAttribute('aria-expanded')).toBe('true'));
    await waitFor(() => expect(posts(fn, '/api/pedidos/PED-004/lida')).toHaveLength(1));
    expect(document.activeElement).toBe(button);
  });

  it('mostra o estado vazio', async () => {
    mockFetch([]);
    await ready();
    expect(await screen.findByText('Nenhuma pergunta ainda')).toBeTruthy();
  });

  it('mostra o carregando', () => {
    vi.stubGlobal('fetch', vi.fn(() => new Promise(() => undefined)));
    render(<AskPage version={0} />);
    expect(screen.getAllByRole('status').filter((status) => status.getAttribute('aria-busy') === 'true')).toHaveLength(1);
  });

  it('mostra o erro com tentar de novo', async () => {
    mockFetch([], (url) => (url === '/api/perguntas-livres' ? new Response('', { status: 500 }) : undefined));
    await ready();
    expect(await screen.findByRole('button', { name: 'Tentar de novo' })).toBeTruthy();
  });

  it('dá nome à ação nas atividades', () => {
    expect(activityLabel('perguntar-agente')).toBe('Perguntar ao agente');
  });
});

describe('AskPage com anexos', () => {
  it('envia cada arquivo em ordem, com o progresso, e só depois cria o pedido', async () => {
    const fn = postWith();
    await typeAndAttach(['a.png', 'b.zip']);
    fireEvent.click(send());
    expect(send().disabled).toBe(true);
    await waitFor(() => expect(document.querySelector('[role=progressbar][aria-label="Enviando a.png"]')?.getAttribute('aria-valuenow')).toBe('50'));
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(FakeXhr.log).toHaveLength(3);
    const [first, second, third] = FakeXhr.log;
    expect(first).toMatch(/^PUT \/api\/pedidos\/anexos\?rascunho=[^&]+&nome=a\.png$/);
    expect(second).toMatch(/^PUT \/api\/pedidos\/anexos\?rascunho=[^&]+&nome=b\.zip$/);
    const draft = draftOf(FakeXhr.requests[0]?.url ?? '');
    expect(draftOf(FakeXhr.requests[1]?.url ?? '')).toBe(draft);
    expect(third).toBe(`POST /api/pedidos?acao=perguntar-agente&rascunho=${draft}`);
    expect(FakeXhr.requests[0]?.headers['X-Painel-Token']).toBe('tok-1');
    expect(posts(fn, '/api/pedidos?')).toHaveLength(1);
  });

  it('sem anexos, não usa rascunho', async () => {
    const fn = postWith();
    await ready();
    fireEvent.input(field(), { target: { value: 'Só texto' } });
    fireEvent.click(send());
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(puts()).toHaveLength(0);
    expect(String(posts(fn, '/api/pedidos?')[0]?.[0])).toBe('/api/pedidos?acao=perguntar-agente');
  });

  it('recusa em um dos arquivos mostra o motivo, mantém texto e lista e não cria pedido', async () => {
    FakeXhr.outcomes = [{ status: 200, text: '{}' }, { status: 413, text: 'Anexo grande demais' }];
    const fn = postWith();
    await typeAndAttach(['a.png', 'b.zip', 'c.txt']);
    fireEvent.click(send());
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toBe('Não foi possível enviar b.zip: Anexo grande demais. Nada entrou na fila.');
    expect(puts()).toHaveLength(2);
    expect(posts(fn, '/api/pedidos?')).toHaveLength(0);
    expect(field().value).toBe('Pergunta com anexos');
    expect(document.querySelectorAll('ul[aria-label="Anexos"] li')).toHaveLength(3);
  });

  it('queda na conexão mostra o motivo em linguagem simples', async () => {
    FakeXhr.outcomes = ['error'];
    const fn = postWith();
    await typeAndAttach(['a.png']);
    fireEvent.click(send());
    expect((await screen.findByRole('alert')).textContent).toBe('A conexão caiu durante o envio de a.png. Nada entrou na fila: tente de novo.');
    expect(posts(fn, '/api/pedidos?')).toHaveLength(0);
    expect(field().value).toBe('Pergunta com anexos');
  });

  it('sessão mudada (403) manda copiar o texto e recarregar', async () => {
    FakeXhr.outcomes = [{ status: 403, text: 'Sessão inválida' }];
    postWith();
    await typeAndAttach(['a.png']);
    fireEvent.click(send());
    expect((await screen.findByRole('alert')).textContent).toBe('A sessão do painel mudou. Copie o texto e recarregue a página.');
  });

  it('recusa no POST mostra o motivo e mantém tudo', async () => {
    postWith((url, init) => (isPost(url, init) ? new Response('Pedido recusado', { status: 400 }) : undefined));
    await typeAndAttach(['a.png']);
    fireEvent.click(send());
    expect((await screen.findByRole('alert')).textContent).toBe('Não foi possível criar a pergunta: Pedido recusado.');
    expect(puts()).toHaveLength(1);
    expect(field().value).toBe('Pergunta com anexos');
    expect(document.querySelectorAll('ul[aria-label="Anexos"] li')).toHaveLength(1);
  });

  it('nova tentativa usa outro rascunho e reenvia todos', async () => {
    FakeXhr.outcomes = [{ status: 200, text: '{}' }, { status: 413, text: 'Grande demais' }];
    const fn = postWith();
    await typeAndAttach(['a.png', 'b.zip']);
    fireEvent.click(send());
    await screen.findByRole('alert');
    fireEvent.click(send());
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(puts()).toHaveLength(4);
    const [firstDraft, , secondDraft] = FakeXhr.requests.map((request) => draftOf(request.url));
    expect(secondDraft).not.toBe(firstDraft);
    expect(FakeXhr.requests.slice(2).map((request) => new URL(request.url, 'http://x').searchParams.get('nome'))).toEqual(['a.png', 'b.zip']);
    expect(String(posts(fn, '/api/pedidos?')[0]?.[0])).toContain(`rascunho=${secondDraft}`);
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('limpa o campo e a lista depois de aceito', async () => {
    postWith();
    await typeAndAttach(['a.png']);
    fireEvent.click(send());
    await screen.findByText('Pergunta enviada: PED-005.');
    expect(field().value).toBe('');
    expect(document.querySelectorAll('ul[aria-label="Anexos"] li')).toHaveLength(0);
  });
});

describe('AskPage pergunta aberta com anexos', () => {
  const base = 'vostok/work/pedidos/anexos/PED-004/';
  const attachments = [
    { nome: '01-tela.png', caminho: `${base}01-tela.png`, tamanho: 2048 },
    { nome: '02-foto.webp', caminho: `${base}02-foto.webp`, tamanho: 10 },
    { nome: '03-dados.zip', caminho: `${base}03-dados.zip`, tamanho: 1536 },
    { nome: '04-pagina.html', caminho: `${base}04-pagina.html`, tamanho: 20 },
    { nome: '05-logo.svg', caminho: `${base}05-logo.svg`, tamanho: 30 },
  ];
  async function open(anexos: unknown) {
    mockFetch([question('PED-004', 'feito', { resultado: 'ok', lida_em: 'x', anexos })]);
    await ready();
    fireEvent.click(await screen.findByRole('button', { name: /Texto de PED-004/ }));
    await screen.findByText('Resposta');
  }

  it('mostra imagem na tela e os demais por link de baixar', async () => {
    await open(attachments);
    expect(screen.getByRole('heading', { name: 'Anexos' })).toBeTruthy();
    const images = [...document.querySelectorAll('img')];
    expect(images.map((image) => image.getAttribute('src'))).toEqual([`/arquivos/${base}01-tela.png`, `/arquivos/${base}02-foto.webp`]);
    expect(images[0]?.getAttribute('alt')).toBe('01-tela.png');
    const links = [...document.querySelectorAll('a[download]')];
    expect(links.map((link) => link.getAttribute('href'))).toEqual([`/arquivos/${base}03-dados.zip`, `/arquivos/${base}04-pagina.html`, `/arquivos/${base}05-logo.svg`]);
    expect(screen.getByText('2,0 KB')).toBeTruthy();
    expect(screen.getByText('1,5 KB')).toBeTruthy();
  });

  it('html e svg nunca viram imagem nem quadro', async () => {
    await open(attachments.slice(3));
    expect(document.querySelectorAll('img, iframe, object, embed')).toHaveLength(0);
    expect(document.querySelectorAll('a[download]')).toHaveLength(2);
  });

  it('pergunta sem anexo não tem a seção', async () => {
    await open([]);
    expect(screen.queryByRole('heading', { name: 'Anexos' })).toBeNull();
  });

  it('lista só o que a API trouxe', async () => {
    await open([attachments[2]]);
    expect(document.querySelectorAll('a[download]')).toHaveLength(1);
    expect(screen.queryByText('05-sumiu.png')).toBeNull();
  });
});
