import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
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
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration = {
    at_height: number;
} | {
    at_time: Timestamp;
} | {
    never: {};
};
/**
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp = Uint64;
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
export type ArrayOfValidatorInfo = ValidatorInfo[];
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
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
export interface DropValidatorsSetSchema {
    responses: Config | OwnershipForString | ValidatorResponse | ArrayOfValidatorInfo;
    query: ValidatorArgs;
    execute: UpdateConfigArgs | UpdateValidatorsArgs | UpdateValidatorsInfoArgs | UpdateValidatorsVotingArgs | UpdateOwnershipArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface Config {
    provider_proposals_contract?: Addr | null;
    stats_contract: Addr;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForString {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: string | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: string | null;
}
export interface ValidatorResponse {
    validator?: ValidatorInfo | null;
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
export interface ValidatorArgs {
    valoper: string;
}
export interface UpdateConfigArgs {
    new_config: ConfigOptional;
}
export interface ConfigOptional {
    provider_proposals_contract?: string | null;
    stats_contract?: string | null;
}
export interface UpdateValidatorsArgs {
    validators: ValidatorData[];
}
export interface ValidatorData {
    valoper_address: string;
    weight: number;
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
export interface InstantiateMsg {
    owner: string;
    stats_contract: string;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[], admin?: string): Promise<InstantiateResult>;
    queryConfig: () => Promise<Config>;
    queryValidator: (args: ValidatorArgs) => Promise<ValidatorResponse>;
    queryValidators: () => Promise<ArrayOfValidatorInfo>;
    queryOwnership: () => Promise<OwnershipForString>;
    updateConfig: (sender: string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateConfigMsg: (args: UpdateConfigArgs) => {
        update_config: UpdateConfigArgs;
    };
    updateValidators: (sender: string, args: UpdateValidatorsArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateValidatorsMsg: (args: UpdateValidatorsArgs) => {
        update_validators: UpdateValidatorsArgs;
    };
    updateValidatorsInfo: (sender: string, args: UpdateValidatorsInfoArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateValidatorsInfoMsg: (args: UpdateValidatorsInfoArgs) => {
        update_validators_info: UpdateValidatorsInfoArgs;
    };
    updateValidatorsVoting: (sender: string, args: UpdateValidatorsVotingArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateValidatorsVotingMsg: (args: UpdateValidatorsVotingArgs) => {
        update_validators_voting: UpdateValidatorsVotingArgs;
    };
    updateOwnership: (sender: string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateOwnershipMsg: (args: UpdateOwnershipArgs) => {
        update_ownership: UpdateOwnershipArgs;
    };
}
