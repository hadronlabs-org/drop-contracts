import {
  CosmWasmClient,
  SigningCosmWasmClient,
  ExecuteResult,
  InstantiateResult,
} from '@cosmjs/cosmwasm-stargate';
import { StdFee } from '@cosmjs/amino';
export interface InstantiateMsg {
  core_address: string;
  ld_token: string;
  withdrawal_manager_address: string;
  withdrawal_voucher_address: string;
}
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
export type BondArgs =
  | {
      with_ld_assets: {};
    }
  | {
      with_n_f_t: {
        token_id: string;
      };
    };

export interface DropAutoWithdrawerSchema {
  responses: BondingsResponse | InstantiateMsg;
  query: BondingsArgs;
  execute: BondArgs | UnbondArgs | WithdrawArgs;
  [k: string]: unknown;
}
export interface BondingsResponse {
  bondings: BondingResponse[];
  next_page_key?: string | null;
}
export interface BondingResponse {
  bonder: string;
  deposit: Coin[];
  token_id: string;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface InstantiateMsg {
  core_address: string;
  ld_token: string;
  withdrawal_manager_address: string;
  withdrawal_voucher_address: string;
}
export interface BondingsArgs {
  /**
   * Pagination limit. Default is 100
   */
  limit?: number | null;
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
  token_id: string;
}
export interface WithdrawArgs {
  token_id: string;
}

function isSigningCosmWasmClient(
  client: CosmWasmClient | SigningCosmWasmClient,
): client is SigningCosmWasmClient {
  return 'execute' in client;
}

export class Client {
  private readonly client: CosmWasmClient | SigningCosmWasmClient;
  contractAddress: string;
  constructor(
    client: CosmWasmClient | SigningCosmWasmClient,
    contractAddress: string,
  ) {
    this.client = client;
    this.contractAddress = contractAddress;
  }
  mustBeSigningClient() {
    return new Error('This client is not a SigningCosmWasmClient');
  }
  static async instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    initCoins?: readonly Coin[],
    fees?: StdFee | 'auto' | number,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryBondings = async (args: BondingsArgs): Promise<BondingsResponse> =>
    this.client.queryContractSmart(this.contractAddress, { bondings: args });
  queryConfig = async (): Promise<InstantiateMsg> =>
    this.client.queryContractSmart(this.contractAddress, { config: {} });
  bond = async (
    sender: string,
    args: BondArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    if (!isSigningCosmWasmClient(this.client)) {
      throw this.mustBeSigningClient();
    }
    return this.client.execute(
      sender,
      this.contractAddress,
      { bond: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  unbond = async (
    sender: string,
    args: UnbondArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    if (!isSigningCosmWasmClient(this.client)) {
      throw this.mustBeSigningClient();
    }
    return this.client.execute(
      sender,
      this.contractAddress,
      { unbond: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  withdraw = async (
    sender: string,
    args: WithdrawArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ): Promise<ExecuteResult> => {
    if (!isSigningCosmWasmClient(this.client)) {
      throw this.mustBeSigningClient();
    }
    return this.client.execute(
      sender,
      this.contractAddress,
      { withdraw: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
}
