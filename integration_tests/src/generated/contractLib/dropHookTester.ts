import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
export type ResponseAnswer =
  | {
      grant_delegate_response: MsgGrantResponse;
    }
  | {
      delegate_response: MsgDelegateResponse;
    }
  | {
      undelegate_response: MsgUndelegateResponse;
    }
  | {
      begin_redelegate_response: MsgBeginRedelegateResponse;
    }
  | {
      tokenize_shares_response: MsgTokenizeSharesResponse;
    }
  | {
      redeem_tokensfor_shares_response: MsgRedeemTokensforSharesResponse;
    }
  | {
      authz_exec_response: MsgExecResponse;
    }
  | {
      i_b_c_transfer: MsgIBCTransfer;
    }
  | {
      transfer_response: MsgSendResponse;
    }
  | {
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
export type Transaction =
  | {
      delegate: {
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
      };
    }
  | {
      undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
      };
    }
  | {
      redelegate: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator_from: string;
        validator_to: string;
      };
    }
  | {
      withdraw_reward: {
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      tokenize_share: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      redeem_shares: {
        interchain_account_id: string;
        items: RedeemShareItem[];
      };
    }
  | {
      claim_rewards_and_optionaly_transfer: {
        denom: string;
        interchain_account_id: string;
        transfer?: TransferReadyBatchesMsg | null;
        validators: string[];
      };
    }
  | {
      i_b_c_transfer: {
        amount: number;
        denom: string;
        reason: IBCTransferReason;
        recipient: string;
      };
    }
  | {
      transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
      };
    }
  | {
      grant_delegate: {
        grantee: string;
        interchain_account_id: string;
      };
    };
export type IBCTransferReason = "l_s_m_share" | "stake";
export type ArrayOfResponseHookSuccessMsg = ResponseHookSuccessMsg[];
export type ArrayOfResponseHookErrorMsg = ResponseHookErrorMsg[];
export type PuppeteerHookArgs =
  | {
      success: ResponseHookSuccessMsg;
    }
  | {
      error: ResponseHookErrorMsg;
    };

export interface DropHookTesterSchema {
  responses: ArrayOfResponseHookSuccessMsg | ArrayOfResponseHookErrorMsg;
  execute:
    | SetConfigArgs
    | DelegateArgs
    | UndelegateArgs
    | RedelegateArgs
    | TokenizeShareArgs
    | RedeemShareArgs
    | PuppeteerHookArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface ResponseHookSuccessMsg {
  answers: ResponseAnswer[];
  local_height: number;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction;
}
export interface MsgGrantResponse {}
export interface MsgDelegateResponse {}
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
export interface MsgIBCTransfer {}
export interface MsgSendResponse {}
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
export interface DelegateArgs {
  amount: Uint128;
  timeout?: number | null;
  validator: string;
}
export interface UndelegateArgs {
  amount: Uint128;
  timeout?: number | null;
  validator: string;
}
export interface RedelegateArgs {
  amount: Uint128;
  timeout?: number | null;
  validator_from: string;
  validator_to: string;
}
export interface TokenizeShareArgs {
  amount: Uint128;
  timeout?: number | null;
  validator: string;
}
export interface RedeemShareArgs {
  amount: Uint128;
  denom: string;
  timeout?: number | null;
  validator: string;
}
export interface InstantiateMsg {}


function isSigningCosmWasmClient(
  client: CosmWasmClient | SigningCosmWasmClient
): client is SigningCosmWasmClient {
  return 'execute' in client;
}

export class Client {
  private readonly client: CosmWasmClient | SigningCosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
  }
  mustBeSigningClient() {
    return new Error("This client is not a SigningCosmWasmClient");
  }
  static async instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryAnswers = async(): Promise<ArrayOfResponseHookSuccessMsg> => {
    return this.client.queryContractSmart(this.contractAddress, { answers: {} });
  }
  queryErrors = async(): Promise<ArrayOfResponseHookErrorMsg> => {
    return this.client.queryContractSmart(this.contractAddress, { errors: {} });
  }
  setConfig = async(sender:string, args: SetConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { set_config: args }, fee || "auto", memo, funds);
  }
  delegate = async(sender:string, args: DelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { delegate: args }, fee || "auto", memo, funds);
  }
  undelegate = async(sender:string, args: UndelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { undelegate: args }, fee || "auto", memo, funds);
  }
  redelegate = async(sender:string, args: RedelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { redelegate: args }, fee || "auto", memo, funds);
  }
  tokenizeShare = async(sender:string, args: TokenizeShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { tokenize_share: args }, fee || "auto", memo, funds);
  }
  redeemShare = async(sender:string, args: RedeemShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { redeem_share: args }, fee || "auto", memo, funds);
  }
  puppeteerHook = async(sender:string, args: PuppeteerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { puppeteer_hook: args }, fee || "auto", memo, funds);
  }
}
