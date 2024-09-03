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
    queryOwnership = async () => {
        return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
    };
    mint = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.mintMsg(args), fee || "auto", memo, funds);
    };
    mintMsg = (args) => { return { mint: args }; };
    burn = async (sender, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.burnMsg(), fee || "auto", memo, funds);
    };
    burnMsg = () => { return { burn: {} }; };
    setTokenMetadata = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.setTokenMetadataMsg(args), fee || "auto", memo, funds);
    };
    setTokenMetadataMsg = (args) => { return { set_token_metadata: args }; };
    updateOwnership = async (sender, args, fee, memo, funds) => {
        if (!isSigningCosmWasmClient(this.client)) {
            throw this.mustBeSigningClient();
        }
        return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
    };
    updateOwnershipMsg = (args) => { return { update_ownership: args }; };
}
exports.Client = Client;
