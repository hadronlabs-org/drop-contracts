import { LCDClient, Wallet } from '@initia/initia.js';

export type MoveLiquidityProviderConfig = {
  lcd: LCDClient;
  wallet: Wallet;
  moduleAddress: string;
  moduleObject: string;
};
