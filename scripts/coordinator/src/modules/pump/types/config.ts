import { Decimal } from '@cosmjs/math';

export type PumpConfig = {
  contractAddress: string;
  minBalance: Decimal;
  icaFeeBuffer: Decimal;
};
