// Grava em .webm, com o recordVideo do Playwright, as ações de PAINEL_ACOES numa URL do painel.
// Uso: node scripts/video.mjs <url> <arquivo.webm> <largura,altura>
// PAINEL_ACOES: ações separadas por ";;", executadas em ordem com uma pausa curta entre elas:
//   clicar <seletor> | arrastar <seletor> <dx> <dy> | tecla <tecla> | esperar <ms>
// Ação desconhecida ou seletor ausente falha com código 1. O vídeo não desenha o cursor; um ponto injetado o mostra.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium } from 'playwright';

const PAUSE_MS = 500;
const DRAG_STEPS = 20;

const [url, output, size = '1280,720'] = process.argv.slice(2);
if (!url || !output) {
  console.error('uso: video.mjs <url> <arquivo.webm> [largura,altura]');
  process.exit(2);
}
const [width, height] = size.split(',').map(Number);
const actions = (process.env.PAINEL_ACOES ?? '').split(';;').map((action) => action.trim()).filter(Boolean);

function showCursor() {
  const dot = document.createElement('div');
  dot.style.cssText = 'position:fixed;z-index:2147483647;width:18px;height:18px;margin:-9px 0 0 -9px;border-radius:50%;background:transparent;border:2px solid #7c3aed;box-shadow:0 0 0 2px #fff;pointer-events:none;left:-40px;top:-40px;transition:transform 80ms';
  document.addEventListener('DOMContentLoaded', () => document.body.appendChild(dot));
  document.addEventListener('mousemove', (event) => {
    dot.style.left = `${event.clientX}px`;
    dot.style.top = `${event.clientY}px`;
  }, true);
  document.addEventListener('mousedown', () => { dot.style.transform = 'scale(0.7)'; }, true);
  document.addEventListener('mouseup', () => { dot.style.transform = 'scale(1)'; }, true);
}

async function locate(page, selector) {
  const target = page.locator(selector).first();
  if ((await target.count()) === 0) {
    console.error(`seletor não encontrado: ${selector}`);
    process.exit(1);
  }
  return target;
}

async function drag(page, selector, dx, dy) {
  const target = await locate(page, selector);
  await target.scrollIntoViewIfNeeded();
  const box = await target.boundingBox();
  const x = box.x + box.width / 2;
  const y = box.y + box.height / 2;
  await page.mouse.move(x, y, { steps: 10 });
  await page.mouse.down();
  await page.mouse.move(x + dx, y + dy, { steps: DRAG_STEPS });
  await page.mouse.up();
}

async function run(page, action) {
  const [command, ...words] = action.split(/\s+/);
  const rest = words.join(' ');
  if (command === 'clicar') {
    await (await locate(page, rest)).click();
  } else if (command === 'arrastar') {
    const dy = Number(words.pop());
    const dx = Number(words.pop());
    await drag(page, words.join(' '), dx, dy);
  } else if (command === 'tecla') {
    await page.keyboard.press(rest);
  } else if (command === 'esperar') {
    await page.waitForTimeout(Number(rest));
  } else {
    console.error(`ação inválida: ${action}`);
    process.exit(1);
  }
}

const folder = mkdtempSync(join(tmpdir(), 'painel-video-'));
const browser = await chromium.launch();
try {
  const context = await browser.newContext({
    viewport: { width, height },
    recordVideo: { dir: folder, size: { width, height } },
    permissions: ['clipboard-read', 'clipboard-write'],
  });
  await context.addInitScript(showCursor);
  const page = await context.newPage();
  await page.goto(url, { waitUntil: 'networkidle' });
  const hash = new URL(url).hash.slice(1);
  if (hash) await page.locator(`[id="${hash}"]`).first().waitFor({ state: 'attached' }).catch(() => {});
  await page.waitForTimeout(PAUSE_MS * 2);
  for (const action of actions) {
    await run(page, action);
    await page.waitForTimeout(PAUSE_MS);
  }
  const video = page.video();
  await context.close();
  await video.saveAs(output);
} finally {
  await browser.close();
  rmSync(folder, { recursive: true, force: true });
}
