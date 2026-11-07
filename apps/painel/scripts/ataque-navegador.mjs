// Ataca a entrega de arquivos do painel em Chromium real, na origem verdadeira do painel: HTML, SVG e
// .png disfarçado em evidências (CT-40, CT-42) e um documento final com script malicioso (CT-41). O mesmo
// ataque roda na página do painel como controle: sem ele ler o token e gravar um pedido, o teste não mede nada.
// Código de saída: 0 tudo passou; 3 alguma linha de CT-41 falhou, o que pede a decisão do usuário; 1 os demais casos.
// Uso: PAINEL_BIN=<vostok> [PAINEL_CHROMIUM=<chrome>] node apps/painel/scripts/ataque-navegador.mjs

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { createInterface } from 'node:readline';

const WAIT_MS = 10000;
const POLL_MS = 100;
const SETTLE_MS = 1000;
const TITLE_PREFIX = 'ATAQUE';
const EVIDENCE_DIR = 'vostok/work/tasks/SPEC-001/TASK-01/evidencias';
const DOCS_DIR = 'vostok/work/.archive/SPEC-000/docs';

// Cada tentativa tem o próprio try: uma barrada não pode esconder as outras.
const ATTACK = `(async () => {
  const outcome = { origem: self.origin, token_fetch: null, token_iframe: null, token_janela: null, status_post: null, armazenamento: false };
  const tokenIn = (page) => page?.querySelector('meta[name="painel-token"]')?.content ?? null;
  const pause = (ms) => new Promise((done) => setTimeout(done, ms));
  try {
    const page = await (await fetch('/')).text();
    outcome.token_fetch = page.match(/name="painel-token" content="([^"]+)"/)?.[1] ?? null;
  } catch (error) {
    outcome.erro_fetch = String(error);
  }
  try {
    const frame = document.createElement('iframe');
    frame.src = '/';
    const loaded = new Promise((done) => {
      frame.addEventListener('load', done);
      setTimeout(done, 2000);
    });
    document.documentElement.appendChild(frame);
    await loaded;
    outcome.token_iframe = tokenIn(frame.contentDocument);
    frame.remove();
  } catch (error) {
    outcome.erro_iframe = String(error);
  }
  try {
    const opened = window.open('/');
    outcome.janela_aberta = Boolean(opened);
    if (opened) {
      await pause(1000);
      outcome.token_janela = tokenIn(opened.document);
      opened.close();
    }
  } catch (error) {
    outcome.erro_janela = String(error);
  }
  try {
    const token = outcome.token_fetch ?? outcome.token_iframe ?? outcome.token_janela ?? '';
    const answer = await fetch('/api/pedidos?acao=criar-spec', {
      method: 'POST',
      headers: { 'X-Painel-Token': token },
      body: 'pedido do ataque',
    });
    outcome.status_post = answer.status;
  } catch (error) {
    outcome.erro_post = String(error);
  }
  try {
    localStorage.setItem('ataque', '1');
    outcome.armazenamento = true;
  } catch (error) {
    outcome.erro_armazenamento = String(error);
  }
  document.title = '${TITLE_PREFIX}:' + JSON.stringify(outcome);
  return outcome;
})()`;

const htmlAttack = `<!doctype html><html><head><title>isca</title></head><body><script>${ATTACK}</script></body></html>`;
const svgAttack = `<svg xmlns="http://www.w3.org/2000/svg"><script><![CDATA[${ATTACK}]]></script></svg>`;

const workDir = mkdtempSync(join(tmpdir(), 'ataque-navegador-'));
const projectDir = join(workDir, 'projeto');
const downloadDir = join(workDir, 'baixados');
const children = [];
const failedGroups = new Set();

function fail(message) {
  console.error(message);
  process.exit(1);
}

function cleanup() {
  for (const child of children) {
    try {
      process.kill(-child.pid, 'SIGKILL');
    } catch {}
  }
  rmSync(workDir, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}

process.on('exit', cleanup);
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => process.exit(130));

const sleep = (ms) => new Promise((done) => setTimeout(done, ms));

function freePort() {
  return new Promise((done, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => done(port));
    });
  });
}

