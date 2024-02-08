import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    hookTimeout: 500_000,
    testTimeout: 500_000,
    watchExclude: ['**/node_modules/**', '**/*.yml', '**/.__cosmopark'],
    maxThreads: process.env.MAX_THREADS ? parseInt(process.env.MAX_THREADS) : 2,
    minThreads: 2,
  },
});
