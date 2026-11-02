import preact from '@preact/preset-vite';
import { defineConfig } from 'vitest/config';

const api = process.env.PAINEL_API;

const proxy = api
  ? {
      '/api': { target: api, ws: true },
      '/arquivos': { target: api },
    }
  : undefined;

export default defineConfig({
  plugins: [preact()],
  server: { proxy },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    modulePreload: { polyfill: false },
    assetsInlineLimit: 0,
  },
  test: { environment: 'jsdom', globals: false },
});
