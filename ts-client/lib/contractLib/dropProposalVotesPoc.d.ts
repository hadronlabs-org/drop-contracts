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
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    queryConfig: () => Promise<Config>;
    queryMetrics: () => Promise<Metrics>;
    updateConfig: (sender: string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateConfigMsg: (args: UpdateConfigArgs) => {
        update_config: UpdateConfigArgs;
    };
    updateActiveProposals: (sender: string, args: UpdateActiveProposalsArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateActiveProposalsMsg: (args: UpdateActiveProposalsArgs) => {
        update_active_proposals: UpdateActiveProposalsArgs;
    };
    updateVotersList: (sender: string, args: UpdateVotersListArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateVotersListMsg: (args: UpdateVotersListArgs) => {
        update_voters_list: UpdateVotersListArgs;
    };
}
