// Tira o print de uma tela do painel depois de uma ação feita com a página aberta: sobe a API de
// PAINEL_BIN e o vite do frontend, abre o Chromium com o protocolo DevTools, roda o comando na pasta
// do projeto, espera o texto aparecer na tela e grava o print.
// Uso: [PAINEL_CHROMIUM=<chrome>] PAINEL_BIN=<caminho do vostok> node apps/painel/scripts/print-aviso.mjs <pasta> <saida.png> <largura,altura> <rota> <texto esperado> <comando>

import { spawn, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { createServer } from 'node:net';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const USAGE = 'uso: print-aviso.mjs <pasta> <saida.png> <largura,altura> <rota> <texto esperado> <comando>';
const TEXT_TIMEOUT_MS = 5000;
const POLL_MS = 100;
const SETTLE_MS = 500;

const [folder, output, size, route, expectedText, command] = process.argv.slice(2);
if (!command) fail(USAGE);

const bin = process.env.PAINEL_BIN ?? 'vostok';
const painelDir = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const rootDir = resolve(painelDir, '../..');
const workDir = mkdtempSync(join(tmpdir(), 'print-aviso-'));
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

async function main() {
  const chromium = findChromium();
  if (!chromium) fail('Chromium do Playwright não encontrado (defina PAINEL_CHROMIUM)');
  const [width, height] = size.split(',').map(Number);

  const apiPort = await freePort();
  start(bin, ['painel', '--sem-navegador', '--porta', String(apiPort), '--pasta', resolve(folder)], { cwd: rootDir });
  await waitFor(`http://127.0.0.1:${apiPort}/api/marca`, 'a API');

  const webPort = await freePort();
  start('pnpm', ['exec', 'vite', '--port', String(webPort), '--strictPort'], {
    cwd: painelDir,
    env: { ...process.env, PAINEL_API: `http://127.0.0.1:${apiPort}` },
  });
  await waitFor(`http://127.0.0.1:${webPort}/`, 'o vite');

  const debugPort = await freePort();
  start(chromium, [
    '--headless=new', '--no-sandbox', '--disable-gpu', '--hide-scrollbars', '--disable-partial-raster',
    `--window-size=${width},${height}`, `--remote-debugging-port=${debugPort}`,
    `--user-data-dir=${join(workDir, 'chromium')}`, 'about:blank',
  ]);
  const send = await connect(await pageSocketUrl(debugPort));
  const read = async (expression) =>
    (await send('Runtime.evaluate', { expression, returnByValue: true })).result.value;

  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false });
  await send('Page.navigate', { url: `http://127.0.0.1:${webPort}${route}` });
  await waitForStablePage(read);

  const run = spawnSync('sh', ['-c', command], { cwd: resolve(folder), stdio: ['ignore', 'ignore', 'inherit'] });
  if (run.status !== 0) fail(`o comando terminou com código ${run.status}`);

  await waitForText(read);
  await waitForAnimations(read);

  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  mkdirSync(dirname(resolve(output)), { recursive: true });
  writeFileSync(output, Buffer.from(data, 'base64'));
  console.log(output);
  process.exit(0);
}

// A primeira leitura da visão geral precisa terminar antes da ação: é a partir dela que o painel nota a volta.
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

// Sem isso, a entrada do aviso poderia ser pega em quadros diferentes a cada execução.
async function waitForAnimations(read) {
  const deadline = Date.now() + TEXT_TIMEOUT_MS;
  while (Date.now() < deadline) {
    if ((await read('document.getAnimations().filter((a) => a.playState === "running").length')) === 0) return;
    await sleep(POLL_MS);
  }
  fail('as animações da página não terminaram');
}

main().catch((error) => fail(error.message));
