import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface InstantiateMsg {}
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

export interface DropDistributionSchema {
  responses: ArrayOfIdealDelegation | ArrayOfIdealDelegation1;
  query: CalcDepositArgs | CalcWithdrawArgs;
  [k: string]: unknown;
}
export interface IdealDelegation {
  current_stake: Uint128;
  ideal_stake: Uint128;
  stake_change: Uint128;
  valoper_address: string;
  weight: number;
}
export interface CalcDepositArgs {
  delegations: Delegation[];
  deposit: Uint128;
}
export interface Delegation {
  stake: Uint128;
  valoper_address: string;
  weight: number;
}
export interface CalcWithdrawArgs {
  delegations: Delegation[];
  withdraw: Uint128;
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
    initCoins?: readonly Coin[],
    fees?: StdFee | 'auto' | number,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryCalcDeposit = async(args: CalcDepositArgs): Promise<ArrayOfIdealDelegation> => {
    return this.client.queryContractSmart(this.contractAddress, { calc_deposit: args });
  }
  queryCalcWithdraw = async(args: CalcWithdrawArgs): Promise<ArrayOfIdealDelegation> => {
    return this.client.queryContractSmart(this.contractAddress, { calc_withdraw: args });
  }
}
