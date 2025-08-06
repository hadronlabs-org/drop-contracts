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
import { connectComet } from '@cosmjs/tendermint-rpc';
import { PumpModule } from './modules/pump';
import { logger } from './logger';
import { Config } from './config';
import { Context } from './types/Context';
import pino from 'pino';
import { FactoryContractHandler } from './factoryContract';
import { ValidatorsStatsModule } from './modules/validators-stats';
import { CoreModule } from './modules/core';
import { SplitterModule } from './modules/splitter';
import { MoveLiquidityProviderModule } from './modules/move-liquidity-provider';

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
    const targetCometClient = await connectComet(config.target.rpc);
    const neutronCometClient = await connectComet(config.neutron.rpc);

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
      neutronCometClient: neutronCometClient,
      neutronQueryClient: QueryClient.withExtensions(
        neutronCometClient,
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
      targetCometClient: targetCometClient,
      targetQueryClient: QueryClient.withExtensions(
        targetCometClient,
        setupStakingExtension,
        setupBankExtension,
      ),
    };

    this.log.info(
      `Coordinator account address: ${this.context.neutronWalletAddress}`,
    );
  }

  startMonitoringConnection() {
    this.log.info('Starting connection monitoring service...');
    setInterval(async () => {
      const { factoryContractHandler } = this.context;
      if (factoryContractHandler) {
        try {
          await factoryContractHandler.contractClient.queryState();
          this.modulesWatcher();
        } catch (error) {
          console.error('Connection lost. Restarting coordinator...');
          process.exit();
        }
      } else {
        console.error('Client is not initialized. Waiting...');
      }
    }, 10000); // Check every 10 seconds, not recommended to set it higher
  }

  private modulesWatcher(): void {
    const currentTime = Date.now();
    for (const module of this.modulesList) {
      if (
        module.lastRun != 0 &&
        currentTime - module.lastRun >
          this.context.config.coordinator.checksPeriod * 3 * 1000
      ) {
        console.error(
          `${module.constructor.name} is not running. Restarting coordinator...`,
        );
        process.exit();
      }
    }
  }

  registerModules() {
    const pumpDenomAllowlist = [
      process.env.PUMP_DENOM || this.context.config.target.denom,
    ];
    const rewardsPumpDenomAllowlist = [
      process.env.REWARDS_PUMP_DENOM || this.context.config.target.denom,
    ];

    if (
      PumpModule.verifyConfig(
        this.log,
        process.env.PUMP_CONTRACT_ADDRESS,
        pumpDenomAllowlist,
      )
    ) {
      this.modulesList.push(
        new PumpModule(
          process.env.PUMP_CONTRACT_ADDRESS,
          pumpDenomAllowlist,
          process.env.PUMP_MIN_BALANCE,
          this.context,
          logger.child({ context: 'PumpModule' }),
        ),
      );
    }

    if (
      PumpModule.verifyConfig(
        this.log,
        process.env.REWARDS_PUMP_CONTRACT_ADDRESS,
        rewardsPumpDenomAllowlist,
      )
    ) {
      this.modulesList.push(
        new PumpModule(
          process.env.REWARDS_PUMP_CONTRACT_ADDRESS,
          rewardsPumpDenomAllowlist,
          process.env.REWARDS_PUMP_MIN_BALANCE,
          this.context,
          logger.child({ context: 'RewardsPumpModule' }),
        ),
      );
    }

    if (
      SplitterModule.verifyConfig(
        this.log,
        this.context.factoryContractHandler.skip,
      )
    ) {
      this.modulesList.push(
        new SplitterModule(
          this.context,
          logger.child({ context: 'SplitterModule' }),
        ),
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

    if (
      MoveLiquidityProviderModule.verifyConfig(
        this.log,
        process.env.INITIA_LP_MODULE_ADDRESS,
        process.env.INITIA_LP_MODULE_OBJECT,
      )
    ) {
      this.modulesList.push(
        new MoveLiquidityProviderModule(
          this.context,
          logger.child({ context: 'MoveLiquidityProviderModule' }),
          process.env.INITIA_LP_MODULE_ADDRESS,
          process.env.INITIA_LP_MODULE_OBJECT,
          BigInt(process.env.INITIA_LP_MIN_AMOUNT_TO_PROVIDE || '20000000'),
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
      await this.context.neutronCometClient.block()
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
          this.log.info(`Running ${module.constructor.name} module...`);
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
  service.startMonitoringConnection();
  service.registerModules();
  service.start();
}

main();
