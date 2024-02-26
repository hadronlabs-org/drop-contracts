import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
export interface InstantiateMsg {
  owner: string;
  stats_contract: string;
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

export interface LidoValidatorsSetSchema {
  responses: Config | ValidatorInfo | ArrayOfValidatorInfo;
  query: ValidatorArgs;
  execute:
    | UpdateConfigArgs
    | UpdateValidatorsArgs
    | UpdateValidatorArgs
    | UpdateValidatorsInfoArgs
    | UpdateValidatorsVotingArgs;
  [k: string]: unknown;
}
export interface Config {
  owner: Addr;
  provider_proposals_contract?: Addr | null;
  stats_contract: Addr;
}
export interface ValidatorInfo {
  init_proposal?: number | null;
  jailed_number?: number | null;
  last_commission_in_range?: number | null;
  last_processed_local_height?: number | null;
  last_processed_remote_height?: number | null;
  last_validated_height?: number | null;
  tombstone: boolean;
  total_passed_proposals: number;
  total_voted_proposals: number;
  uptime: Decimal;
  valoper_address: string;
  weight: number;
}
export interface ValidatorInfo1 {
  init_proposal?: number | null;
  jailed_number?: number | null;
  last_commission_in_range?: number | null;
  last_processed_local_height?: number | null;
  last_processed_remote_height?: number | null;
  last_validated_height?: number | null;
  tombstone: boolean;
  total_passed_proposals: number;
  total_voted_proposals: number;
  uptime: Decimal;
  valoper_address: string;
  weight: number;
}
export interface ValidatorArgs {
  valoper: Addr;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  owner?: Addr | null;
  provider_proposals_contract?: Addr | null;
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
export interface UpdateValidatorsInfoArgs {
  validators: ValidatorInfoUpdate[];
}
export interface ValidatorInfoUpdate {
  jailed_number?: number | null;
  last_commission_in_range?: number | null;
  last_processed_local_height?: number | null;
  last_processed_remote_height?: number | null;
  last_validated_height?: number | null;
  tombstone: boolean;
  uptime: Decimal;
  valoper_address: string;
}
export interface UpdateValidatorsVotingArgs {
  proposal: ProposalInfo;
}
export interface ProposalInfo {
  is_spam: boolean;
  proposal: Proposal;
  votes?: ProposalVote[] | null;
}
/**
 * Proposal defines the core field members of a governance proposal.
 */
export interface Proposal {
  deposit_end_time?: number | null;
  final_tally_result?: TallyResult | null;
  proposal_id: number;
  proposal_type?: string | null;
  status: number;
  submit_time?: number | null;
  total_deposit: Coin[];
  voting_end_time?: number | null;
  voting_start_time?: number | null;
  [k: string]: unknown;
}
/**
 * TallyResult defines a standard tally for a governance proposal.
 */
export interface TallyResult {
  abstain: Uint128;
  no: Uint128;
  no_with_veto: Uint128;
  yes: Uint128;
  [k: string]: unknown;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
/**
 * Proposal vote defines the core field members of a governance proposal votes.
 */
export interface ProposalVote {
  options: WeightedVoteOption[];
  proposal_id: number;
  voter: string;
  [k: string]: unknown;
}
/**
 * Proposal vote option defines the members of a governance proposal vote option.
 */
export interface WeightedVoteOption {
  option: number;
  weight: string;
  [k: string]: unknown;
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
  updateValidatorsInfo = async(sender:string, args: UpdateValidatorsInfoArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_validators_info: args }, fee || "auto", memo, funds);
  }
  updateValidatorsVoting = async(sender:string, args: UpdateValidatorsVotingArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_validators_voting: args }, fee || "auto", memo, funds);
  }
}
