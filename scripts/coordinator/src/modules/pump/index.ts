import { ManagerModule } from '../../types/Module';
import { DropPump } from 'drop-ts-client';
import { PumpConfig } from './types/config';
import { Context } from '../../types/Context';
import { Uint64 } from '@cosmjs/math';
import pino from 'pino';
import JSONBig from 'json-bigint';

const PumpContractClient = DropPump.Client;

export class PumpModule implements ManagerModule {
  contractClient?: InstanceType<typeof PumpContractClient>;
  icaAddress?: string;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    this.prepareConfig();

    this.contractClient = new DropPump.Client(
      this.context.neutronSigningClient,
      this.config.contractAddress,
    );
  }

  private _config: PumpConfig;
  get config(): PumpConfig {
    return this._config;
  }

  private _lastRun: number = 0;
  get lastRun(): number {
    return this._lastRun;
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
            amount: '20000',
            denom: 'untrn',
          },
        ],
      );

      this.log.info(`Push response: ${JSONBig.stringify(res)}`);
    }
  }

  prepareConfig(): PumpConfig {
    this._config = {
      contractAddress: process.env.PUMP_CONTRACT_ADDRESS,
      minBalance: Uint64.fromString(process.env.PUMP_MIN_BALANCE ?? '1000'),
    };

    return this.config;
  }

  static verifyConfig(log: pino.Logger): boolean {
    if (!process.env.PUMP_CONTRACT_ADDRESS) {
      log.error('PUMP_CONTRACT_ADDRESS is not provided');
      return false;
    }

    return true;
  }
}
