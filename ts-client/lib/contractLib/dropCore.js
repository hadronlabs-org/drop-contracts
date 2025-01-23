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
    queryConfig = async () => {
        return this.client.queryContractSmart(this.contractAddress, { config: {} });
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
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    bond = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.bondMsg(args), fee || "auto", memo, funds);
    };
    bondMsg = (args) => { return { bond: args }; };
    unbond = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.unbondMsg(), fee || "auto", memo, funds);
    };
    unbondMsg = () => { return { unbond: {} }; };
    tick = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.tickMsg(), fee || "auto", memo, funds);
    };
    tickMsg = () => { return { tick: {} }; };
    addBondProvider = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.addBondProviderMsg(args), fee || "auto", memo, funds);
    };
    addBondProviderMsg = (args) => { return { add_bond_provider: args }; };
    removeBondProvider = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.removeBondProviderMsg(args), fee || "auto", memo, funds);
    };
    removeBondProviderMsg = (args) => { return { remove_bond_provider: args }; };
    updateConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateConfigMsg(args), fee || "auto", memo, funds);
    };
    updateConfigMsg = (args) => { return { update_config: args }; };
    updateWithdrawnAmount = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateWithdrawnAmountMsg(args), fee || "auto", memo, funds);
    };
    updateWithdrawnAmountMsg = (args) => { return { update_withdrawn_amount: args }; };
    peripheralHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.peripheralHookMsg(args), fee || "auto", memo, funds);
    };
    peripheralHookMsg = (args) => { return { peripheral_hook: args }; };
    resetBondedAmount = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.resetBondedAmountMsg(), fee || "auto", memo, funds);
    };
    resetBondedAmountMsg = () => { return { reset_bonded_amount: {} }; };
    processEmergencyBatch = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.processEmergencyBatchMsg(args), fee || "auto", memo, funds);
    };
    processEmergencyBatchMsg = (args) => { return { process_emergency_batch: args }; };
    setPause = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setPauseMsg(args), fee || "auto", memo, funds);
    };
    setPauseMsg = (args) => { return { set_pause: args }; };
    setBondHooks = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setBondHooksMsg(args), fee || "auto", memo, funds);
    };
    setBondHooksMsg = (args) => { return { set_bond_hooks: args }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
