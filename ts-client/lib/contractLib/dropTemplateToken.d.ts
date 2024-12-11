import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export type String = string;
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint128 = string;
export interface DropTemplateTokenSchema {
    responses: String;
    execute: MintArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface MintArgs {
    amount: Uint128;
}
export interface InstantiateMsg {
    exponent: number;
    subdenom: string;
    token_metadata: DenomMetadata;
}
/**
 * Replicates the cosmos-sdk bank module Metadata type
 */
export interface DenomMetadata {
    base: string;
    denom_units: DenomUnit[];
    description: string;
    display: string;
    name: string;
    symbol: string;
    uri: string;
    uri_hash: string;
    [k: string]: unknown;
}
/**
 * Replicates the cosmos-sdk bank module DenomUnit type
 */
export interface DenomUnit {
    aliases: string[];
    denom: string;
    exponent: number;
    [k: string]: unknown;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryDenom: () => Promise<String>;
    mint: (sender: string, args: MintArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    burn: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}