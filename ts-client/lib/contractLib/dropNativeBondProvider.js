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
    static async instantiate(client, sender, codeId, initMsg, label, fees, initCoins) {
        const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
            ...(initCoins && initCoins.length && { funds: initCoins }),
        });
        return res;
    }
    static async instantiate2(client, sender, codeId, salt, initMsg, label, fees, initCoins) {
        const res = await client.instantiate2(sender, codeId, new Uint8Array([salt]), initMsg, label, fees, {
            ...(initCoins && initCoins.length && { funds: initCoins }),
        });
        return res;
    }
    queryConfig = async () => {
        return this.client.queryContractSmart(this.contractAddress, { config: {} });
    };
    queryNonStakedBalance = async () => {
        return this.client.queryContractSmart(this.contractAddress, { non_staked_balance: {} });
    };
    queryAllBalance = async () => {
        return this.client.queryContractSmart(this.contractAddress, { all_balance: {} });
    };
    queryTxState = async () => {
        return this.client.queryContractSmart(this.contractAddress, { tx_state: {} });
    };
    queryLastPuppeteerResponse = async () => {
        return this.client.queryContractSmart(this.contractAddress, { last_puppeteer_response: {} });
    };
    queryCanBond = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { can_bond: args });
    };
    queryCanProcessOnIdle = async () => {
        return this.client.queryContractSmart(this.contractAddress, { can_process_on_idle: {} });
    };
    queryTokensAmount = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { tokens_amount: args });
    };
    queryAsyncTokensAmount = async () => {
        return this.client.queryContractSmart(this.contractAddress, { async_tokens_amount: {} });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    updateConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
    };
    puppeteerTransfer = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { puppeteer_transfer: {} }, fee || "auto", memo, funds);
    };
    puppeteerHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { puppeteer_hook: args }, fee || "auto", memo, funds);
    };
    bond = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { bond: {} }, fee || "auto", memo, funds);
    };
    processOnIdle = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { process_on_idle: {} }, fee || "auto", memo, funds);
    };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
    };
}
exports.Client = Client;
