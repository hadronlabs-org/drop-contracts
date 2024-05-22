import { ManagerModule } from '../../types/Module';
import { DropValidatorsStats } from '../../generated/contractLib';
import { ValidatorsStatsConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import { runQueryRelayer } from '../../utils';

const ValidatorsStatsContractClient = DropValidatorsStats.Client;

export class ValidatorsStatsModule implements ManagerModule {
  private contractClient?: InstanceType<typeof ValidatorsStatsContractClient>;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    this.prepareConfig();

    this.contractClient = new ValidatorsStatsContractClient(
      this.context.neutronSigningClient,
      this.config.contractAddress,
    );
  }

  private _config: ValidatorsStatsConfig;
  get config(): ValidatorsStatsConfig {
    return this._config;
  }

  async run(): Promise<void> {
    const queryIds = await this.contractClient.queryKVQueryIds();

    this.log.info(`Validator stats query ids: ${JSON.stringify(queryIds)}`);

    const queryIdsArray = Object.values(queryIds).filter((id) => !!id);

    if (queryIdsArray.length > 0) {
      runQueryRelayer(this.context, this.log, queryIdsArray);
    }
  }

  prepareConfig(): void {
    this._config = {
      contractAddress: process.env.VALIDATOR_STATS_CONTRACT_ADDRESS,
    };
  }

  static verifyConfig(log: pino.Logger): boolean {
    if (!process.env.VALIDATOR_STATS_CONTRACT_ADDRESS) {
      log.error('VALIDATOR_STATS_CONTRACT_ADDRESS is not provided');
      return false;
    }

    return true;
  }
}
