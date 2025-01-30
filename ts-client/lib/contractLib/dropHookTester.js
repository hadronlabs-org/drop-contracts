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
    queryAnswers = async () => {
        return this.client.queryContractSmart(this.contractAddress, { answers: {} });
    };
    queryErrors = async () => {
        return this.client.queryContractSmart(this.contractAddress, { errors: {} });
    };
    setConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setConfigMsg(args), fee || "auto", memo, funds);
    };
    setConfigMsg = (args) => { return { set_config: args }; };
    undelegate = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.undelegateMsg(args), fee || "auto", memo, funds);
    };
    undelegateMsg = (args) => { return { undelegate: args }; };
    redelegate = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.redelegateMsg(args), fee || "auto", memo, funds);
    };
    redelegateMsg = (args) => { return { redelegate: args }; };
    tokenizeShare = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.tokenizeShareMsg(args), fee || "auto", memo, funds);
    };
    tokenizeShareMsg = (args) => { return { tokenize_share: args }; };
    redeemShare = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.redeemShareMsg(args), fee || "auto", memo, funds);
    };
    redeemShareMsg = (args) => { return { redeem_share: args }; };
    puppeteerHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.puppeteerHookMsg(args), fee || "auto", memo, funds);
    };
    puppeteerHookMsg = (args) => { return { puppeteer_hook: args }; };
}
exports.Client = Client;
