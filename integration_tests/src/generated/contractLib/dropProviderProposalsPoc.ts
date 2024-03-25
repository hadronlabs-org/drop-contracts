import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
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
export type ArrayOfProposalInfo = ProposalInfo1[];

export interface DropProviderProposalsPocSchema {
  responses: Config | ProposalInfo | ArrayOfProposalInfo | Metrics;
  query: GetProposalArgs;
  execute: UpdateConfigArgs | UpdateProposalVotesArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  connection_id: string;
  core_address: string;
  init_proposal: number;
  port_id: string;
  proposal_votes_address?: string | null;
  proposals_prefetch: number;
  update_period: number;
  validators_set_address: string;
  veto_spam_threshold: Decimal;
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
export interface ProposalInfo1 {
  is_spam: boolean;
  proposal: Proposal;
  votes?: ProposalVote[] | null;
}
export interface Metrics {
  last_proposal: number;
}
export interface GetProposalArgs {
  proposal_id: number;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  connection_id?: string | null;
  core_address?: string | null;
  init_proposal?: number | null;
  port_id?: string | null;
  proposal_votes_address?: string | null;
  proposals_prefetch?: number | null;
  update_period?: number | null;
  validators_set_address?: string | null;
  veto_spam_threshold?: Decimal | null;
}
export interface UpdateProposalVotesArgs {
  votes: ProposalVote[];
}
export interface InstantiateMsg {
  connection_id: string;
  core_address: string;
  init_proposal: number;
  port_id: string;
  proposals_prefetch: number;
  update_period: number;
  validators_set_address: string;
  veto_spam_threshold: Decimal;
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
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryGetProposal = async(args: GetProposalArgs): Promise<ProposalInfo> => {
    return this.client.queryContractSmart(this.contractAddress, { get_proposal: args });
  }
  queryGetProposals = async(): Promise<ArrayOfProposalInfo> => {
    return this.client.queryContractSmart(this.contractAddress, { get_proposals: {} });
  }
  queryMetrics = async(): Promise<Metrics> => {
    return this.client.queryContractSmart(this.contractAddress, { metrics: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  updateProposalVotes = async(sender:string, args: UpdateProposalVotesArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_proposal_votes: args }, fee || "auto", memo, funds);
  }
}
