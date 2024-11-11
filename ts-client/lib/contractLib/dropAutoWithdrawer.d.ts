import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
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
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;
export type BondArgs = {
    with_ld_assets: {};
} | {
    with_withdrawal_denoms: {
        batch_id: Uint128;
    };
};
/**
 * A human readable address.
 *
 * In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.
 *
 * This type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.
 *
 * This type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.
 */
export type Addr = string;
export interface DropAutoWithdrawerSchema {
    responses: BondingsResponse | InstantiateMsg;
    query: BondingsArgs;
    execute: BondArgs | UnbondArgs | WithdrawArgs;
    instantiate?: InstantiateMsg1;
    [k: string]: unknown;
}
export interface BondingsResponse {
    bondings: BondingResponse[];
    next_page_key?: string | null;
}
export interface BondingResponse {
    bonder: string;
    bonding_id: string;
    deposit: Coin[];
    withdrawal_amount: Uint128;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export interface InstantiateMsg {
    core_address: string;
    ld_token: string;
    withdrawal_denom_prefix: string;
    withdrawal_manager_address: string;
    withdrawal_token_address: string;
}
export interface BondingsArgs {
    /**
     * Pagination limit. Default is 100
     */
    limit?: Uint64 | null;
    /**
     * Pagination offset
     */
    page_key?: string | null;
    /**
     * Optionally filter bondings by user address
     */
    user?: string | null;
}
export interface UnbondArgs {
    batch_id: Uint128;
}
export interface WithdrawArgs {
    batch_id: Uint128;
    receiver?: Addr | null;
}
export interface InstantiateMsg1 {
    core_address: string;
    ld_token: string;
    withdrawal_denom_prefix: string;
    withdrawal_manager_address: string;
    withdrawal_token_address: string;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryBondings: (args: BondingsArgs) => Promise<BondingsResponse>;
    queryConfig: () => Promise<InstantiateMsg>;
    bond: (sender: string, args: BondArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    unbond: (sender: string, args: UnbondArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    withdraw: (sender: string, args: WithdrawArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
