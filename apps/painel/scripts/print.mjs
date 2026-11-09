// Tira o print de uma URL do painel com o Playwright: aceita âncora, rolagem e localStorage.
// Uso: node scripts/print.mjs <url> <arquivo.png> <largura,altura>
// PAINEL_ROLAR: pixels a rolar antes do print, ou "fim" para o fim da página.
// PAINEL_STORAGE: JSON com as chaves do localStorage gravadas antes de abrir a página.
import { chromium } from 'playwright';

const [url, output, size = '1280,900'] = process.argv.slice(2);
if (!url || !output) {
  console.error('uso: print.mjs <url> <arquivo.png> [largura,altura]');
  process.exit(2);
}
const [width, height] = size.split(',').map(Number);

const browser = await chromium.launch();
try {
  const context = await browser.newContext({ viewport: { width, height } });
  const storage = process.env.PAINEL_STORAGE;
  if (storage) {
    await context.addInitScript((entries) => {
      for (const [key, value] of Object.entries(JSON.parse(entries))) {
        localStorage.setItem(key, typeof value === 'string' ? value : JSON.stringify(value));
      }
    }, storage);
  }
  const page = await context.newPage();
  await page.goto(url, { waitUntil: 'networkidle' });
  const hash = new URL(url).hash.slice(1);
  if (hash) {
    await page.locator(`[id="${hash}"]`).first().waitFor({ state: 'attached' }).catch(() => {});
  }
  const scroll = process.env.PAINEL_ROLAR;
  if (scroll) {
    await page.evaluate((amount) => {
      const top = amount === 'fim' ? document.documentElement.scrollHeight : Number(amount);
      window.scrollTo(0, top);
      window.dispatchEvent(new Event('scroll'));
    }, scroll);
  }
  await page.waitForTimeout(500);
  await page.screenshot({ path: output });
} finally {
  await browser.close();
}
