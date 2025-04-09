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
    queryOwnerOf = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { owner_of: args });
    };
    queryApproval = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { approval: args });
    };
    queryApprovals = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { approvals: args });
    };
    queryOperator = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { operator: args });
    };
    queryAllOperators = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { all_operators: args });
    };
    queryNumTokens = async () => {
        return this.client.queryContractSmart(this.contractAddress, { num_tokens: {} });
    };
    queryContractInfo = async () => {
        return this.client.queryContractSmart(this.contractAddress, { contract_info: {} });
    };
    queryGetConfig = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_config: {} });
    };
    queryGetCollectionInfoAndExtension = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_collection_info_and_extension: {} });
    };
    queryGetAllInfo = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_all_info: {} });
    };
    queryGetCollectionExtensionAttributes = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_collection_extension_attributes: {} });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    queryMinter = async () => {
        return this.client.queryContractSmart(this.contractAddress, { minter: {} });
    };
    queryGetMinterOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_minter_ownership: {} });
    };
    queryGetCreatorOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_creator_ownership: {} });
    };
    queryNftInfo = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { nft_info: args });
    };
    queryGetNftByExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { get_nft_by_extension: args });
    };
    queryAllNftInfo = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { all_nft_info: args });
    };
    queryTokens = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { tokens: args });
    };
    queryAllTokens = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { all_tokens: args });
    };
    queryExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { extension: args });
    };
    queryGetCollectionExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { get_collection_extension: args });
    };
    queryGetWithdrawAddress = async () => {
        return this.client.queryContractSmart(this.contractAddress, { get_withdraw_address: {} });
    };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
    updateMinterOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateMinterOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateMinterOwnershipMsg = (args) => { return { update_minter_ownership: args }; };
    updateCreatorOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateCreatorOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateCreatorOwnershipMsg = (args) => { return { update_creator_ownership: args }; };
    updateCollectionInfo = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateCollectionInfoMsg(args), fee || "auto", memo, funds);
    };
    updateCollectionInfoMsg = (args) => { return { update_collection_info: args }; };
    transferNft = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.transferNftMsg(args), fee || "auto", memo, funds);
    };
    transferNftMsg = (args) => { return { transfer_nft: args }; };
    sendNft = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.sendNftMsg(args), fee || "auto", memo, funds);
    };
    sendNftMsg = (args) => { return { send_nft: args }; };
    approve = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.approveMsg(args), fee || "auto", memo, funds);
    };
    approveMsg = (args) => { return { approve: args }; };
    revoke = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.revokeMsg(args), fee || "auto", memo, funds);
    };
    revokeMsg = (args) => { return { revoke: args }; };
    approveAll = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.approveAllMsg(args), fee || "auto", memo, funds);
    };
    approveAllMsg = (args) => { return { approve_all: args }; };
    revokeAll = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.revokeAllMsg(args), fee || "auto", memo, funds);
    };
    revokeAllMsg = (args) => { return { revoke_all: args }; };
    mint = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.mintMsg(args), fee || "auto", memo, funds);
    };
    mintMsg = (args) => { return { mint: args }; };
    burn = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.burnMsg(args), fee || "auto", memo, funds);
    };
    burnMsg = (args) => { return { burn: args }; };
    updateExtension = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateExtensionMsg(args), fee || "auto", memo, funds);
    };
    updateExtensionMsg = (args) => { return { update_extension: args }; };
    updateNftInfo = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateNftInfoMsg(args), fee || "auto", memo, funds);
    };
    updateNftInfoMsg = (args) => { return { update_nft_info: args }; };
    setWithdrawAddress = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setWithdrawAddressMsg(args), fee || "auto", memo, funds);
    };
    setWithdrawAddressMsg = (args) => { return { set_withdraw_address: args }; };
    removeWithdrawAddress = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.removeWithdrawAddressMsg(), fee || "auto", memo, funds);
    };
    removeWithdrawAddressMsg = () => { return { remove_withdraw_address: {} }; };
    withdrawFunds = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.withdrawFundsMsg(args), fee || "auto", memo, funds);
    };
    withdrawFundsMsg = (args) => { return { withdraw_funds: args }; };
}
exports.Client = Client;
