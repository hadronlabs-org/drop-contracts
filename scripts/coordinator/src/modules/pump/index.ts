import { ManagerModule } from '../../types/Module';
import { DropPump } from 'drop-ts-client';
import { PumpConfig } from './types/config';
import { Context } from '../../types/Context';
import { Uint64 } from '@cosmjs/math';
import pino from 'pino';
import JSONBig from 'json-bigint';

const PumpContractClient = DropPump.Client;

export class PumpModule extends ManagerModule {
  contractClient?: InstanceType<typeof PumpContractClient>;
  icaAddress?: string;
  contractAddress: string;
  minBalance: string | undefined;
  constructor(
    contractAddress: string,
    minBalance: string | undefined,
    private context: Context,
    private log: pino.Logger,
  ) {
    super();
    this.prepareConfig(contractAddress, minBalance);

    this.contractClient = new DropPump.Client(
      this.context.neutronSigningClient,
      this.config.contractAddress,
    );
  }

  private _config: PumpConfig;
  get config(): PumpConfig {
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

    const balance = await this.context.targetQueryClient.bank.balance(
      this.icaAddress,
      this.context.config.target.denom,
    );

    const balanceAmount = Uint64.fromString(balance.amount);

    if (balanceAmount > this.config.minBalance) {
      this.log.info(`Pushing ${balanceAmount} coins to Neutron wallet...`);
      const res = await this.contractClient.push(
        this.context.neutronWalletAddress,
        {
          coins: [
            {
              amount: balanceAmount.toString(),
              denom: this.context.config.target.denom,
            },
          ],
        },
        1.5,
        undefined,
        [
          {
            amount: this.context.config.neutron.icaFee,
            denom: 'untrn',
          },
        ],
      );

      this.log.info(`Push response: ${JSONBig.stringify(res)}`);
    }
  }

  prepareConfig(
    contractAddress: string,
    minBalance: string | undefined,
  ): PumpConfig {
    this._config = {
      contractAddress: contractAddress,
      minBalance: Uint64.fromString(minBalance ?? '1000'),
    };

    return this.config;
  }

  static verifyConfig(
    log: pino.Logger,
    contractAddress: string | undefined,
  ): boolean {
    if (!contractAddress) {
      log.error('Pump contract address is not provided');
      return false;
    }

    return true;
  }
}
