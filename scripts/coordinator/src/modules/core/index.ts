import { ManagerModule } from '../../types/Module';
import { DropPuppeteer, DropCore } from 'drop-ts-client';
import { PuppeteerConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';

import { runQueryRelayer, waitBlocks } from '../../utils';
import { fromAscii, toAscii } from '@cosmjs/encoding';

const PuppeteerContractClient = DropPuppeteer.Client;
const CoreContractClient = DropCore.Client;

const IDLE_ADDITIONAL_INTERVAL = 120; // Seconds. Coordinator idle timeout calculation is a little frontrunning before actual idle timeout

export class CoreModule extends ManagerModule {
  private puppeteerContractClient?: InstanceType<
    typeof PuppeteerContractClient
  >;
  private coreContractClient?: InstanceType<typeof CoreContractClient>;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    super();
  }

  private _config: PuppeteerConfig;
  get config(): PuppeteerConfig {
    return this._config;
  }

  init() {
    this.prepareConfig();

    if (this.config.puppeteerContractAddress) {
      this.puppeteerContractClient = new PuppeteerContractClient(
        this.context.neutronSigningClient,
        this.config.puppeteerContractAddress,
      );
    }

    if (this.config.coreContractAddress) {
      this.coreContractClient = new CoreContractClient(
        this.context.neutronSigningClient,
        this.config.coreContractAddress,
      );
    }
  }

  async run(): Promise<void> {
    this._lastRun = Date.now();
    if (!this.puppeteerContractClient || !this.coreContractClient) {
      this.init();
    }

    const coreContractState =
      await this.coreContractClient.queryContractState();

    const lastTickRaw =
      await this.context.neutronSigningClient.queryContractRaw(
        this.config.coreContractAddress,
        toAscii('last_tick'),
      );
    const lastTick = Number.parseInt(fromAscii(lastTickRaw), 10);

    const config = await this.coreContractClient.queryConfig();

    if (
      this.lastRun / 1000 <
        lastTick + config.idle_min_interval + IDLE_ADDITIONAL_INTERVAL &&
      coreContractState === 'idle'
    ) {
      const lastRedeemRaw =
        await this.context.neutronSigningClient.queryContractRaw(
          this.config.coreContractAddress,
          toAscii('last_lsm_redeem'),
        );
      const lastRedeem = Number.parseInt(fromAscii(lastRedeemRaw), 10);

      const pendingLsmSharesAmount = (
        await this.coreContractClient.queryPendingLSMShares()
      ).length;

      const lsmSharesToRedeemAmount = (
        await this.coreContractClient.queryLSMSharesToRedeem()
      ).length;

      if (pendingLsmSharesAmount === 0 && lsmSharesToRedeemAmount === 0) {
        this.log.info(
          'Skipping idle tick because idle min interval is not reached',
        );
        return;
      }

      if (
        pendingLsmSharesAmount === 0 &&
        lsmSharesToRedeemAmount < config.lsm_redeem_threshold &&
        lastRedeem + config.lsm_redeem_maximum_interval > this.lastRun / 1000
      ) {
        this.log.info('Skipping tick because pending LSM shares is not ready');
        return;
      }
    }

    const lastPuppeteerResponse =
      await this.coreContractClient.queryLastPuppeteerResponse();

    const puppeteerResponseReceived = !!lastPuppeteerResponse.response;

    const lastStakerResponse =
      await this.coreContractClient.queryLastStakerResponse();

    const stakerResponseReceived = !!lastStakerResponse.response;

    this.log.debug(
      `Core contract state: ${coreContractState}, puppeteer response received: ${puppeteerResponseReceived}, staker response received: ${stakerResponseReceived}`,
    );

    if (
      puppeteerResponseReceived ||
      coreContractState === 'idle' ||
      (stakerResponseReceived && coreContractState === 'staking_bond')
    ) {
      const queryIds = await this.puppeteerContractClient.queryKVQueryIds();

      this.log.info(`Puppeteer query ids: ${JSON.stringify(queryIds)}`);

      const queryIdsArray = queryIds.map(([queryId]) => queryId.toString());

      this.log.info(
        `Puppeteer query ids plain: ${JSONBig.stringify(queryIdsArray)}`,
      );

      if (queryIdsArray.length > 0) {
        runQueryRelayer(this.context, this.log, queryIdsArray);

        await waitBlocks(this.context, 3, this.log);

        const res = await this.coreContractClient.tick(
          this.context.neutronWalletAddress,
          1.5,
          undefined,
          [],
        );

        this.log.info(`Core contract tick response: ${JSONBig.stringify(res)}`);
      }
    }
  }

  prepareConfig(): void {
    this._config = {
      puppeteerContractAddress:
        process.env.PUPPETEER_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.puppeteer_contract,
      coreContractAddress:
        process.env.CORE_CONTRACT_ADDRESS ||
        this.context.factoryContractHandler.factoryState.core_contract,
    };
  }

  static verifyConfig(log: pino.Logger, skipFactory: boolean): boolean {
    if (skipFactory && !process.env.PUPPETEER_CONTRACT_ADDRESS) {
      log.error('PUPPETEER_CONTRACT_ADDRESS is not provided');
      return false;
    }

    if (skipFactory && !process.env.CORE_CONTRACT_ADDRESS) {
      log.error('CORE_CONTRACT_ADDRESS is not provided');
      return false;
    }

    return true;
  }
}
