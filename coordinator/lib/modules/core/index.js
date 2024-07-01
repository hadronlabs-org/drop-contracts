"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.CoreModule = void 0;
const contractLib_1 = require("../../generated/contractLib");
const CoreContractClient = contractLib_1.DropCore.Client;
class CoreModule {
    context;
    log;
    contractClient;
    constructor(context, log) {
        this.context = context;
        this.log = log;
    }
    _config;
    get config() {
        return this._config;
    }
    init() {
        this._config = {
            contractAddress: process.env.CORE_CONTRACT_ADDRESS
                ? process.env.CORE_CONTRACT_ADDRESS
                : this.context.factoryContractHandler.factoryState.core_contract,
        };
        this.contractClient = new contractLib_1.DropCore.Client(this.context.neutronSigningClient, this.config.contractAddress);
        return this.config;
    }
    async run() {
        if (!this.contractClient) {
            this.init();
        }
        let contractState;
        let transferAck;
        try {
            contractState = await this.contractClient.queryContractState();
            transferAck = await this.contractClient.queryTransferAckReceived();
        }
        catch (error) {
            this.log.error(`Error querying contract state: ${error.message}`);
            return;
        }
        this.log.info(`Core contract state: ${contractState}, transfer ACK received: ${transferAck}`);
    }
}
exports.CoreModule = CoreModule;
