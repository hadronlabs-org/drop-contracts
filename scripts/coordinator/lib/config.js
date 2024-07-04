"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Config = void 0;
const stargate_1 = require("@cosmjs/stargate");
class Config {
    logContext;
    coordinator;
    neutron;
    target;
    constructor(logContext) {
        this.logContext = logContext;
        this.load();
    }
    load() {
        this.coordinator = {
            mnemonic: process.env.COORDINATOR_MNEMONIC,
            factoryContractAddress: process.env.FACTORY_CONTRACT_ADDRESS,
            icqRunCmd: process.env.ICQ_RUN_COMMAND,
            checksPeriod: parseInt(process.env.CHECKS_PERIOD, 10),
        };
        this.neutron = {
            rpc: process.env.RELAYER_NEUTRON_CHAIN_RPC_ADDR,
            rest: process.env.RELAYER_NEUTRON_CHAIN_REST_ADDR,
            gasPrice: stargate_1.GasPrice.fromString(process.env.RELAYER_NEUTRON_CHAIN_GAS_PRICES),
            gasAdjustment: process.env.NEUTRON_GAS_ADJUSTMENT,
        };
        this.target = {
            rpc: process.env.RELAYER_TARGET_CHAIN_RPC_ADDR,
            rest: process.env.RELAYER_TARGET_CHAIN_REST_ADDR,
            denom: process.env.RELAYER_TARGET_CHAIN_DENOM,
            gasPrice: stargate_1.GasPrice.fromString(process.env.RELAYER_TARGET_CHAIN_GAS_PRICES),
            accountPrefix: process.env.RELAYER_TARGET_CHAIN_ACCOUNT_PREFIX,
            validatorAccountPrefix: process.env.RELAYER_TARGET_CHAIN_VALIDATOR_ACCOUNT_PREFIX,
        };
        this.logContext.info('Config loaded');
    }
}
exports.Config = Config;
