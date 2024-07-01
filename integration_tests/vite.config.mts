import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    hookTimeout: 1_500_000,
    testTimeout: 1_500_000,
    watchExclude: ['**/node_modules/**', '**/*.yml', '**/.__cosmopark'],
    maxThreads: process.env.MAX_THREADS ? parseInt(process.env.MAX_THREADS) : 2,
    minThreads: 2,
  },
});
