import { SigningStargateClient, StargateClient } from '@cosmjs/stargate';
import { waitFor } from './waitFor';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export const waitForTx = async (
  client: SigningStargateClient | SigningCosmWasmClient | StargateClient,
  hash: string,
  timeout: number = 10000,
  interval: number = 600,
): Promise<void> =>
  await waitFor(
    async () => {
      const tx = await client.getTx(hash);
      if (tx === null) {
        return false;
      }
      if (tx.code !== 0) {
        throw new Error(`Transaction failed with code: ${tx.code}`);
      }
      return tx.code === 0 && tx.height > 0;
    },
    timeout,
    interval,
  );
