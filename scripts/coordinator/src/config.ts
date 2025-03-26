import pino from 'pino';
import { GasPrice } from '@cosmjs/stargate';

export class Config {
  coordinator: {
    mnemonic: string;
    factoryContractAddress: string;
    icqRunCmd: string;
    checksPeriod: number;
  };
  neutron: {
    rpc: string;
    rest: string;
    gasPrice: GasPrice;
    gasAdjustment: string;
    icaFee: string;
  };
  target: {
    rpc: string;
    rest: string;
    denom: string;
    gasPrice: GasPrice;
    gasAdjustment: string;
    accountPrefix: string;
    validatorAccountPrefix: string;
    chainId: string;
  };

  constructor(private logContext: pino.Logger) {
    this.load();
  }

  load() {
    this.coordinator = {
      mnemonic: process.env.COORDINATOR_MNEMONIC,
      factoryContractAddress: process.env.FACTORY_CONTRACT_ADDRESS,
      icqRunCmd: process.env.ICQ_RUN_COMMAND,
      checksPeriod: process.env.COORDINATOR_CHECKS_PERIOD
        ? parseInt(process.env.COORDINATOR_CHECKS_PERIOD, 10)
        : 10,
    };

    this.neutron = {
      rpc: process.env.RELAYER_NEUTRON_CHAIN_RPC_ADDR,
      rest: process.env.RELAYER_NEUTRON_CHAIN_REST_ADDR,
      gasPrice: GasPrice.fromString(
        process.env.RELAYER_NEUTRON_CHAIN_GAS_PRICES,
      ),
      gasAdjustment: process.env.NEUTRON_GAS_ADJUSTMENT,
      icaFee: process.env.ICA_FEE_AMOUNT,
    };

    this.target = {
      rpc: process.env.RELAYER_TARGET_CHAIN_RPC_ADDR,
      rest: process.env.RELAYER_TARGET_CHAIN_REST_ADDR,
      denom: process.env.RELAYER_TARGET_CHAIN_DENOM,
      gasPrice: GasPrice.fromString(
        process.env.RELAYER_TARGET_CHAIN_GAS_PRICES,
      ),
      gasAdjustment: process.env.RELAYER_TARGET_CHAIN_GAS_ADJUSTMENT,
      accountPrefix: process.env.RELAYER_TARGET_CHAIN_ACCOUNT_PREFIX,
      validatorAccountPrefix:
      process.env.RELAYER_TARGET_CHAIN_VALIDATOR_ACCOUNT_PREFIX,
      chainId: process.env.RELAYER_TARGET_CHAIN_ID,
    };

    this.logContext.info('Config loaded');
  }
}
