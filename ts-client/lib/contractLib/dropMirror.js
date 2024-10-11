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
    queryOne = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { one: args });
    };
    queryAll = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { all: args });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    bond = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { bond: args }, fee || "auto", memo, funds);
    };
    updateConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
    };
    complete = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { complete: args }, fee || "auto", memo, funds);
    };
    changeReturnType = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { change_return_type: args }, fee || "auto", memo, funds);
    };
    updateBond = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_bond: args }, fee || "auto", memo, funds);
    };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
    };
}
exports.Client = Client;
