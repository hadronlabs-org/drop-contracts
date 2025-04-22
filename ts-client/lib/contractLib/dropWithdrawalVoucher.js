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
    queryNftInfo = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { nft_info: args });
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
    queryMinter = async () => {
        return this.client.queryContractSmart(this.contractAddress, { minter: {} });
    };
    queryExtension = async (args) => {
        return this.client.queryContractSmart(this.contractAddress, { extension: args });
    };
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
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
    extension = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.extensionMsg(args), fee || "auto", memo, funds);
    };
    extensionMsg = (args) => { return { extension: args }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
