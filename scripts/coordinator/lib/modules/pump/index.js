"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PumpModule = void 0;
const contractLib_1 = require("../../generated/contractLib");
const math_1 = require("@cosmjs/math");
const PumpContractClient = contractLib_1.DropPump.Client;
class PumpModule {
    context;
    log;
    contractClient;
    icaAddress;
    constructor(context, log) {
        this.context = context;
        this.log = log;
        this.prepareConfig();
        this.contractClient = new contractLib_1.DropPump.Client(this.context.neutronSigningClient, this.config.contractAddress);
        this.contractClient.queryIca().then((result) => {
            if (result.registered.ica_address) {
                this.icaAddress = result.registered.ica_address;
                this.log.info(`Pump ICA address: ${this.icaAddress}`);
            }
            else {
                throw new Error('ICA address not found');
            }
        });
    }
    _config;
    get config() {
        return this._config;
    }
    async run() {
        const balance = await this.context.targetQueryClient.bank.balance(this.icaAddress, this.context.config.target.denom);
        const balanceAmount = math_1.Uint64.fromString(balance.amount);
        if (balanceAmount > this.config.minBalance) {
            this.contractClient.push(this.context.neutronWalletAddress, {
                coins: [
                    {
                        amount: balanceAmount.toString(),
                        denom: this.context.config.target.denom,
                    },
                ],
            }, 1.5, undefined, [
                {
                    amount: '20000',
                    denom: 'untrn',
                },
            ]);
        }
    }
    prepareConfig() {
        this._config = {
            contractAddress: process.env.PUMP_CONTRACT_ADDRESS,
            minBalance: math_1.Uint64.fromString(process.env.PUMP_MIN_BALANCE),
        };
        return this.config;
    }
}
exports.PumpModule = PumpModule;
