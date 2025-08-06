import { Wallet } from '@initia/initia.js';

export type MoveLiquidityProviderConfig = {
  wallet: Wallet;
  moduleAddress: string;
  moduleObject: string;
  minAmountToProvide: bigint;
};
