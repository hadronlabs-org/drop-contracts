import { ManagerModule } from '../../types/Module';
import { DropCore, DropStaker } from 'drop-ts-client';
import { StakerConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';
import { Decimal } from '@cosmjs/math';

const StakerContractClient = DropStaker.Client;
const CoreContractClient = DropCore.Client;

export class StakerModule extends ManagerModule {
  contractClient?: InstanceType<typeof StakerContractClient>;
  coreContractClient?: InstanceType<typeof CoreContractClient>;
  icaAddress?: string;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    super();
    this.prepareConfig();

    this.contractClient = new DropStaker.Client(
      this.context.neutronSigningClient,
      this.config.contractAddress,
    );

    this.coreContractClient = new DropCore.Client(
      this.context.neutronSigningClient,
      this.config.coreContractAddress,
    );
  }

  private _config: StakerConfig;
  get config(): StakerConfig {
    return this._config;
  }

  async run(): Promise<void> {
    this._lastRun = Date.now();
    if (!this.icaAddress) {
      const res = await this.contractClient.queryIca();
      if ((res as any).registered && (res as any).registered.ica_address) {
        this.icaAddress = (res as any).registered.ica_address;
      } else {
        this.log.error('ICA address not found');
        return;
      }
    }

    const { base_denom: baseDenom } =
      await this.coreContractClient.queryConfig();

    const baseDenomBalance = await this.context.neutronQueryClient.bank.balance(
      this.config.contractAddress,
      baseDenom,
    );

    const balanceAmount = Decimal.fromAtomics(baseDenomBalance.amount, 0);

    const ntrnBalance = await this.context.neutronQueryClient.bank.balance(
      this.config.contractAddress,
      'untrn',
    );

    const ntrnBalanceAmount = Decimal.fromAtomics(ntrnBalance.amount, 0);

    this.log.info(
      `Contract balances: ${ntrnBalanceAmount}untrn, ${balanceAmount}${baseDenom}.`,
    );

    if (balanceAmount.isGreaterThanOrEqual(this.config.stakerMinBalance)) {
      this.log.info(`Transferring ${balanceAmount}${baseDenom} coins...`);

      const funds = ntrnBalanceAmount.isLessThan(this.config.icaFeeBuffer)
        ? [
            {
              amount: this.context.config.neutron.icaFee,
              denom: 'untrn',
            },
          ]
        : [];

      const res = await this.contractClient.iBCTransfer(
        this.context.neutronWalletAddress,
        1.5,
        undefined,
        funds,
      );

      this.log.info(`IBC transfer response: ${JSONBig.stringify(res)}`);

      return;
    }

    this.log.info(
      `Staker balance (${balanceAmount}) is less than the minimum required balance of ${this.config.stakerMinBalance}`,
    );
  }

  prepareConfig(): StakerConfig {
    this._config = {
      contractAddress:
        process.env.STAKER_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.staker_contract,
      coreContractAddress:
        process.env.CORE_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.core_contract,
      stakerMinBalance: Decimal.fromAtomics(
        process.env.STAKER_MIN_BALANCE ?? '1000000',
        0,
      ),
      icaFeeBuffer: Decimal.fromAtomics(
        process.env.ICA_FEE_COINS_BUFFER ?? '1000000',
        0,
      ),
    };

    return this.config;
  }

  static verifyConfig(log: pino.Logger, skipFactory: boolean): boolean {
    if (skipFactory && !process.env.STAKER_CONTRACT_ADDRESS) {
      log.error('STAKER_CONTRACT_ADDRESS is not provided');
      return false;
    }

    return true;
  }
}
