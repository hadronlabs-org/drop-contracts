import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { Client as NeutronClient } from '@neutron-org/client-ts';
import { ManagerModule } from './types/Module';
import dotenv from 'dotenv';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import {
  QueryClient,
  setupBankExtension,
  setupStakingExtension,
} from '@cosmjs/stargate';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { PumpModule } from './modules/pump';
import { logger } from './logger';
import { Config } from './config';
import { Context } from './types/Context';
import pino from 'pino';
import { FactoryContractHandler } from './factoryContract';
import { ValidatorsStatsModule } from './modules/validators-stats';
import { CoreModule } from './modules/core';
import { StakerModule } from './modules/staker';

export type Uint128 = string;

dotenv.config();

class Service {
  private context: Context;
  private modulesList: ManagerModule[] = [];
  private workHandler: NodeJS.Timeout;
  private log: pino.Logger;

  constructor() {
    logger.level = 'debug';
    this.log = logger.child({ context: 'main' });

    process.on('SIGINT', () => {
      this.log.info('Stopping manager service...');
      clearInterval(this.workHandler);

      process.exit(0);
    });
  }

  async init() {
    const config = new Config(this.log);
    const neutronWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      config.coordinator.mnemonic,
      {
        prefix: 'neutron',
      },
    );
    const targetWallet = await DirectSecp256k1HdWallet.fromMnemonic(
      config.coordinator.mnemonic,
      {
        prefix: config.target.accountPrefix,
      },
    );
    const targetTmClient = await Tendermint34Client.connect(config.target.rpc);
    const neutronTmClient = await Tendermint34Client.connect(
      config.neutron.rpc,
    );

    const neutronSigningClient = await SigningCosmWasmClient.connectWithSigner(
      config.neutron.rpc,
      neutronWallet,
      {
        gasPrice: config.neutron.gasPrice,
      },
    );

    const factoryContractHandler = new FactoryContractHandler(
      neutronSigningClient,
      config.coordinator.factoryContractAddress,
    );
    await factoryContractHandler.connect();

    this.context = {
      config: config,
      neutronWallet,
      height: 0,
      factoryContractHandler,
      neutronWalletAddress: (await neutronWallet.getAccounts())[0].address,
      targetWallet,
      targetWalletAddress: (await targetWallet.getAccounts())[0].address,
      neutronTmClient,
      neutronQueryClient: QueryClient.withExtensions(
        neutronTmClient,
        setupBankExtension,
      ),
      neutronClient: new NeutronClient({
        apiURL: config.neutron.rest,
        rpcURL: config.neutron.rpc,
        prefix: 'neutron',
      }),
      neutronSigningClient,
      targetSigningClient: await SigningCosmWasmClient.connectWithSigner(
        config.target.rpc,
        targetWallet,
        {
          gasPrice: config.target.gasPrice,
        },
      ),
      targetTmClient,
      targetQueryClient: QueryClient.withExtensions(
        targetTmClient,
        setupStakingExtension,
        setupBankExtension,
      ),
    };

    this.log.info(`Coordinator account address: ${this.context.neutronWalletAddress}`);
  }

  registerModules() {
    if (PumpModule.verifyConfig(this.log)) {
      this.modulesList.push(
        new PumpModule(this.context, logger.child({ context: 'PumpModule' })),
      );
    }

    if (StakerModule.verifyConfig(this.log, this.context.factoryContractHandler.skip)) {
      this.modulesList.push(
        new StakerModule(this.context, logger.child({ context: 'StakerModule' })),
      );
    }

    if (
      CoreModule.verifyConfig(
        this.log,
        this.context.factoryContractHandler.skip,
      )
    ) {
      this.modulesList.push(
        new CoreModule(this.context, logger.child({ context: 'CoreModule' })),
      );
    }

    if (ValidatorsStatsModule.verifyConfig(this.log)) {
      this.modulesList.push(
        new ValidatorsStatsModule(
          this.context,
          logger.child({ context: 'ValidatorsStatsModule' }),
        ),
      );
    }
  }

  start() {
    this.workHandler = setInterval(
      () => this.performWork(),
      this.context.config.coordinator.checksPeriod * 1000,
    );
  }

  async showStats() {
    const balance = await this.context.neutronQueryClient.bank.balance(
      this.context.neutronWalletAddress,
      'untrn',
    );
    this.context.height = (
      await this.context.neutronTmClient.block()
    ).block.header.height;

    this.log.info(
      `Coordinator address state: ${balance.amount}${balance.denom}, height: ${this.context.height}`,
    );
  }

  async performWork() {
    await this.showStats();
    if (
      this.context.factoryContractHandler.skip ||
      this.context.factoryContractHandler.connected
    ) {
      for (const module of this.modulesList) {
        try {
          this.log.info(
            `Running ${module.constructor.name} module...`,
          );
          await module.run();
        } catch (error) {
          this.log.error(
            `Error running module ${module.constructor.name}: ${error.message}`,
          );
        }
      }
    } else {
      this.log.info('Factory contract not connected, skipping work');
      await this.context.factoryContractHandler.reconnect();
    }
  }
}

async function main() {
  const service = new Service();
  await service.init();
  service.registerModules();
  service.start();
}

main();
