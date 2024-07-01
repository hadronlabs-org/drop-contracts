"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const proto_signing_1 = require("@cosmjs/proto-signing");
const client_ts_1 = require("@neutron-org/client-ts");
const dotenv_1 = __importDefault(require("dotenv"));
const cosmwasm_stargate_1 = require("@cosmjs/cosmwasm-stargate");
const stargate_1 = require("@cosmjs/stargate");
const tendermint_rpc_1 = require("@cosmjs/tendermint-rpc");
const logger_1 = require("./logger");
const config_1 = require("./config");
const factoryContract_1 = require("./factoryContract");
const validators_stats_1 = require("./modules/validators-stats");
dotenv_1.default.config();
class Service {
    context;
    modulesList = [];
    workHandler;
    log;
    constructor() {
        logger_1.logger.level = 'debug';
        this.log = logger_1.logger.child({ context: 'main' });
        process.on('SIGINT', () => {
            this.log.info('Stopping manager service...');
            clearInterval(this.workHandler);
            process.exit(0);
        });
    }
    async init() {
        const config = new config_1.Config(this.log);
        const neutronWallet = await proto_signing_1.DirectSecp256k1HdWallet.fromMnemonic(config.coordinator.mnemonic, {
            prefix: 'neutron',
        });
        const targetWallet = await proto_signing_1.DirectSecp256k1HdWallet.fromMnemonic(config.coordinator.mnemonic, {
            prefix: config.target.accountPrefix,
        });
        const targetTmClient = await tendermint_rpc_1.Tendermint34Client.connect(config.target.rpc);
        const neutronTmClient = await tendermint_rpc_1.Tendermint34Client.connect(config.neutron.rpc);
        const neutronSigningClient = await cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(config.neutron.rpc, neutronWallet, {
            gasPrice: config.neutron.gasPrice,
        });
        const factoryContractHandler = new factoryContract_1.FactoryContractHandler(neutronSigningClient, config.coordinator.factoryContractAddress);
        await factoryContractHandler.connect();
        this.context = {
            config: config,
            neutronWallet,
            factoryContractHandler,
            neutronWalletAddress: (await neutronWallet.getAccounts())[0].address,
            targetWallet,
            targetWalletAddress: (await targetWallet.getAccounts())[0].address,
            neutronTmClient,
            neutronQueryClient: stargate_1.QueryClient.withExtensions(neutronTmClient, stargate_1.setupBankExtension),
            neutronClient: new client_ts_1.Client({
                apiURL: config.neutron.rest,
                rpcURL: config.neutron.rpc,
                prefix: 'neutron',
            }),
            neutronSigningClient,
            targetSigningClient: await cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(config.target.rpc, targetWallet, {
                gasPrice: config.target.gasPrice,
            }),
            targetTmClient,
            targetQueryClient: stargate_1.QueryClient.withExtensions(targetTmClient, stargate_1.setupStakingExtension, stargate_1.setupBankExtension),
        };
    }
    registerModules() {
        this.modulesList.push(
        // new PumpModule(this.context, logger.child({ context: 'PumpModule' })),
        // new CoreModule(this.context, logger.child({ context: 'CoreModule' })),
        new validators_stats_1.ValidatorsStatsModule(this.context, logger_1.logger.child({ context: 'ValidatorsStatsModule' })));
    }
    start() {
        this.workHandler = setInterval(() => this.performWork(), this.context.config.coordinator.checksPeriod * 1000);
    }
    async showStats() {
        const balance = await this.context.neutronQueryClient.bank.balance(this.context.neutronWalletAddress, 'untrn');
        this.log.info(`Coordinator address state: ${balance.amount}${balance.denom}`);
    }
    async performWork() {
        await this.showStats();
        if (this.context.factoryContractHandler.skip ||
            this.context.factoryContractHandler.connected) {
            for (const module of this.modulesList) {
                await module.run();
            }
        }
        else {
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