function start(command, args, stdio) {
  const child = spawn(command, args, { stdio, detached: true });
  child.on('error', (error) => fail(`falha ao iniciar ${command}: ${error.message}`));
  children.push(child);
  return child;
}

function findChromium() {
  if (process.env.PAINEL_CHROMIUM) return process.env.PAINEL_CHROMIUM;
  const base = join(homedir(), '.cache/ms-playwright');
  if (!existsSync(base)) return '';
  const candidates = readdirSync(base)
    .filter((name) => name.startsWith('chromium-'))
    .sort()
    .flatMap((name) => ['chrome-linux', 'chrome-linux64'].map((dir) => join(base, name, dir, 'chrome')))
    .filter((path) => existsSync(path));
  return candidates.at(-1) ?? '';
}

function connect(url, onEvent = () => {}) {
  return new Promise((done, reject) => {
    const socket = new WebSocket(url);
    let nextId = 0;
    const pending = new Map();
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      if (!message.id) return onEvent(message);
      const call = pending.get(message.id);
      if (!call) return undefined;
      pending.delete(message.id);
      if (message.error) call.reject(new Error(message.error.message));
      else call.resolve(message.result);
      return undefined;
    };
    socket.onerror = () => reject(new Error('falha ao conectar no DevTools'));
    socket.onopen = () =>
      done((method, params = {}) => new Promise((resolveCall, rejectCall) => {
        const id = ++nextId;
        pending.set(id, { resolve: resolveCall, reject: rejectCall });
        socket.send(JSON.stringify({ id, method, params }));
      }));
  });
}

async function devtoolsJson(debugPort, path, ready) {
  for (let tries = 0; tries < 100; tries++) {
    try {
      const value = await (await fetch(`http://127.0.0.1:${debugPort}${path}`)).json();
      const found = ready(value);
      if (found) return found;
    } catch {}
    await sleep(200);
  }
  return fail(`o Chromium não respondeu em ${path}`);
}

async function waitUntil(condition) {
  const deadline = Date.now() + WAIT_MS;
  while (Date.now() < deadline) {
    const value = await condition();
    if (value) return value;
    await sleep(POLL_MS);
  }
  return null;
}

