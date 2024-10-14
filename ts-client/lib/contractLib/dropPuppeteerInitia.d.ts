import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
export type IcaState = ("none" | "in_progress" | "timeout") | {
    registered: {
        channel_id: string;
        ica_address: string;
        port_id: string;
    };
};
export type ArrayOfTupleOfUint64AndString = [number, string][];
export type Transaction = {
    undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
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
        real_amount: number;
        reason: IBCTransferReason;
        recipient: string;
    };
} | {
    stake: {
        amount: Uint128;
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
export type IBCTransferReason = "l_s_m_share" | "delegate";
export type ArrayOfTransaction = Transaction[];
export type TxStateStatus = "idle" | "in_progress" | "waiting_for_ack";
export type QueryExtMsg = {
    delegations: {};
} | {
    balances: {};
} | {
    non_native_rewards_balances: {};
} | {
    unbonding_delegations: {};
} | {
    ownership: {};
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
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration = {
    at_height: number;
} | {
    at_time: Timestamp;
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
export type Timestamp = Uint64;
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
export interface DropPuppeteerInitiaSchema {
    responses: ConfigResponse | Binary | IcaState | ArrayOfTupleOfUint64AndString | ArrayOfTransaction | TxState;
    query: ExtensionArgs;
    execute: RegisterBalanceAndDelegatorDelegationsQueryArgs | RegisterDelegatorUnbondingDelegationsQueryArgs | RegisterNonNativeRewardsBalancesQueryArgs | SetupProtocolArgs | DelegateArgs | UndelegateArgs | RedelegateArgs | TokenizeShareArgs | RedeemSharesArgs | TransferArgs | ClaimRewardsAndOptionalyTransferArgs | UpdateConfigArgs | UpdateOwnershipArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface ConfigResponse {
    connection_id: string;
    update_period: number;
}
export interface RedeemShareItem {
    amount: Uint128;
    local_denom: string;
    remote_denom: string;
}
export interface TransferReadyBatchesMsg {
    amount: Uint128;
    batch_ids: number[];
    emergency: boolean;
    recipient: string;
}
export interface Coin {
    amount: Uint128;
    denom: string;
    [k: string]: unknown;
}
export interface TxState {
    reply_to?: string | null;
    seq_id?: number | null;
    status: TxStateStatus;
    transaction?: Transaction | null;
}
export interface ExtensionArgs {
    msg: QueryExtMsg;
}
export interface RegisterBalanceAndDelegatorDelegationsQueryArgs {
    validators: string[];
}
export interface RegisterDelegatorUnbondingDelegationsQueryArgs {
    validators: string[];
}
export interface RegisterNonNativeRewardsBalancesQueryArgs {
    denoms: string[];
}
export interface SetupProtocolArgs {
    rewards_withdraw_address: string;
}
export interface DelegateArgs {
    items: [string, Uint128][];
    reply_to: string;
}
export interface UndelegateArgs {
    batch_id: number;
    items: [string, Uint128][];
    reply_to: string;
}
export interface RedelegateArgs {
    amount: Uint128;
    reply_to: string;
    validator_from: string;
    validator_to: string;
}
export interface TokenizeShareArgs {
    amount: Uint128;
    reply_to: string;
    validator: string;
}
export interface RedeemSharesArgs {
    items: RedeemShareItem[];
    reply_to: string;
}
export interface TransferArgs {
    items: [string, Coin][];
    reply_to: string;
}
export interface ClaimRewardsAndOptionalyTransferArgs {
    reply_to: string;
    transfer?: TransferReadyBatchesMsg | null;
    validators: string[];
}
export interface UpdateConfigArgs {
    new_config: ConfigOptional;
}
export interface ConfigOptional {
    allowed_senders?: string[] | null;
    connection_id?: string | null;
    native_bond_provider?: Addr | null;
    port_id?: string | null;
    remote_denom?: string | null;
    sdk_version?: string | null;
    timeout?: number | null;
    transfer_channel_id?: string | null;
    update_period?: number | null;
}
export interface InstantiateMsg {
    allowed_senders: string[];
    connection_id: string;
    delegations_queries_chunk_size?: number | null;
    native_bond_provider: string;
    owner?: string | null;
    port_id: string;
    remote_denom: string;
    sdk_version: string;
    timeout: number;
    transfer_channel_id: string;
    update_period: number;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryConfig: () => Promise<ConfigResponse>;
    queryIca: () => Promise<IcaState>;
    queryTransactions: () => Promise<ArrayOfTransaction>;
    queryKVQueryIds: () => Promise<ArrayOfTupleOfUint64AndString>;
    queryExtension: (args: ExtensionArgs) => Promise<Binary>;
    queryTxState: () => Promise<TxState>;
    registerICA: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    registerQuery: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    registerBalanceAndDelegatorDelegationsQuery: (sender: string, args: RegisterBalanceAndDelegatorDelegationsQueryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    registerDelegatorUnbondingDelegationsQuery: (sender: string, args: RegisterDelegatorUnbondingDelegationsQueryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    registerNonNativeRewardsBalancesQuery: (sender: string, args: RegisterNonNativeRewardsBalancesQueryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    setupProtocol: (sender: string, args: SetupProtocolArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    delegate: (sender: string, args: DelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    undelegate: (sender: string, args: UndelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redelegate: (sender: string, args: RedelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    tokenizeShare: (sender: string, args: TokenizeShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redeemShares: (sender: string, args: RedeemSharesArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    transfer: (sender: string, args: TransferArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    claimRewardsAndOptionalyTransfer: (sender: string, args: ClaimRewardsAndOptionalyTransferArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateConfig: (sender: string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateOwnership: (sender: string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
