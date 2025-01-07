import path from 'path';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';

export default defineConfig({
  plugins: [solidPlugin()],
  server: {
    port: 4200,
  },
  build: { target: 'esnext' },
  base: './',
  resolve: {
    alias: {
      '~': path.resolve(__dirname, './src'),
    },
  },
});