function writeProjectFile(relative, content) {
  const path = join(projectDir, relative);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

function runInProject(command, args) {
  const run = spawnSync(command, args, { cwd: projectDir, stdio: ['ignore', 'pipe', 'pipe'] });
  if (run.status !== 0) fail(`falha em ${command} ${args.join(' ')}: ${run.stderr ?? run.error}`);
}

function withBeforeBody(html, extra) {
  return html.replace('</body>', () => `${extra}</body>`);
}

function prepareProject(bin) {
  mkdirSync(projectDir);
  runInProject('git', ['init', '-q']);
  runInProject(bin, ['init', '--yes']);
  const template = (name) => readFileSync(join(projectDir, 'vostok/templates/finalize', name), 'utf8');
  writeProjectFile(`${EVIDENCE_DIR}/ataque.html`, htmlAttack);
  writeProjectFile(`${EVIDENCE_DIR}/ataque.svg`, svgAttack);
  writeProjectFile(`${EVIDENCE_DIR}/falso.png`, htmlAttack);
  writeProjectFile(
    `${DOCS_DIR}/executivo.html`,
    withBeforeBody(template('executive.html'), `<a id="anexo" href="assets/anexo.zip">anexo</a><script>${ATTACK}</script>`),
  );
  writeProjectFile(`${DOCS_DIR}/tecnico.html`, template('technical.html'));
  writeProjectFile(`${DOCS_DIR}/assets/anexo.zip`, 'PK');
}

function startPainel(bin) {
  const painel = start(bin, ['painel', '--sem-navegador', '--porta', '0', '--pasta', projectDir], ['ignore', 'pipe', 'ignore']);
  return new Promise((done, reject) => {
    const lines = createInterface({ input: painel.stdout });
    lines.once('line', (line) => {
      // O painel segue escrevendo na saída; um pipe sem leitor o derrubaria.
      painel.stdout.resume();
      const url = line.trim().split(/\s+/).at(-1);
      if (url?.startsWith('http://')) done(url.replace(/\/$/, ''));
      else reject(new Error(`sem URL na saída do painel: ${line}`));
    });
    painel.once('exit', (code) => reject(new Error(`o painel terminou com o código ${code}`)));
  });
}

function countRequests(dir = join(projectDir, 'vostok/work/pedidos')) {
  if (!existsSync(dir)) return 0;
  return readdirSync(dir, { withFileTypes: true }).reduce((total, entry) => {
    if (entry.name.startsWith('PED-')) return total + 1;
    return entry.isDirectory() ? total + countRequests(join(dir, entry.name)) : total;
  }, 0);
}

function report(group, passed, detail) {
  if (!passed) failedGroups.add(group);
  console.log(`${passed ? 'PASSOU' : 'FALHOU'} ${group}: ${detail}`);
}

async function attempt(action) {
  try {
    return await action();
  } catch (error) {
    return `erro: ${error.message}`;
  }
}

async function main() {
  if (!process.env.PAINEL_BIN) fail('uso: PAINEL_BIN=<vostok> [PAINEL_CHROMIUM=<chrome>] node apps/painel/scripts/ataque-navegador.mjs');
  const bin = process.env.PAINEL_BIN.includes('/') ? resolve(process.env.PAINEL_BIN) : process.env.PAINEL_BIN;
  const chromium = findChromium();
  if (!chromium) fail('Chromium do Playwright não encontrado (defina PAINEL_CHROMIUM)');

  prepareProject(bin);
  const origin = await startPainel(bin);

  const debugPort = await freePort();
  start(chromium, [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--window-size=1280,900',
    `--remote-debugging-port=${debugPort}`, `--user-data-dir=${join(workDir, 'chromium')}`, 'about:blank',
  ], 'ignore');

  const downloads = [];
  const browserUrl = await devtoolsJson(debugPort, '/json/version', (info) => info.webSocketDebuggerUrl);
  const browser = await connect(browserUrl, (message) => {
    if (message.method === 'Browser.downloadWillBegin') downloads.push(message.params.suggestedFilename);
  });
  console.log(`Chromium: ${(await browser('Browser.getVersion')).product}`);
  mkdirSync(downloadDir);
  await browser('Browser.setDownloadBehavior', { behavior: 'allow', downloadPath: downloadDir, eventsEnabled: true });

  const pageUrl = await devtoolsJson(debugPort, '/json/list', (targets) => targets.find((target) => target.type === 'page')?.webSocketDebuggerUrl);
  const send = await connect(pageUrl);
  await send('Page.enable');

  const read = async (expression) => (await send('Runtime.evaluate', { expression, returnByValue: true })).result.value;
  const navigate = (path) => send('Page.navigate', { url: `${origin}${path}` });
  const waitForLoad = () => waitUntil(() => read(`document.readyState === 'complete'`));
  const leavePage = async () => {
    await send('Page.navigate', { url: 'about:blank' });
    await waitForLoad();
  };
  const attackTitle = async () => String((await read('document.title')) ?? '');
  const waitForDownload = (name, seen) => waitUntil(() => downloads.slice(seen).includes(name));
  const click = async (selector) => {
    const box = await read(`(() => {
      const element = document.querySelector(${JSON.stringify(selector)});
      element.scrollIntoView({ block: 'center' });
      const { x, y, width, height } = element.getBoundingClientRect();
      return { x: x + width / 2, y: y + height / 2 };
    })()`);
    for (const type of ['mousePressed', 'mouseReleased']) {
      await send('Input.dispatchMouseEvent', { type, x: box.x, y: box.y, button: 'left', clickCount: 1 });
    }
  };

  for (const name of ['ataque.html', 'ataque.svg']) {
    const before = countRequests();
    const seen = downloads.length;
    const { errorText } = await navigate(`/arquivos/${EVIDENCE_DIR}/${name}`);
    const downloaded = Boolean(await waitForDownload(name, seen));
    await sleep(SETTLE_MS);
    const title = await attackTitle();
    const created = countRequests() - before;
    report(
      'CT-40',
      errorText === 'net::ERR_ABORTED' && downloaded && !title.startsWith(TITLE_PREFIX) && created === 0,
      `${name} baixou=${downloaded} erro=${errorText} titulo=${JSON.stringify(title)} pedidos novos=${created}`,
    );
    await leavePage();
  }

  {
    const before = countRequests();
    await navigate(`/arquivos/${EVIDENCE_DIR}/falso.png`);
    await waitForLoad();
    await sleep(SETTLE_MS);
    const contentType = await read('document.contentType');
    const title = await attackTitle();
    const created = countRequests() - before;
    report(
      'CT-42',
      contentType === 'image/png' && !title.startsWith(TITLE_PREFIX) && created === 0,
      `falso.png contentType=${contentType} titulo=${JSON.stringify(title)} pedidos novos=${created}`,
    );
    await leavePage();
  }

  {
    const before = countRequests();
    await navigate(`/arquivos/${DOCS_DIR}/executivo.html`);
    const title = (await waitUntil(async () => {
      const current = await attackTitle();
      return current.startsWith(TITLE_PREFIX) ? current : null;
    })) ?? (await attackTitle());
    const ran = title.startsWith(`${TITLE_PREFIX}:`);
    report('CT-41', ran, `script rodou=${ran}`);
    const outcome = ran ? JSON.parse(title.slice(TITLE_PREFIX.length + 1)) : {};
    await sleep(SETTLE_MS);
    const created = countRequests() - before;
    report(
      'CT-41',
      ran && outcome.token_fetch === null && outcome.token_iframe === null && outcome.token_janela === null && created === 0,
      `executivo.html token_fetch=${outcome.token_fetch} token_iframe=${outcome.token_iframe} token_janela=${outcome.token_janela} status_post=${outcome.status_post} pedidos novos=${created}`,
    );
    const documentOrigin = await read('self.origin');
    report('CT-41', documentOrigin === 'null', `executivo.html origem=${documentOrigin}`);

    const toggleLabel = `document.querySelector('#theme-toggle').textContent`;
    const labels = await attempt(async () => {
      const initial = await read(toggleLabel);
      await click('#theme-toggle');
      return [initial, await read(toggleLabel)];
    });
    report('CT-41', Array.isArray(labels) && labels[0] !== labels[1], `tema trocou: ${JSON.stringify(labels)}`);

    const seen = downloads.length;
    const attachment = await attempt(async () => {
      await click('#anexo');
      return Boolean(await waitForDownload('anexo.zip', seen));
    });
    report('CT-41', attachment === true, `anexo.zip baixou=${attachment}`);

    const reached = await attempt(async () => {
      await click('a[href="tecnico.html"]');
      return waitUntil(async () => {
        const path = await read('location.pathname');
        return path.endsWith('/docs/tecnico.html') && (await read(`document.readyState === 'complete'`)) ? path : null;
      });
    });
    const linkedOrigin = await read('self.origin');
    report(
      'CT-41',
      typeof reached === 'string' && reached.endsWith('/docs/tecnico.html') && linkedOrigin === 'null',
      `link tecnico.html caminho=${reached} origem=${linkedOrigin}`,
    );
    await leavePage();
  }

  {
    const before = countRequests();
    await navigate('/');
    await waitForLoad();
    const evaluated = await send('Runtime.evaluate', { expression: ATTACK, awaitPromise: true, returnByValue: true });
    const outcome = evaluated.result.value ?? {};
    const created = countRequests() - before;
    report(
      'CONTROLE',
      Boolean(outcome.token_fetch) && Boolean(outcome.token_iframe) && outcome.status_post === 200 && created === 1,
      `pagina do painel token_lido=${Boolean(outcome.token_fetch)} iframe=${Boolean(outcome.token_iframe)} status_post=${outcome.status_post} origem=${outcome.origem} pedidos novos=${created}`,
    );
  }

  if (failedGroups.has('CT-41')) {
    console.log('CT-41 FALHOU: pare e peça a decisão do usuário');
    process.exit(3);
  }
  process.exit(failedGroups.size === 0 ? 0 : 1);
}

main().catch((error) => fail(error.stack ?? String(error)));
