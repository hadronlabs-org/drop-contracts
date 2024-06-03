import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface DropProposalVotesPocSchema {
  responses: Config | Metrics;
  execute: UpdateConfigArgs | UpdateActiveProposalsArgs | UpdateVotersListArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  connection_id: string;
  core_address: string;
  port_id: string;
  provider_proposals_address: string;
  update_period: number;
}
export interface Metrics {
  total_voters: number;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  connection_id?: string | null;
  core_address?: string | null;
  port_id?: string | null;
  provider_proposals_address?: string | null;
  update_period?: number | null;
}
export interface UpdateActiveProposalsArgs {
  active_proposals: number[];
}
export interface UpdateVotersListArgs {
  voters: string[];
}
export interface InstantiateMsg {
  connection_id: string;
  core_address: string;
  port_id: string;
  provider_proposals_address: string;
  update_period: number;
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
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryMetrics = async(): Promise<Metrics> => {
    return this.client.queryContractSmart(this.contractAddress, { metrics: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  updateActiveProposals = async(sender:string, args: UpdateActiveProposalsArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_active_proposals: args }, fee || "auto", memo, funds);
  }
  updateVotersList = async(sender:string, args: UpdateVotersListArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_voters_list: args }, fee || "auto", memo, funds);
  }
}
