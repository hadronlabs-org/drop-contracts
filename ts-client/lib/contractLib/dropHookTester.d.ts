import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
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
        items: [string, Uint128][];
    };
} | {
    transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
    };
} | {
    setup_protocol: {
        delegate_grantee: string;
        interchain_account_id: string;
        rewards_withdraw_address: string;
    };
};
export type IBCTransferReason = "l_s_m_share" | "delegate";
export type ArrayOfResponseHookSuccessMsg = ResponseHookSuccessMsg[];
export type ArrayOfResponseHookErrorMsg = ResponseHookErrorMsg[];
export type PuppeteerHookArgs = {
    success: ResponseHookSuccessMsg;
} | {
    error: ResponseHookErrorMsg;
};
export interface DropHookTesterSchema {
    responses: ArrayOfResponseHookSuccessMsg | ArrayOfResponseHookErrorMsg;
    execute: SetConfigArgs | UndelegateArgs | RedelegateArgs | TokenizeShareArgs | RedeemShareArgs | PuppeteerHookArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
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
    amount: Uint128;
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
export interface ResponseHookErrorMsg {
    details: string;
    request: RequestPacket;
    request_id: number;
    transaction: Transaction;
}
export interface SetConfigArgs {
    puppeteer_addr: string;
}
export interface UndelegateArgs {
    amount: Uint128;
    validator: string;
}
export interface RedelegateArgs {
    amount: Uint128;
    validator_from: string;
    validator_to: string;
}
export interface TokenizeShareArgs {
    amount: Uint128;
    validator: string;
}
export interface RedeemShareArgs {
    amount: Uint128;
    denom: string;
    validator: string;
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
    queryAnswers: () => Promise<ArrayOfResponseHookSuccessMsg>;
    queryErrors: () => Promise<ArrayOfResponseHookErrorMsg>;
    setConfig: (sender: string, args: SetConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    undelegate: (sender: string, args: UndelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redelegate: (sender: string, args: RedelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    tokenizeShare: (sender: string, args: TokenizeShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redeemShare: (sender: string, args: RedeemShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    puppeteerHook: (sender: string, args: PuppeteerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
