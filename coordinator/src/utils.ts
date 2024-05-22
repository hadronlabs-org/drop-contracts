import { execSync } from 'child_process';
import { Context } from './types/Context';
import pino from 'pino';

export function runQueryRelayer(
  context: Context,
  log: pino.Logger,
  queryIds: string[],
) {
  try {
    const stdout = execSync(
      `${context.config.coordinator.icqRunCmd} -q ${queryIds.join(' -q ')}`,
    );
    log.debug(`stdout: ${stdout}`);
  } catch (error) {
    log.error(`Error running query relayer: ${error.message}`);
  }
}

export async function waitBlocks(
  context: Context,
  blocks: number,
  log: pino.Logger,
): Promise<void> {
  const initBlock = (await context.neutronTmClient.block()).block.header.height;
  return waitFor(async () => {
    try {
      const currentBlock = (await context.neutronTmClient.block()).block.header
        .height;
      if (currentBlock - initBlock >= blocks) {
        return true;
      }
    } catch (e) {
      log.error('Unable to reach required block ', e);
      return false;
    }
  }, 200_000);
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitFor(
  fn: () => Promise<boolean>,
  timeout: number = 10000,
  interval: number = 600,
): Promise<void> {
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
}
