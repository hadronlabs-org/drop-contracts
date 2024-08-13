import { Uint64 } from '@cosmjs/math';

export type StakerConfig = {
  contractAddress: string;
  coreContractAddress: string;
  stakerMinBalance: Uint64;
};
