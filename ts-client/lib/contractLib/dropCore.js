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
    queryOwner = async () => {
        return this.client.queryContractSmart(this.contractAddress, { owner: {} });
    };
    queryExchangeRate = async () => {
        return this.client.queryContractSmart(this.contractAddress, { exchange_rate: {} });
    };
    queryCurrentUnbondBatch = async () => {
        return this.client.queryContractSmart(this.contractAddress, { current_unbond_batch: {} });
    };
    queryUnbondBatch = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { unbond_batch: args });
    };
    queryUnbondBatches = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { unbond_batches: args });
    };
    queryContractState = async () => {
        return this.client.queryContractSmart(this.contractAddress, { contract_state: {} });
    };
    queryLastPuppeteerResponse = async () => {
        return this.client.queryContractSmart(this.contractAddress, { last_puppeteer_response: {} });
    };
    queryTotalBonded = async () => {
        return this.client.queryContractSmart(this.contractAddress, { total_bonded: {} });
    };
    queryBondProviders = async () => {
        return this.client.queryContractSmart(this.contractAddress, { bond_providers: {} });
    };
    queryTotalAsyncTokens = async () => {
        return this.client.queryContractSmart(this.contractAddress, { total_async_tokens: {} });
    };
    queryFailedBatch = async () => {
        return this.client.queryContractSmart(this.contractAddress, { failed_batch: {} });
    };
    queryPause = async () => {
        return this.client.queryContractSmart(this.contractAddress, { pause: {} });
    };
    queryBondHooks = async () => {
        return this.client.queryContractSmart(this.contractAddress, { bond_hooks: {} });
    };
    bond = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { bond: args }, fee || "auto", memo, funds);
    };
    unbond = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { unbond: {} }, fee || "auto", memo, funds);
    };
    tick = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { tick: {} }, fee || "auto", memo, funds);
    };
    addBondProvider = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { add_bond_provider: args }, fee || "auto", memo, funds);
    };
    removeBondProvider = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { remove_bond_provider: args }, fee || "auto", memo, funds);
    };
    updateConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
    };
    updateWithdrawnAmount = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_withdrawn_amount: args }, fee || "auto", memo, funds);
    };
    peripheralHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { peripheral_hook: args }, fee || "auto", memo, funds);
    };
    processEmergencyBatch = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { process_emergency_batch: args }, fee || "auto", memo, funds);
    };
    setPause = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { set_pause: args }, fee || "auto", memo, funds);
    };
    setBondHooks = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { set_bond_hooks: args }, fee || "auto", memo, funds);
    };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
    };
}
exports.Client = Client;
