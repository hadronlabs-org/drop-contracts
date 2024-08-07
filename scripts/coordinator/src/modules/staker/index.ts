import { ManagerModule } from '../../types/Module';
import { DropStaker } from 'drop-ts-client';
import { StakerConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';

const StakerContractClient = DropStaker.Client;

export class StakerModule extends ManagerModule {
  contractClient?: InstanceType<typeof StakerContractClient>;
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

    const res = await this.contractClient.iBCTransfer(
      this.context.neutronWalletAddress,
      1.5,
      undefined,
      [
        {
          amount: this.context.config.neutron.icaFee,
          denom: 'untrn',
        },
      ],
    );

    this.log.info(`IBC transfer response: ${JSONBig.stringify(res)}`);
  }

  prepareConfig(): StakerConfig {
    this._config = {
      contractAddress:
        process.env.STAKER_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.staker_contract,
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
