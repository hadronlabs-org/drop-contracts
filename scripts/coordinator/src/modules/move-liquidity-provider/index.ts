import { ManagerModule } from '../../types/Module';
import {
  LCDClient,
  Wallet,
  MnemonicKey,
  MsgExecute,
  bcs,
} from '@initia/initia.js';
import { MoveLiquidityProviderConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';

export class MoveLiquidityProviderModule extends ManagerModule {
  constructor(
    private context: Context,
    private log: pino.Logger,
    lpModuleAddress: string,
    lpModuleObject: string,
  ) {
    super();
    const lcd = new LCDClient(context.config.target.rest, {
      chainId: context.config.target.chainId,
      gasPrices: context.config.target.gasPrice.toString(),
      gasAdjustment: context.config.target.gasAdjustment,
    });
    const key = new MnemonicKey({
      mnemonic: context.config.coordinator.mnemonic,
    });
    const wallet = new Wallet(lcd, key);
    this._config = {
      lcd,
      wallet,
      moduleAddress: lpModuleAddress,
      moduleObject: lpModuleObject,
    };
  }

  private _config: MoveLiquidityProviderConfig;
  get config(): MoveLiquidityProviderConfig {
    return this._config;
  }

  async run(): Promise<void> {
    this._lastRun = Date.now();

    const msg = new MsgExecute(
      this.config.wallet.key.accAddress,
      this.config.moduleAddress,
      'drop_lp',
      'provide',
      [],
      [bcs.address().serialize(this.config.moduleObject).toBase64()],
    );
    const signedTx = await this.config.wallet.createAndSignTx({
      msgs: [msg],
      memo: 'sample memo',
    });
    const broadcastResult = await this.config.lcd.tx.broadcast(signedTx);
    this.log.info(
      `Move LP module tx broadcast result: ${JSON.stringify(broadcastResult)}`,
    );
  }

  static verifyConfig(
    log: pino.Logger,
    lpModuleAddress: string | undefined,
    lpModuleObject: string | undefined,
  ): boolean {
    if (!lpModuleAddress || !lpModuleObject) {
      log.error('move LP configuration is wrong');
      return false;
    }

    return true;
  }
}
