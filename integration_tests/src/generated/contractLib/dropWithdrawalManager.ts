import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface InstantiateMsg {
  base_denom: string;
  core_contract: string;
  owner: string;
  voucher_contract: string;
}
export interface DropWithdrawalManagerSchema {
  responses: Config;
  execute: UpdateConfigArgs | ReceiveNftArgs;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  core_contract: string;
  owner: string;
  withdrawal_voucher_contract: string;
}
export interface UpdateConfigArgs {
  core_contract?: string | null;
  owner?: string | null;
  voucher_contract?: string | null;
}
/**
 * Cw721ReceiveMsg should be de/serialized under `Receive()` variant in a ExecuteMsg
 */
export interface ReceiveNftArgs {
  description?: "Cw721ReceiveMsg should be de/serialized under `Receive()` variant in a ExecuteMsg";
  type?: "object";
  required?: ["msg", "sender", "token_id"];
  properties?: {
    [k: string]: unknown;
  };
  additionalProperties?: false;
}


function isSigningCosmWasmClient(
  client: CosmWasmClient | SigningCosmWasmClient
): client is SigningCosmWasmClient {
  return 'execute' in client;
}

export class Client {
  private readonly client: CosmWasmClient | SigningCosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
  }
  mustBeSigningClient() {
    return new Error("This client is not a SigningCosmWasmClient");
  }
  static async instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    initCoins?: readonly Coin[],
    fees?: StdFee | 'auto' | number,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  receiveNft = async(sender:string, args: ReceiveNftArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { receive_nft: args }, fee || "auto", memo, funds);
  }
}
