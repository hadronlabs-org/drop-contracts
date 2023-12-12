import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface InstantiateMsg {
  /**
   * This denom will be locked on contract's balance. Users are expected to send this denom with [`ExecuteMsg::Mint`] message in order to receive minted canonical funds.
   */
  bridged_denom: string;
  /**
   * This subdenom will form a canonical denom, minted by contract in exchange for bridged funds sent by users. Users are expected to send this denom with [`ExecuteMsg::Burn`] message in order to receive original bridged funds back.
   */
  canonical_subdenom: string;
}
export interface LidoSatelliteSchema {
  responses: ConfigResponse;
  execute: MintArgs | BurnArgs;
  [k: string]: unknown;
}
export interface ConfigResponse {
  bridged_denom: string;
  canonical_denom: string;
}
export interface MintArgs {
  /**
   * By default, canonical funds are minted to sender, but they can optionally be minted to any address specified in this field.
   */
  receiver?: string | null;
}
export interface BurnArgs {
  /**
   * By default, bridged funds are returned back to sender, but they can optionally be returned to any address specified in this field.
   */
  receiver?: string | null;
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
  queryConfig = async(): Promise<ConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  mint = async(sender:string, args: MintArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { mint: args }, fee || "auto", memo, funds);
  }
  burn = async(sender:string, args: BurnArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { burn: args }, fee || "auto", memo, funds);
  }
}
