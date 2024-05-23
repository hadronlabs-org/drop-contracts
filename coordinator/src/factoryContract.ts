import pino from 'pino';
import { logger } from './logger';
import { DropFactory } from 'drop-ts-client';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { State } from 'drop-ts-client/lib/src/contractLib/dropFactory';

export class FactoryContractHandler {
  private log: pino.Logger;
  private contractClient: InstanceType<typeof DropFactory.Client>;

  constructor(
    private signingClient: SigningCosmWasmClient,
    private factoryContractAddress: string,
  ) {
    this.log = logger.child({ context: 'factoryContract' });
  }

  get skip(): boolean {
    return !this.factoryContractAddress;
  }

  private _connected: boolean = false;
  get connected(): boolean {
    return this._connected;
  }

  private _factoryState: State;
  get factoryState(): State {
    return this._factoryState;
  }

  async connect() {
    if (this.skip) {
      this.log.info('Factory contract address not provided, skipping');
      return;
    }
    if (this.connected) {
      return;
    }

    this.log.info('Connecting to factory contract...');

    this.contractClient = new DropFactory.Client(
      this.signingClient,
      this.factoryContractAddress,
    );

    await this.reconnect();
  }

  async reconnect() {
    try {
      this._factoryState = await this.contractClient.queryState();
      this._connected = true;
    } catch (e) {
      this.log.error('Unable to query factory contract state', e);
    }
  }
}
