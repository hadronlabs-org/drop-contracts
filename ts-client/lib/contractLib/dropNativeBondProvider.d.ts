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
export type Uint1281 = string;
export type Boolean = boolean;
export type Boolean1 = boolean;
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
export type Uint1282 = string;
export type ResponseHookMsg = {
    success: ResponseHookSuccessMsg;
} | {
    error: ResponseHookErrorMsg;
};
export type ResponseAnswer = {
    grant_delegate_response: MsgGrantResponse;
} | {
    delegate_response: MsgDelegateResponse;
} | {
    undelegate_response: MsgUndelegateResponse;
} | {
    begin_redelegate_response: MsgBeginRedelegateResponse;
} | {
    tokenize_shares_response: MsgTokenizeSharesResponse;
} | {
    redeem_tokensfor_shares_response: MsgRedeemTokensforSharesResponse;
} | {
    authz_exec_response: MsgExecResponse;
} | {
    i_b_c_transfer: MsgIBCTransfer;
} | {
    transfer_response: MsgSendResponse;
} | {
    unknown_response: {};
};
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
export type Transaction = {
    undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint1282][];
    };
} | {
    redelegate: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator_from: string;
        validator_to: string;
    };
} | {
    withdraw_reward: {
        interchain_account_id: string;
        validator: string;
    };
} | {
    tokenize_share: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator: string;
    };
} | {
    redeem_shares: {
        interchain_account_id: string;
        items: RedeemShareItem[];
    };
} | {
    claim_rewards_and_optionaly_transfer: {
        denom: string;
        interchain_account_id: string;
        transfer?: TransferReadyBatchesMsg | null;
        validators: string[];
    };
} | {
    i_b_c_transfer: {
        amount: number;
        denom: string;
        reason: IBCTransferReason;
        recipient: string;
    };
} | {
    stake: {
        items: [string, Uint1282][];
    };
} | {
    transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
    };
} | {
    setup_protocol: {
        interchain_account_id: string;
        rewards_withdraw_address: string;
    };
};
export type IBCTransferReason = "l_s_m_share" | "delegate";
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
export type Uint1283 = string;
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration = {
    at_height: number;
} | {
    at_time: Timestamp2;
} | {
    never: {};
};
/**
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp2 = Uint64;
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
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
export type TxStateStatus = "idle" | "in_progress" | "waiting_for_ack";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal1 = string;
export type PuppeteerHookArgs = {
    success: ResponseHookSuccessMsg;
} | {
    error: ResponseHookErrorMsg;
};
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
export interface DropNativeBondProviderSchema {
    responses: Uint128 | Uint1281 | Boolean | Boolean1 | Config | LastPuppeteerResponse | Uint1283 | OwnershipForString | Decimal | TxState;
    query: CanBondArgs | TokensAmountArgs;
    execute: UpdateConfigArgs | PuppeteerHookArgs | UpdateOwnershipArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface Config {
    base_denom: string;
    core_contract: Addr;
    min_ibc_transfer: Uint1282;
    min_stake_amount: Uint1282;
    port_id: string;
    puppeteer_contract: Addr;
    strategy_contract: Addr;
    timeout: number;
    transfer_channel_id: string;
}
export interface LastPuppeteerResponse {
    response?: ResponseHookMsg | null;
}
export interface ResponseHookSuccessMsg {
    answers: ResponseAnswer[];
    local_height: number;
    remote_height: number;
    request: RequestPacket;
    request_id: number;
    transaction: Transaction;
}
export interface MsgGrantResponse {
}
export interface MsgDelegateResponse {
}
export interface MsgUndelegateResponse {
    completion_time?: Timestamp | null;
}
export interface Timestamp {
    nanos: number;
    seconds: number;
}
export interface MsgBeginRedelegateResponse {
    completion_time?: Timestamp | null;
}
export interface MsgTokenizeSharesResponse {
    amount?: Coin | null;
}
export interface Coin {
    amount: Uint1282;
    denom: string;
    [k: string]: unknown;
}
export interface MsgRedeemTokensforSharesResponse {
    amount?: Coin | null;
}
export interface MsgExecResponse {
    results: number[][];
}
export interface MsgIBCTransfer {
}
export interface MsgSendResponse {
}
export interface RequestPacket {
    data?: Binary | null;
    destination_channel?: string | null;
    destination_port?: string | null;
    sequence?: number | null;
    source_channel?: string | null;
    source_port?: string | null;
    timeout_height?: RequestPacketTimeoutHeight | null;
    timeout_timestamp?: number | null;
    [k: string]: unknown;
}
export interface RequestPacketTimeoutHeight {
    revision_height?: number | null;
    revision_number?: number | null;
    [k: string]: unknown;
}
export interface RedeemShareItem {
    amount: Uint1282;
    local_denom: string;
    remote_denom: string;
}
export interface TransferReadyBatchesMsg {
    amount: Uint1282;
    batch_ids: number[];
    emergency: boolean;
    recipient: string;
}
export interface ResponseHookErrorMsg {
    details: string;
    request: RequestPacket;
    request_id: number;
    transaction: Transaction;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForString {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: string | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: string | null;
}
export interface TxState {
    status: TxStateStatus;
    transaction?: Transaction | null;
}
export interface CanBondArgs {
    denom: string;
}
export interface TokensAmountArgs {
    coin: Coin;
    exchange_rate: Decimal1;
}
export interface UpdateConfigArgs {
    new_config: ConfigOptional;
}
export interface ConfigOptional {
    base_denom?: string | null;
    core_contract?: Addr | null;
    min_ibc_transfer?: Uint1282 | null;
    min_stake_amount?: Uint1282 | null;
    port_id?: string | null;
    puppeteer_contract?: Addr | null;
    strategy_contract?: Addr | null;
    timeout?: number | null;
    transfer_channel_id?: string | null;
}
export interface InstantiateMsg {
    base_denom: string;
    core_contract: string;
    min_ibc_transfer: Uint1282;
    min_stake_amount: Uint1282;
    owner: string;
    port_id: string;
    puppeteer_contract: string;
    strategy_contract: string;
    timeout: number;
    transfer_channel_id: string;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryConfig: () => Promise<Config>;
    queryNonStakedBalance: () => Promise<Uint128>;
    queryAllBalance: () => Promise<Uint128>;
    queryTxState: () => Promise<TxState>;
    queryLastPuppeteerResponse: () => Promise<LastPuppeteerResponse>;
    queryCanBond: (args: CanBondArgs) => Promise<Boolean>;
    queryCanProcessOnIdle: () => Promise<Boolean>;
    queryTokensAmount: (args: TokensAmountArgs) => Promise<Decimal>;
    queryAsyncTokensAmount: () => Promise<Uint128>;
    queryOwnership: () => Promise<OwnershipForString>;
    updateConfig: (sender: string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    puppeteerHook: (sender: string, args: PuppeteerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    bond: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    processOnIdle: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateOwnership: (sender: string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
