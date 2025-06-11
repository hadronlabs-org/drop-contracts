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
    queryTransactions = async () => {
        return this.client.queryContractSmart(this.contractAddress, { transactions: {} });
    };
    queryExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { extension: args });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
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
    registerBalanceAndDelegatorDelegationsQuery = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.registerBalanceAndDelegatorDelegationsQueryMsg(args), fee || "auto", memo, funds);
    };
    registerBalanceAndDelegatorDelegationsQueryMsg = (args) => { return { register_balance_and_delegator_delegations_query: args }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
