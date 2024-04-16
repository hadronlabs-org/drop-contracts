import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
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
export type ArrayOfIdealDelegation = IdealDelegation[];
export type ArrayOfIdealDelegation1 = IdealDelegation[];

export interface DropStrategySchema {
  responses: ArrayOfIdealDelegation | ArrayOfIdealDelegation1 | ConfigResponse;
  query: CalcDepositArgs | CalcWithdrawArgs;
  execute: UpdateConfigArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface IdealDelegation {
  current_stake: Uint128;
  ideal_stake: Uint128;
  stake_change: Uint128;
  valoper_address: string;
  weight: number;
}
export interface ConfigResponse {
  core_address: string;
  denom: string;
  distribution_address: string;
  puppeteer_address: string;
  validator_set_address: string;
}
export interface CalcDepositArgs {
  deposit: Uint128;
}
export interface CalcWithdrawArgs {
  withdraw: Uint128;
}
export interface UpdateConfigArgs {
  core_address?: string | null;
  denom?: string | null;
  distribution_address?: string | null;
  puppeteer_address?: string | null;
  validator_set_address?: string | null;
}
export interface InstantiateMsg {
  core_address: string;
  denom: string;
  distribution_address: string;
  puppeteer_address: string;
  validator_set_address: string;
}


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
  queryConfig = async(): Promise<ConfigResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryCalcDeposit = async(args: CalcDepositArgs): Promise<ArrayOfIdealDelegation> => {
    return this.client.queryContractSmart(this.contractAddress, { calc_deposit: args });
  }
  queryCalcWithdraw = async(args: CalcWithdrawArgs): Promise<ArrayOfIdealDelegation> => {
    return this.client.queryContractSmart(this.contractAddress, { calc_withdraw: args });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
}
