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
        return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
    };
    updateMinterOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_minter_ownership: args }, fee || "auto", memo, funds);
    };
    updateCreatorOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_creator_ownership: args }, fee || "auto", memo, funds);
    };
    updateCollectionInfo = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_collection_info: args }, fee || "auto", memo, funds);
    };
    transferNft = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { transfer_nft: args }, fee || "auto", memo, funds);
    };
    sendNft = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { send_nft: args }, fee || "auto", memo, funds);
    };
    approve = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { approve: args }, fee || "auto", memo, funds);
    };
    revoke = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { revoke: args }, fee || "auto", memo, funds);
    };
    approveAll = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { approve_all: args }, fee || "auto", memo, funds);
    };
    revokeAll = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { revoke_all: args }, fee || "auto", memo, funds);
    };
    mint = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { mint: args }, fee || "auto", memo, funds);
    };
    burn = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { burn: args }, fee || "auto", memo, funds);
    };
    updateExtension = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_extension: args }, fee || "auto", memo, funds);
    };
    updateNftInfo = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { update_nft_info: args }, fee || "auto", memo, funds);
    };
    setWithdrawAddress = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { set_withdraw_address: args }, fee || "auto", memo, funds);
    };
    removeWithdrawAddress = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { remove_withdraw_address: {} }, fee || "auto", memo, funds);
    };
    withdrawFunds = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, { withdraw_funds: args }, fee || "auto", memo, funds);
    };
}
exports.Client = Client;
