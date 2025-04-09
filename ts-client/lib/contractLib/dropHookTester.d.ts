import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
export type Transaction = {
    undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
    };
} | {
    redelegate: {
        amount: Uint128;
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
        amount: Uint128;
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
        amount: Uint128;
        denom: string;
        real_amount: Uint128;
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
} | {
    enable_tokenize_shares: {};
} | {
    disable_tokenize_shares: {};
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
    local_height: number;
    remote_height: number;
    transaction: Transaction;
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
}
export interface ResponseHookErrorMsg {
    details: string;
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
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: Uint8Array, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    queryAnswers: () => Promise<ArrayOfResponseHookSuccessMsg>;
    queryErrors: () => Promise<ArrayOfResponseHookErrorMsg>;
    setConfig: (sender: string, args: SetConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    setConfigMsg: (args: SetConfigArgs) => {
        set_config: SetConfigArgs;
    };
    undelegate: (sender: string, args: UndelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    undelegateMsg: (args: UndelegateArgs) => {
        undelegate: UndelegateArgs;
    };
    redelegate: (sender: string, args: RedelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redelegateMsg: (args: RedelegateArgs) => {
        redelegate: RedelegateArgs;
    };
    tokenizeShare: (sender: string, args: TokenizeShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    tokenizeShareMsg: (args: TokenizeShareArgs) => {
        tokenize_share: TokenizeShareArgs;
    };
    redeemShare: (sender: string, args: RedeemShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    redeemShareMsg: (args: RedeemShareArgs) => {
        redeem_share: RedeemShareArgs;
    };
    puppeteerHook: (sender: string, args: PuppeteerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    puppeteerHookMsg: (args: PuppeteerHookArgs) => {
        puppeteer_hook: PuppeteerHookArgs;
    };
}
