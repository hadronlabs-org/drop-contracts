import { Uint64 } from '@cosmjs/math';

export type PumpConfig = {
  contractAddress: string;
  minBalance: Uint64;
  icaFeeBuffer: Uint64;
};
