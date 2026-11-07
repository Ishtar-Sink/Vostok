// Tira o print de uma tela do painel depois de cliques, digitação e rolagem: sobe a API de PAINEL_BIN e
// o vite do frontend, abre o Chromium com o protocolo DevTools, aplica as ações na ordem da linha de
// comando, espera o texto aparecer na tela e grava o print. Sem o texto, ou com uma ação que falha,
// termina com erro e não grava nada.
// Uso: [PAINEL_CHROMIUM=<chrome>] PAINEL_BIN=<caminho do vostok> node apps/painel/scripts/print-clique.mjs <pasta> <saida.png> <largura,altura> <rota> <texto esperado> [--clicar <texto ou seletor>]... [--digitar <seletor> <texto>]... [--rolar <px>]... [--avaliar <js>]... [--tecla <Tab|Enter|Espaco|Escape>]... [--comando <sh>]...

import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { createServer } from 'node:net';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const USAGE = 'uso: PAINEL_BIN=<vostok> print-clique.mjs <pasta> <saida.png> <largura,altura> <rota> <texto esperado> [--clicar <texto ou seletor>]... [--digitar <seletor> <texto>]... [--rolar <px>]... [--avaliar <js>]... [--tecla <Tab|Enter|Espaco|Escape>]... [--comando <sh>]...';
const TEXT_TIMEOUT_MS = 5000;
const POLL_MS = 100;
const SETTLE_MS = 500;
const CLICKABLE = 'button, a, summary, label, [role=button], [role=tab], [role=menuitem], [role=option], [role=radio], [role=checkbox]';
const FLAGS = {
  '--clicar': { kind: 'click', values: 1 },
  '--digitar': { kind: 'type', values: 2 },
  '--rolar': { kind: 'scroll', values: 1 },
  '--avaliar': { kind: 'check', values: 1 },
  '--tecla': { kind: 'key', values: 1 },
  '--comando': { kind: 'command', values: 1 },
};
// Nome aceito em --tecla: valores do evento de teclado do protocolo DevTools.
const KEYS = {
  tab: { key: 'Tab', code: 'Tab', keyCode: 9 },
  enter: { key: 'Enter', code: 'Enter', keyCode: 13, text: '\r' },
  espaco: { key: ' ', code: 'Space', keyCode: 32, text: ' ' },
  space: { key: ' ', code: 'Space', keyCode: 32, text: ' ' },
  escape: { key: 'Escape', code: 'Escape', keyCode: 27 },
  esc: { key: 'Escape', code: 'Escape', keyCode: 27 },
};

const [folder, output, size, route, expectedText, ...rest] = process.argv.slice(2);
if (!expectedText) fail(USAGE);
if (!process.env.PAINEL_BIN) fail(USAGE);

const actions = [];
for (let index = 0; index < rest.length; ) {
  const flag = FLAGS[rest[index]];
  if (!flag) fail(USAGE);
  const values = rest.slice(index + 1, index + 1 + flag.values);
  if (values.length !== flag.values) fail(USAGE);
  if (flag.kind === 'scroll' && !Number.isInteger(Number(values[0]))) fail(USAGE);
  if (flag.kind === 'key' && !KEYS[values[0].toLowerCase()]) fail(`tecla desconhecida: ${values[0]} (use Tab, Enter, Espaco ou Escape)\n${USAGE}`);
  actions.push({ kind: flag.kind, values });
  index += 1 + flag.values;
}

// Um caminho relativo mudaria de sentido no cwd da API.
const bin = process.env.PAINEL_BIN.includes('/') ? resolve(process.env.PAINEL_BIN) : process.env.PAINEL_BIN;
const painelDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const rootDir = resolve(painelDir, '../..');
const workDir = mkdtempSync(join(tmpdir(), 'print-clique-'));
const children = [];

function fail(message) {
  console.error(message);
  process.exit(1);
}

