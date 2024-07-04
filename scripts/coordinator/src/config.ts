import pino from 'pino';
import { GasPrice } from '@cosmjs/stargate';

function throwExceptionOnUndefined(value: any, name: string): any {
  return (
    value ||
    (() => {
      throw `${name} parameter in coordinator script is undefined`;
    })()
  );
}

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
  };
  target: {
    rpc: string;
    rest: string;
    denom: string;
    gasPrice: GasPrice;
    accountPrefix: string;
    validatorAccountPrefix: string;
  };

  constructor(private logContext: pino.Logger) {
    this.load();
  }

  load() {
    this.coordinator = {
      mnemonic: throwExceptionOnUndefined(
        process.env.COORDINATOR_MNEMONIC,
        'coordinator_mnemonic',
      ),
      factoryContractAddress: throwExceptionOnUndefined(
        process.env.FACTORY_CONTRACT_ADDRESS,
        'factory_contract_address',
      ),
      icqRunCmd: throwExceptionOnUndefined(
        process.env.ICQ_RUN_COMMAND,
        'icq_run_command',
      ),
      checksPeriod: throwExceptionOnUndefined(
        process.env.COORDINATOR_CHECKS_PERIOD,
        'coordinator_checks_period',
      ),
    };

    this.neutron = {
      rpc: throwExceptionOnUndefined(
        process.env.RELAYER_NEUTRON_CHAIN_RPC_ADDR,
        'relayer_neutron_chain_rps_addr',
      ),
      rest: throwExceptionOnUndefined(
        process.env.RELAYER_NEUTRON_CHAIN_REST_ADDR,
        'relayer_neutron_chain_rest_addr',
      ),
      gasPrice: GasPrice.fromString(
        throwExceptionOnUndefined(
          process.env.RELAYER_NEUTRON_CHAIN_GAS_PRICES,
          'relayer_neutron_chain_gas_price',
        ),
      ),
      gasAdjustment: throwExceptionOnUndefined(
        process.env.NEUTRON_GAS_ADJUSTMENT,
        'neutron_gas_adjustment',
      ),
    };

    this.target = {
      rpc: throwExceptionOnUndefined(
        process.env.RELAYER_TARGET_CHAIN_RPC_ADDR,
        'relayer_target_chain_rpc_addr',
      ),
      rest: throwExceptionOnUndefined(
        process.env.RELAYER_TARGET_CHAIN_REST_ADDR,
        'relayer_target_chain_rest_addr',
      ),
      denom: throwExceptionOnUndefined(
        process.env.RELAYER_TARGET_CHAIN_DENOM,
        'relayer_target_chain_denom',
      ),
      gasPrice: GasPrice.fromString(
        throwExceptionOnUndefined(
          process.env.RELAYER_TARGET_CHAIN_GAS_PRICES,
          'relayer_target_chain_gas_price',
        ),
      ),
      accountPrefix: throwExceptionOnUndefined(
        process.env.RELAYER_TARGET_CHAIN_ACCOUNT_PREFIX,
        'relayer_target_chain_account_prefix',
      ),
      validatorAccountPrefix: throwExceptionOnUndefined(
        process.env.RELAYER_TARGET_CHAIN_VALIDATOR_ACCOUNT_PREFIX,
        'relayer_target_chain_validator_account_prefix',
      ),
    };

    this.logContext.info('Config loaded');
  }
}
