import { ManagerModule } from '../../types/Module';
import { DropStaker } from 'drop-ts-client';
import { StakerConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';

const StakerContractClient = DropStaker.Client;

export class StakerModule implements ManagerModule {
  contractClient?: InstanceType<typeof StakerContractClient>;
  icaAddress?: string;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {}

  private _config: StakerConfig;
  get config(): StakerConfig {
    return this._config;
  }

  init() {
    this.prepareConfig();

    if (this.config.contractAddress) {
      this.contractClient = new DropStaker.Client(
        this.context.neutronSigningClient,
        this.config.contractAddress,
      );
    }
  }

  async run(): Promise<void> {
    if (!this.contractClient) {
      this.init();
    }

    const res = await this.contractClient.iBCTransfer(
      this.context.neutronWalletAddress,
      1.5,
      undefined,
      [
        {
          amount: '20000',
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
