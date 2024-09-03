import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
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
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
export type ArrayOfValidatorState = ValidatorState[];
export interface DropValidatorsStatsSchema {
    responses: Config | KvQueryIds | ArrayOfValidatorState;
    execute: RegisterStatsQueriesArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface Config {
    avg_block_time: number;
    connection_id: string;
    info_update_period: number;
    owner: Addr;
    port_id: string;
    profile_update_period: number;
}
export interface KvQueryIds {
    signing_info_id?: string | null;
    validator_profile_id?: string | null;
}
export interface ValidatorState {
    jailed_number?: number | null;
    last_commission_in_range?: number | null;
    last_processed_local_height?: number | null;
    last_processed_remote_height?: number | null;
    last_validated_height?: number | null;
    prev_jailed_state: boolean;
    tombstone: boolean;
    uptime: Decimal;
    valcons_address: string;
    valoper_address: string;
}
export interface RegisterStatsQueriesArgs {
    validators: string[];
}
export interface InstantiateMsg {
    avg_block_time: number;
    connection_id: string;
    info_update_period: number;
    owner: string;
    port_id: string;
    profile_update_period: number;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryConfig: () => Promise<Config>;
    queryKVQueryIds: () => Promise<KvQueryIds>;
    queryState: () => Promise<ArrayOfValidatorState>;
    registerStatsQueries: (sender: string, args: RegisterStatsQueriesArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    registerStatsQueriesMsg: (args: RegisterStatsQueriesArgs) => {
        register_stats_queries: RegisterStatsQueriesArgs;
    };
}