function cleanup() {
  // Cada filho lidera o próprio grupo: o `pnpm exec` deixaria o vite órfão se só ele fosse encerrado.
  for (const child of children) {
    try {
      process.kill(-child.pid, 'SIGKILL');
    } catch {}
  }
  rmSync(workDir, { recursive: true, force: true });
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

function start(cmd, args, options) {
  const child = spawn(cmd, args, { stdio: 'ignore', detached: true, ...options });
  child.on('error', (error) => fail(`falha ao iniciar ${cmd}: ${error.message}`));
  children.push(child);
  return child;
}

async function waitFor(url, label) {
  for (let tries = 0; tries < 150; tries++) {
    try {
      if ((await fetch(url)).ok) return;
    } catch {}
    await sleep(200);
  }
  fail(`sem resposta de ${label} (${url})`);
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

function connect(url) {
  return new Promise((done, reject) => {
    const socket = new WebSocket(url);
    let nextId = 0;
    const pending = new Map();
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      const call = pending.get(message.id);
      if (!call) return;
      pending.delete(message.id);
      if (message.error) call.reject(new Error(message.error.message));
      else call.resolve(message.result);
    };
    socket.onerror = () => reject(new Error('falha ao conectar no DevTools'));
    socket.onopen = () =>
      done((method, params = {}) => new Promise((resolve, rejectCall) => {
        const id = ++nextId;
        pending.set(id, { resolve, reject: rejectCall });
        socket.send(JSON.stringify({ id, method, params }));
      }));
  });
}

async function pageSocketUrl(debugPort) {
  for (let tries = 0; tries < 100; tries++) {
    try {
      const targets = await (await fetch(`http://127.0.0.1:${debugPort}/json/list`)).json();
      const page = targets.find((target) => target.type === 'page');
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
  }
  return fail('o Chromium não abriu uma página');
}

// O vite não conhece o token: ele só vem na página que a API serve, e sem ele toda ação recebe 403.
async function sessionToken(apiPort) {
  const page = await (await fetch(`http://127.0.0.1:${apiPort}/`)).text();
  const token = page.match(/name="painel-token" content="([^"]+)"/)?.[1];
  if (!token) fail('a API não entregou o token de sessão (o binário foi compilado com --features painel?)');
  return token;
}

async function main() {
  // Um print antigo com o mesmo nome passaria por resultado desta execução, se ela falhar.
  rmSync(resolve(output), { force: true });

  const chromium = findChromium();
  if (!chromium) fail('Chromium do Playwright não encontrado (defina PAINEL_CHROMIUM)');
  const [width, height] = size.split(',').map(Number);

  const apiPort = await freePort();
  start(bin, ['painel', '--sem-navegador', '--porta', String(apiPort), '--pasta', resolve(folder)], { cwd: rootDir });
  await waitFor(`http://127.0.0.1:${apiPort}/api/marca`, 'a API');
  const token = await sessionToken(apiPort);

  const webPort = await freePort();
  start('pnpm', ['exec', 'vite', '--port', String(webPort), '--strictPort'], {
    cwd: painelDir,
    env: { ...process.env, PAINEL_API: `http://127.0.0.1:${apiPort}` },
  });
  await waitFor(`http://127.0.0.1:${webPort}/`, 'o vite');

  const debugPort = await freePort();
  start(chromium, [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--hide-scrollbars', '--disable-partial-raster',
    '--disable-smooth-scrolling', `--window-size=${width},${height}`, `--remote-debugging-port=${debugPort}`,
    `--user-data-dir=${join(workDir, 'chromium')}`, 'about:blank',
  ]);
  const send = await connect(await pageSocketUrl(debugPort));
  const read = async (expression) =>
    (await send('Runtime.evaluate', { expression, returnByValue: true })).result.value;

  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
  await send('Page.addScriptToEvaluateOnNewDocument', {
    source: `document.addEventListener('DOMContentLoaded', () => {
      const meta = document.createElement('meta');
      meta.name = 'painel-token';
      meta.content = ${JSON.stringify(token)};
      document.head.append(meta);
    });`,
  });
  await send('Page.navigate', { url: `http://127.0.0.1:${webPort}${route}` });
  await waitForStablePage(read);

  for (const action of actions) {
    if (action.kind === 'click') await click(read, action.values[0]);
    if (action.kind === 'type') await type(read, action.values[0], action.values[1]);
    if (action.kind === 'scroll') await scroll(send, read, width, height, Number(action.values[0]));
    if (action.kind === 'check') await check(send, action.values[0]);
    if (action.kind === 'key') await press(send, action.values[0]);
    if (action.kind === 'command') runCommand(action.values[0]);
    await waitForStablePage(read);
  }

  await waitForText(read);
  await waitForAnimations(read);
  await waitForStablePage(read);
  // O cursor de um campo com foco pisca: o print sairia diferente a cada execução.
  await read(`document.head.append(Object.assign(document.createElement('style'), { textContent: '* { caret-color: transparent !important; }' }))`);

  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  mkdirSync(dirname(resolve(output)), { recursive: true });
  writeFileSync(output, Buffer.from(data, 'base64'));
  console.log(output);
  process.exit(0);
}

function runCommand(command) {
  const run = spawnSync('sh', ['-c', command], { cwd: resolve(folder), stdio: ['ignore', 'ignore', 'inherit'] });
  if (run.status !== 0) fail(`o comando terminou com código ${run.status}: ${command}`);
}

// Procura pelo texto e depois por seletor, dentro da janela aberta, se houver uma; o que está visível
// vem primeiro, e o escondido, como um botão da barra lateral no celular, fica por último. Usa
// `.click()` em vez de coordenadas justamente para alcançar esse botão.
function clickTarget(target) {
  return `(() => {
    const target = ${JSON.stringify(target)};
    const visible = (element) => {
      const box = element.getBoundingClientRect();
      return box.width > 0 && box.height > 0;
    };
    const scope = document.querySelector('dialog[open], [role=dialog]') ?? document;
    // O innerText de quem está com visibility: hidden vem vazio, daí o textContent.
    const label = (element) => (element.innerText.trim() || element.textContent.trim() || element.getAttribute('aria-label') || '').trim();
    const clickable = [...scope.querySelectorAll(${JSON.stringify(CLICKABLE)})];
    let selected = [];
    try {
      selected = [...document.querySelectorAll(target)];
    } catch {}
    const element =
      clickable.filter(visible).find((item) => label(item) === target) ??
      clickable.filter(visible).find((item) => label(item).includes(target)) ??
      selected.find(visible) ??
      clickable.find((item) => label(item) === target) ??
      clickable.find((item) => label(item).includes(target)) ??
      selected[0];
    if (!element) return false;
    element.click();
    return true;
  })()`;
}

async function click(read, target) {
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if (await read(clickTarget(target))) return;
    await sleep(POLL_MS);
  }
  fail(`nada para clicar com "${target}" em ${TEXT_TIMEOUT_MS / 1000} s`);
}

