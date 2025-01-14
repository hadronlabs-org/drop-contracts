import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface DropTemplateFactorySchema {
    responses: State;
    execute: UpdateStateArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface State {
    core_contract: string;
    distribution_contract: string;
    lsm_share_bond_provider_contract: string;
    native_bond_provider_contract: string;
    puppeteer_contract: string;
    rewards_manager_contract: string;
    rewards_pump_contract: string;
    splitter_contract: string;
    strategy_contract: string;
    token_contract: string;
    validators_set_contract: string;
    withdrawal_manager_contract: string;
    withdrawal_voucher_contract: string;
}
export interface UpdateStateArgs {
    state: State1;
}
export interface State1 {
    core_contract: string;
    distribution_contract: string;
    lsm_share_bond_provider_contract: string;
    native_bond_provider_contract: string;
    puppeteer_contract: string;
    rewards_manager_contract: string;
    rewards_pump_contract: string;
    splitter_contract: string;
    strategy_contract: string;
    token_contract: string;
    validators_set_contract: string;
    withdrawal_manager_contract: string;
    withdrawal_voucher_contract: string;
}
export interface InstantiateMsg {
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryState: () => Promise<State>;
    updateState: (sender: string, args: UpdateStateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
