import { Decimal } from '@cosmjs/math';

export type PumpConfig = {
  contractAddress: string;
  denomAllowlist: string[];
  minBalance: Decimal;
  icaFeeBuffer: Decimal;
};
