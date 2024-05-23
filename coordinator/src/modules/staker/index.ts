import { ManagerModule } from '../../types/Module';
import { DropStaker } from 'drop-ts-client';
import { StakerConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';

const StakerContractClient = DropStaker.Client;

export class StakerModule implements ManagerModule {
  contractClient?: InstanceType<typeof StakerContractClient>;
  icaAddress?: string;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
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
    if (!this.icaAddress) {
      const res = await this.contractClient.queryIca();
      if ((res as any).registered && (res as any).registered.ica_address) {
        this.icaAddress = (res as any).registered.ica_address;
      } else {
        this.log.error('ICA address not found');
        return;
      }
    }

    await this.contractClient.iBCTransfer(
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
