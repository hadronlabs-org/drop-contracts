import { ManagerModule } from '../../types/Module';
import {
  DropPuppeteer,
  DropCore,
  DropNativeBondProvider,
  DropLsmShareBondProvider,
} from 'drop-ts-client';
import { CoreConfig } from './types/config';
import { Context } from '../../types/Context';
import pino from 'pino';
import JSONBig from 'json-bigint';

import { runQueryRelayer, waitBlocks } from '../../utils';
import { fromAscii, toAscii } from '@cosmjs/encoding';

const PuppeteerContractClient = DropPuppeteer.Client;
const CoreContractClient = DropCore.Client;
const NativeBondProviderContractClient = DropNativeBondProvider.Client;
const LSMShareBondProviderContractClient = DropLsmShareBondProvider.Client;

const IDLE_ADDITIONAL_INTERVAL = 120; // Seconds. Coordinator idle timeout calculation is a little frontrunning before actual idle timeout

export class CoreModule extends ManagerModule {
  private puppeteerContractClient?: InstanceType<
    typeof PuppeteerContractClient
  >;
  private coreContractClient?: InstanceType<typeof CoreContractClient>;
  private nativeBondProviderContractClient?: InstanceType<
    typeof NativeBondProviderContractClient
  >;
  private lsmShareBondProviderContractClient?: InstanceType<
    typeof LSMShareBondProviderContractClient
  >;

  constructor(
    private context: Context,
    private log: pino.Logger,
  ) {
    super();
  }

  private _config: CoreConfig;
  get config(): CoreConfig {
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

    if (this.config.nativeBondProviderAddress) {
      this.nativeBondProviderContractClient =
        new NativeBondProviderContractClient(
          this.context.neutronSigningClient,
          this.config.nativeBondProviderAddress,
        );
    }

    if (this.config.lsmShareBondProviderAddress) {
      this.lsmShareBondProviderContractClient =
        new LSMShareBondProviderContractClient(
          this.context.neutronSigningClient,
          this.config.lsmShareBondProviderAddress,
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

    let lsmShareCanProcessOnIdle = false;
    try {
      lsmShareCanProcessOnIdle =
        this.lsmShareBondProviderContractClient &&
        (await this.lsmShareBondProviderContractClient.queryCanProcessOnIdle());
    } catch (e) {
      //
    }

    let nativeBondCanProcessOnIdle = false;
    try {
      nativeBondCanProcessOnIdle =
        this.nativeBondProviderContractClient &&
        (await this.nativeBondProviderContractClient.queryCanProcessOnIdle());
    } catch (e) {
      //
    }

    if (
      this.lastRun / 1000 <
        lastTick + config.idle_min_interval + IDLE_ADDITIONAL_INTERVAL &&
      coreContractState === 'idle'
    ) {
      if (!lsmShareCanProcessOnIdle && !nativeBondCanProcessOnIdle) {
        this.log.info(
          'Skipping idle tick because idle min interval is not reached',
        );
        return;
      }
    }

    const lastPuppeteerResponse =
      await this.coreContractClient.queryLastPuppeteerResponse();

    const puppeteerResponseReceived = !!lastPuppeteerResponse.response;

    this.log.debug(
      `Core contract state: ${coreContractState}, puppeteer response received: ${puppeteerResponseReceived}`,
    );

    if (puppeteerResponseReceived || coreContractState === 'idle') {
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
      nativeBondProviderAddress:
        process.env.NATIVE_BOND_PROVIDER_ADDRESS ||
        this.context.factoryContractHandler.factoryState
          .native_bond_provider_contract,
      lsmShareBondProviderAddress:
        process.env.LSM_SHARE_BOND_PROVIDER_ADDRESS ||
        this.context.factoryContractHandler.factoryState
          .lsm_share_bond_provider_contract,
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
