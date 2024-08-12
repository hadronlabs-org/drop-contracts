import { Config } from '../config';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { CometClient } from '@cosmjs/tendermint-rpc';
import { BankExtension, QueryClient, StakingExtension } from '@cosmjs/stargate';
import { FactoryContractHandler } from '../factoryContract';

export type Context = {
  config: Config;
  height: number;
  factoryContractHandler: FactoryContractHandler;
  neutronWallet: DirectSecp256k1HdWallet;
  neutronWalletAddress: string;
  targetWallet: DirectSecp256k1HdWallet;
  targetWalletAddress: string;
  neutronCometClient: CometClient;
  neutronQueryClient: QueryClient & BankExtension;
  neutronClient: InstanceType<typeof NeutronClient>;
  neutronSigningClient: SigningCosmWasmClient;
  targetSigningClient: SigningCosmWasmClient;
  targetCometClient: CometClient;
  targetQueryClient: QueryClient & StakingExtension & BankExtension;
};
