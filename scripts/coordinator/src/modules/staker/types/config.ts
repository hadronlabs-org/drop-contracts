import { Decimal } from '@cosmjs/math';

export type StakerConfig = {
  contractAddress: string;
  coreContractAddress: string;
  stakerMinBalance: Decimal;
  icaFeeBuffer: Decimal;
};
