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

export interface InstantiateMsg {
  owner: Addr;
  stats_contract: Addr;
}
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
export type ArrayOfValidatorInfo = ValidatorInfo1[];

export interface LidoValidatorsSetSchema {
  responses: Config | ValidatorInfo | ArrayOfValidatorInfo;
  query: ValidatorArgs;
  execute: UpdateConfigArgs | UpdateValidatorsArgs | UpdateValidatorArgs | UpdateValidatorInfoArgs;
  [k: string]: unknown;
}
export interface Config {
  owner: Addr;
  stats_contract: Addr;
}
export interface ValidatorInfo {
  jailed_number?: number | null;
  last_commission_in_range?: number | null;
  last_processed_local_height?: number | null;
  last_processed_remote_height?: number | null;
  last_validated_height?: number | null;
  tombstone: boolean;
  uptime: Decimal;
  valoper_address: string;
  weight: number;
}
export interface ValidatorInfo1 {
  jailed_number?: number | null;
  last_commission_in_range?: number | null;
  last_processed_local_height?: number | null;
  last_processed_remote_height?: number | null;
  last_validated_height?: number | null;
  tombstone: boolean;
  uptime: Decimal;
  valoper_address: string;
  weight: number;
}
export interface ValidatorArgs {
  valoper: Addr;
}
export interface UpdateConfigArgs {
  owner?: Addr | null;
  stats_contract?: Addr | null;
}
export interface UpdateValidatorsArgs {
  validators: ValidatorData[];
}
export interface ValidatorData {
  valoper_address: string;
  weight: number;
}
export interface UpdateValidatorArgs {
  validator: ValidatorData;
}
export interface UpdateValidatorInfoArgs {
  validators: ValidatorInfo1[];
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
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryValidator = async(args: ValidatorArgs): Promise<ValidatorInfo> => {
    return this.client.queryContractSmart(this.contractAddress, { validator: args });
  }
  queryValidators = async(): Promise<ArrayOfValidatorInfo> => {
    return this.client.queryContractSmart(this.contractAddress, { validators: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  updateValidators = async(sender:string, args: UpdateValidatorsArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_validators: args }, fee || "auto", memo, funds);
  }
  updateValidator = async(sender:string, args: UpdateValidatorArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_validator: args }, fee || "auto", memo, funds);
  }
  updateValidatorInfo = async(sender:string, args: UpdateValidatorInfoArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_validator_info: args }, fee || "auto", memo, funds);
  }
}