// O setter nativo do valor e o evento `input` fazem o estado do formulário mudar no Preact; atribuir
// `value` direto no elemento não dispararia nada.
async function type(read, selector, text) {
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    const typed = await read(`(() => {
      let field = null;
      try {
        field = document.querySelector(${JSON.stringify(selector)});
      } catch {}
      if (!field) return false;
      const proto = field instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
      field.focus();
      Object.getOwnPropertyDescriptor(proto, 'value').set.call(field, ${JSON.stringify(text)});
      field.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    if (typed) return;
    await sleep(POLL_MS);
  }
  fail(`nenhum campo para digitar com "${selector}" em ${TEXT_TIMEOUT_MS / 1000} s`);
}

// Tecla de verdade, pelo protocolo: o navegador move o foco e dispara o padrão do elemento focado.
async function press(send, name) {
  const { key, code, keyCode, text } = KEYS[name.toLowerCase()];
  const base = { key, code, windowsVirtualKeyCode: keyCode, nativeVirtualKeyCode: keyCode };
  await send('Input.dispatchKeyEvent', { type: text ? 'keyDown' : 'rawKeyDown', ...base, ...(text ? { text } : {}) });
  await send('Input.dispatchKeyEvent', { type: 'keyUp', ...base });
}

// Com `awaitPromise`, a expressão pode devolver uma promessa, e a rejeição chega como exceção.
async function check(send, expression) {
  const { result, exceptionDetails } = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (exceptionDetails) fail(`a expressão falhou: ${expression}: ${exceptionDetails.exception?.description ?? exceptionDetails.text}`);
  if (!result.value) fail(`a expressão deu falso: ${expression}`);
}

const SCROLL_TOTAL = `[document.scrollingElement, ...document.querySelectorAll('*')].reduce((total, element) => total + (element ? element.scrollTop : 0), 0)`;

// A roda no meio da tela rola quem estiver ali, como o usuário faria, e dispara o que observa a rolagem.
// Tenta de novo porque os dados podem chegar depois: até lá, a tela pode não ter o que rolar.
async function scroll(send, read, width, height, deltaY) {
  const before = await read(SCROLL_TOTAL);
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    await send('Input.dispatchMouseEvent', { type: 'mouseWheel', x: width / 2, y: height / 2, deltaX: 0, deltaY });
    await sleep(3 * POLL_MS);
    if ((await read(SCROLL_TOTAL)) !== before) return;
  }
  fail(`a rolagem de ${deltaY} px não moveu a tela em ${TEXT_TIMEOUT_MS / 1000} s`);
}

// A primeira leitura da tela precisa terminar antes da ação, e cada ação precisa assentar antes da próxima.
async function waitForStablePage(read) {
  let last = '';
  let stableSince = Date.now();
  const deadline = Date.now() + 15000;
  while (Date.now() < deadline) {
    const text = (await read('document.readyState === "complete" ? document.body.innerText : ""')) ?? '';
    if (text !== last) {
      last = text;
      stableSince = Date.now();
    } else if (text && Date.now() - stableSince >= SETTLE_MS) return;
    await sleep(POLL_MS);
  }
  fail('a página não terminou de carregar');
}

async function waitForText(read) {
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  const needle = JSON.stringify(expectedText);
  while (Date.now() < deadline) {
    if (await read(`document.body.innerText.includes(${needle})`)) return;
    await sleep(POLL_MS);
  }
  fail(`o texto "${expectedText}" não apareceu em ${TEXT_TIMEOUT_MS / 1000} s`);
}

// Sem isso, a entrada de uma janela ou de um aviso poderia ser pega em quadros diferentes a cada execução.
async function waitForAnimations(read) {
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if ((await read('document.getAnimations().filter((a) => a.playState === "running").length')) === 0) return;
    await sleep(POLL_MS);
  }
  fail('as animações da página não terminaram');
}

main().catch((error) => fail(error.message));
