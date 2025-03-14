import { CosmWasmClient, SigningCosmWasmClient, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
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
export type ArrayOfTupleOfStringAndUint128 = [string, Uint128][];
export type ArrayOfTupleOfStringAndUint1281 = [string, Uint128][];
export interface DropDistributionSchema {
    responses: ArrayOfTupleOfStringAndUint128 | ArrayOfTupleOfStringAndUint1281;
    query: CalcDepositArgs | CalcWithdrawArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface CalcDepositArgs {
    delegations: Delegations;
    deposit: Uint128;
}
export interface Delegations {
    delegations: Delegation[];
    total_on_top: Uint128;
    total_stake: Uint128;
    total_weight: number;
}
export interface Delegation {
    on_top: Uint128;
    stake: Uint128;
    valoper_address: string;
    weight: number;
}
export interface CalcWithdrawArgs {
    delegations: Delegations;
    withdraw: Uint128;
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
    queryCalcDeposit: (args: CalcDepositArgs) => Promise<ArrayOfTupleOfStringAndUint128>;
    queryCalcWithdraw: (args: CalcWithdrawArgs) => Promise<ArrayOfTupleOfStringAndUint128>;
}
