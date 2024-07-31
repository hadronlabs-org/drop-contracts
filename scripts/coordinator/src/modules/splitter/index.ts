import { ManagerModule } from '../../types/Module';
import { DropSplitter } from 'drop-ts-client';
import { SplitterConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';

const SplitterContractClient = DropSplitter.Client;

export class SplitterModule implements ManagerModule {
  contractClient?: InstanceType<typeof SplitterContractClient>;
  icaAddress?: string;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    this.prepareConfig();

    this.contractClient = new DropSplitter.Client(
      this.context.neutronSigningClient,
      this.config.contractAddress,
    );
  }

  private _config: SplitterConfig;
  get config(): SplitterConfig {
    return this._config;
  }

  private _lastRun: number = 0;
  get lastRun(): number {
    return this._lastRun;
  }

  async run(): Promise<void> {
    this._lastRun = Date.now();

    const res = await this.contractClient.distribute(
      this.context.neutronWalletAddress,
      1.5,
      undefined,
    );

    this.log.info(`IBC transfer response: ${JSONBig.stringify(res)}`);
  }

  prepareConfig(): SplitterConfig {
    this._config = {
      contractAddress:
        process.env.SPLITTER_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.splitter_contract,
    };

    return this.config;
  }

  static verifyConfig(log: pino.Logger, skipFactory: boolean): boolean {
    if (skipFactory && !process.env.SPLITTER_CONTRACT_ADDRESS) {
      log.error('SPLITTER_CONTRACT_ADDRESS is not provided');
      return false;
    }

    return true;
  }
}
