import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface DropTemplateFactoryContractSchema {
  responses: State;
  execute: UpdateStateArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface State {
  core_contract: string;
  distribution_contract: string;
  lsm_share_bond_provider_contract: string;
  native_bond_provider_contract: string;
  puppeteer_contract: string;
  rewards_manager_contract: string;
  rewards_pump_contract: string;
  splitter_contract: string;
  strategy_contract: string;
  token_contract: string;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}
export interface UpdateStateArgs {
  state: State1;
}
export interface State1 {
  core_contract: string;
  distribution_contract: string;
  lsm_share_bond_provider_contract: string;
  native_bond_provider_contract: string;
  puppeteer_contract: string;
  rewards_manager_contract: string;
  rewards_pump_contract: string;
  splitter_contract: string;
  strategy_contract: string;
  token_contract: string;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
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
  static async instantiate2(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    salt: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult> {
    const res = await client.instantiate2(sender, codeId, new Uint8Array([salt]), initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryState = async(): Promise<State> => {
    return this.client.queryContractSmart(this.contractAddress, { state: {} });
  }
  updateState = async(sender:string, args: UpdateStateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_state: args }, fee || "auto", memo, funds);
  }
}
