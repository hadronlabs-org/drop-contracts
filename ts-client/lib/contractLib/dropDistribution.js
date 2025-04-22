"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Client = void 0;
function isSigningCosmWasmClient(client) {
    return 'execute' in client;
}
class Client {
    client;
    contractAddress;
    constructor(client, contractAddress) {
        this.client = client;
        this.contractAddress = contractAddress;
    }
    mustBeSigningClient() {
        return new Error("This client is not a SigningCosmWasmClient");
    }
    static async instantiate(client, sender, codeId, initMsg, label, fees, initCoins, admin) {
        const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
            ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
        });
        return res;
    }
    static async instantiate2(client, sender, codeId, salt, initMsg, label, fees, initCoins, admin) {
        const res = await client.instantiate2(sender, codeId, salt, initMsg, label, fees, {
            ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
        });
        return res;
    }
    queryCalcDeposit = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { calc_deposit: args });
    };
    queryCalcWithdraw = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { calc_withdraw: args });
    };
}
exports.Client = Client;
