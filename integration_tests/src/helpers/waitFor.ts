import { sleep } from './sleep';

export const waitFor = async (
  fn: () => Promise<boolean>,
  timeout: number = 10000,
  interval: number = 600,
): Promise<void> => {
  const start = Date.now();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    if (await fn()) {
      break;
    }
    if (Date.now() - start > timeout) {
      throw new Error('Timeout waiting for condition');
    }
    await sleep(interval);
  }
};
