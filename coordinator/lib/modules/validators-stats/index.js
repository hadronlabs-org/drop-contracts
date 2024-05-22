"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ValidatorsStatsModule = void 0;
const contractLib_1 = require("../../generated/contractLib");
const utils_1 = require("../../utils");
const ValidatorsStatsContractClient = contractLib_1.DropValidatorsStats.Client;
class ValidatorsStatsModule {
    context;
    log;
    contractClient;
    constructor(context, log) {
        this.context = context;
        this.log = log;
        this.prepareConfig();
        this.contractClient = new ValidatorsStatsContractClient(this.context.neutronSigningClient, this.config.contractAddress);
    }
    _config;
    get config() {
        return this._config;
    }
    async run() {
        let queryIds;
        try {
            queryIds = await this.contractClient.queryQueryIds();
        }
        catch (error) {
            this.log.error(`Error querying contract query ids: ${error.message}`);
            return;
        }
        this.log.info(`Validator stats query ids: ${JSON.stringify(queryIds)}`);
        const queryIdsArray = Object.values(queryIds).filter((id) => !!id);
        (0, utils_1.runQueryRelayer)(this.context, this.log, queryIdsArray);
    }
    prepareConfig() {
        this._config = {
            contractAddress: process.env.VALIDATOR_STATS_CONTRACT_ADDRESS,
        };
        return this.config;
    }
    onFactoryConnected() {
        return Promise.resolve();
    }
}
exports.ValidatorsStatsModule = ValidatorsStatsModule;
