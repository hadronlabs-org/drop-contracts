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
    queryLastStakerResponse = async () => {
        return this.client.queryContractSmart(this.contractAddress, { last_staker_response: {} });
    };
    queryPendingLSMShares = async () => {
        return this.client.queryContractSmart(this.contractAddress, { pending_l_s_m_shares: {} });
    };
    queryLSMSharesToRedeem = async () => {
        return this.client.queryContractSmart(this.contractAddress, { l_s_m_shares_to_redeem: {} });
    };
    queryTotalBonded = async () => {
        return this.client.queryContractSmart(this.contractAddress, { total_bonded: {} });
    };
    queryTotalLSMShares = async () => {
        return this.client.queryContractSmart(this.contractAddress, { total_l_s_m_shares: {} });
    };
    queryFailedBatch = async () => {
        return this.client.queryContractSmart(this.contractAddress, { failed_batch: {} });
    };
    queryPauseInfo = async () => {
        return this.client.queryContractSmart(this.contractAddress, { pause_info: {} });
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
    tick = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.tickMsg(), fee || "auto", memo, funds);
    };
    tickMsg = () => { return { tick: {} }; };
    puppeteerHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.puppeteerHookMsg(args), fee || "auto", memo, funds);
    };
    puppeteerHookMsg = (args) => { return { puppeteer_hook: args }; };
    stakerHook = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.stakerHookMsg(args), fee || "auto", memo, funds);
    };
    stakerHookMsg = (args) => { return { staker_hook: args }; };
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
    pause = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.pauseMsg(), fee || "auto", memo, funds);
    };
    pauseMsg = () => { return { pause: {} }; };
    unpause = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.unpauseMsg(), fee || "auto", memo, funds);
    };
    unpauseMsg = () => { return { unpause: {} }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
