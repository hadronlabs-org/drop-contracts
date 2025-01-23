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
    queryIca = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ica: {} });
    };
    queryTransactions = async () => {
        return this.client.queryContractSmart(this.contractAddress, { transactions: {} });
    };
    queryKVQueryIds = async () => {
        return this.client.queryContractSmart(this.contractAddress, { k_v_query_ids: {} });
    };
    queryExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { extension: args });
    };
    queryTxState = async () => {
        return this.client.queryContractSmart(this.contractAddress, { tx_state: {} });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    registerICA = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerICAMsg(), fee || "auto", memo, funds);
    };
    registerICAMsg = () => { return { register_i_c_a: {} }; };
    registerQuery = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerQueryMsg(), fee || "auto", memo, funds);
    };
    registerQueryMsg = () => { return { register_query: {} }; };
    registerBalanceAndDelegatorDelegationsQuery = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerBalanceAndDelegatorDelegationsQueryMsg(args), fee || "auto", memo, funds);
    };
    registerBalanceAndDelegatorDelegationsQueryMsg = (args) => { return { register_balance_and_delegator_delegations_query: args }; };
    registerDelegatorUnbondingDelegationsQuery = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerDelegatorUnbondingDelegationsQueryMsg(args), fee || "auto", memo, funds);
    };
    registerDelegatorUnbondingDelegationsQueryMsg = (args) => { return { register_delegator_unbonding_delegations_query: args }; };
    registerNonNativeRewardsBalancesQuery = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerNonNativeRewardsBalancesQueryMsg(args), fee || "auto", memo, funds);
    };
    registerNonNativeRewardsBalancesQueryMsg = (args) => { return { register_non_native_rewards_balances_query: args }; };
    setupProtocol = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setupProtocolMsg(args), fee || "auto", memo, funds);
    };
    setupProtocolMsg = (args) => { return { setup_protocol: args }; };
    delegate = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.delegateMsg(args), fee || "auto", memo, funds);
    };
    delegateMsg = (args) => { return { delegate: args }; };
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
    redeemShares = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.redeemSharesMsg(args), fee || "auto", memo, funds);
    };
    redeemSharesMsg = (args) => { return { redeem_shares: args }; };
    transfer = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.transferMsg(args), fee || "auto", memo, funds);
    };
    transferMsg = (args) => { return { transfer: args }; };
    claimRewardsAndOptionalyTransfer = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.claimRewardsAndOptionalyTransferMsg(args), fee || "auto", memo, funds);
    };
    claimRewardsAndOptionalyTransferMsg = (args) => { return { claim_rewards_and_optionaly_transfer: args }; };
    updateConfig = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateConfigMsg(args), fee || "auto", memo, funds);
    };
    updateConfigMsg = (args) => { return { update_config: args }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
