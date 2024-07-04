"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FactoryContractHandler = void 0;
const logger_1 = require("./logger");
const contractLib_1 = require("./generated/contractLib");
class FactoryContractHandler {
    signingClient;
    factoryContractAddress;
    log;
    contractClient;
    constructor(signingClient, factoryContractAddress) {
        this.signingClient = signingClient;
        this.factoryContractAddress = factoryContractAddress;
        this.log = logger_1.logger.child({ context: 'factoryContract' });
    }
    get skip() {
        return !this.factoryContractAddress;
    }
    _connected = false;
    get connected() {
        return this._connected;
    }
    _factoryState;
    get factoryState() {
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
        this.contractClient = new contractLib_1.DropFactory.Client(this.signingClient, this.factoryContractAddress);
        await this.reconnect();
    }
    async reconnect() {
        try {
            this._factoryState = await this.contractClient.queryState();
            this._connected = true;
        }
        catch (e) {
            this.log.error('Unable to query factory contract state', e);
        }
    }
}
exports.FactoryContractHandler = FactoryContractHandler;
