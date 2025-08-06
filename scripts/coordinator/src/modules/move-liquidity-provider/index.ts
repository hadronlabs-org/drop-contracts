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
import { fromHex, toBech32 } from '@cosmjs/encoding';

export class MoveLiquidityProviderModule extends ManagerModule {
  lcd: LCDClient;

  constructor(
    private context: Context,
    private log: pino.Logger,
    lpModuleAddress: string,
    lpModuleObject: string,
    minAmountToProvide: bigint,
  ) {
    super();
    this.lcd = new LCDClient(context.config.target.rest, {
      chainId: context.config.target.chainId,
      gasPrices: context.config.target.gasPrice.toString(),
      gasAdjustment: context.config.target.gasAdjustment,
    });
    const key = new MnemonicKey({
      mnemonic: context.config.coordinator.mnemonic,
    });
    const wallet = new Wallet(this.lcd, key);
    this._config = {
      wallet,
      moduleAddress: lpModuleAddress,
      moduleObject: lpModuleObject,
      minAmountToProvide,
    };
  }

  private _config: MoveLiquidityProviderConfig;
  get config(): MoveLiquidityProviderConfig {
    return this._config;
  }

  async run(): Promise<void> {
    this._lastRun = Date.now();

    const bech32Address = toBech32(
      'init',
      fromHex(this.config.moduleObject.substring(2)),
    );

    const coin = await this.lcd.bank.balanceByDenom(bech32Address, 'uinit');

    const amount = BigInt(coin?.amount || '0');

    if (amount < this.config.minAmountToProvide) {
      this.log.info(
        `uInit amount is less than min amount to provide: ${amount} < ${this.config.minAmountToProvide}`,
      );

      return;
    }

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
    const broadcastResult = await this.lcd.tx.broadcast(signedTx);
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
      log.error('move LP configuration is incomplete');
      return false;
    }

    return true;
  }
}
