import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/amino";
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
export type Null = null;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
export type Null1 = null;
export type ArrayOfAttribute = Attribute[];
export type NullableNftInfoResponseForNullableMetadata = NftInfoResponseFor_Nullable_Metadata | null;
export type NullableString = string | null;
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateMinterOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateCreatorOwnershipArgs = {
    transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
    };
} | "accept_ownership" | "renounce_ownership";
export interface DropWithdrawalVoucherSchema {
    responses: AllNftInfoResponseForNullableMetadata | OperatorsResponse | TokensResponse | ApprovalResponse | ApprovalsResponse | CollectionInfoAndExtensionResponseForEmpty | Null | AllInfoResponse | Null1 | ArrayOfAttribute | CollectionInfoAndExtensionResponseForEmpty1 | ConfigResponseForEmpty | OwnershipForAddr | OwnershipForAddr1 | NullableNftInfoResponseForNullableMetadata | NullableString | MinterResponse | NftInfoResponseForNullableMetadata | NumTokensResponse | OperatorResponse | OwnerOfResponse1 | OwnershipForAddr2 | TokensResponse1;
    query: OwnerOfArgs | ApprovalArgs | ApprovalsArgs | OperatorArgs | AllOperatorsArgs | NftInfoArgs | GetNftByExtensionArgs | AllNftInfoArgs | TokensArgs | AllTokensArgs | ExtensionArgs | GetCollectionExtensionArgs;
    execute: UpdateOwnershipArgs | UpdateMinterOwnershipArgs | UpdateCreatorOwnershipArgs | UpdateCollectionInfoArgs | TransferNftArgs | SendNftArgs | ApproveArgs | RevokeArgs | ApproveAllArgs | RevokeAllArgs | MintArgs | BurnArgs | UpdateExtensionArgs | UpdateNftInfoArgs | SetWithdrawAddressArgs | WithdrawFundsArgs;
    instantiate?: InstantiateMsg;
    [k: string]: unknown;
}
export interface AllNftInfoResponseForNullableMetadata {
    /**
     * Who can transfer the token
     */
    access: OwnerOfResponse;
    /**
     * Data on the token itself,
     */
    info: NftInfoResponseFor_Nullable_Metadata;
}
export interface OwnerOfResponse {
    /**
     * If set this address is approved to transfer/send the token as well
     */
    approvals: Approval[];
    /**
     * Owner of the token
     */
    owner: string;
}
export interface Approval {
    /**
     * When the Approval expires (maybe Expiration::never)
     */
    expires: Expiration;
    /**
     * Account that can transfer/send the token
     */
    spender: Addr;
}
export interface NftInfoResponseFor_Nullable_Metadata {
    /**
     * You can add any custom metadata here when you extend cw721-base
     */
    extension?: Metadata | null;
    /**
     * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
     */
    token_uri?: string | null;
}
export interface Metadata {
    amount: Uint128;
    attributes?: Trait[] | null;
    batch_id: string;
    description?: string | null;
    name: string;
}
export interface Trait {
    display_type?: string | null;
    trait_type: string;
    value: string;
}
export interface OperatorsResponse {
    operators: Approval[];
}
export interface TokensResponse {
    /**
     * Contains all token_ids in lexicographical ordering If there are more than `limit`, use `start_after` in future queries to achieve pagination.
     */
    tokens: string[];
}
export interface ApprovalResponse {
    approval: Approval;
}
export interface ApprovalsResponse {
    approvals: Approval[];
}
/**
 * This is a wrapper around CollectionInfo that includes the extension.
 */
export interface CollectionInfoAndExtensionResponseForEmpty {
    extension: Empty;
    name: string;
    symbol: string;
    updated_at: Timestamp;
}
/**
 * An empty struct that serves as a placeholder in different places, such as contracts that don't set a custom message.
 *
 * It is designed to be expressible in correct JSON and JSON Schema but contains no meaningful data. Previously we used enums without cases, but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {
}
/**
 * This is a wrapper around CollectionInfo that includes the extension, contract info, and number of tokens (supply).
 */
export interface AllInfoResponse {
    collection_extension: Attribute[];
    collection_info: CollectionInfo;
    contract_info: ContractInfoResponse;
    num_tokens: number;
}
export interface Attribute {
    key: string;
    value: Binary;
}
export interface CollectionInfo {
    name: string;
    symbol: string;
    updated_at: Timestamp;
}
export interface ContractInfoResponse {
    /**
     * admin who can run migrations (if any)
     */
    admin?: Addr | null;
    code_id: number;
    /**
     * address that instantiated this contract
     */
    creator: Addr;
    /**
     * set if this contract has bound an IBC port
     */
    ibc_port?: string | null;
    /**
     * if set, the contract is pinned to the cache, and thus uses less gas when called
     */
    pinned: boolean;
}
/**
 * This is a wrapper around CollectionInfo that includes the extension.
 */
export interface CollectionInfoAndExtensionResponseForEmpty1 {
    extension: Empty;
    name: string;
    symbol: string;
    updated_at: Timestamp;
}
/**
 * This is a wrapper around CollectionInfo that includes the extension.
 */
export interface ConfigResponseForEmpty {
    collection_extension: Empty;
    collection_info: CollectionInfo;
    contract_info: ContractInfoResponse;
    creator_ownership: OwnershipFor_Addr;
    minter_ownership: OwnershipFor_Addr;
    num_tokens: number;
    withdraw_address?: string | null;
}
/**
 * The contract's ownership info
 */
export interface OwnershipFor_Addr {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: Addr | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: Addr | null;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForAddr {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: Addr | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: Addr | null;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForAddr1 {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: Addr | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: Addr | null;
}
/**
 * Deprecated: use Cw721QueryMsg::GetMinterOwnership instead! Shows who can mint these tokens.
 */
export interface MinterResponse {
    minter?: string | null;
}
export interface NftInfoResponseForNullableMetadata {
    /**
     * You can add any custom metadata here when you extend cw721-base
     */
    extension?: Metadata | null;
    /**
     * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
     */
    token_uri?: string | null;
}
export interface NumTokensResponse {
    count: number;
}
export interface OperatorResponse {
    approval: Approval;
}
export interface OwnerOfResponse1 {
    /**
     * If set this address is approved to transfer/send the token as well
     */
    approvals: Approval[];
    /**
     * Owner of the token
     */
    owner: string;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForAddr2 {
    /**
     * The contract's current owner. `None` if the ownership has been renounced.
     */
    owner?: Addr | null;
    /**
     * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
     */
    pending_expiry?: Expiration | null;
    /**
     * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
     */
    pending_owner?: Addr | null;
}
export interface TokensResponse1 {
    /**
     * Contains all token_ids in lexicographical ordering If there are more than `limit`, use `start_after` in future queries to achieve pagination.
     */
    tokens: string[];
}
export interface OwnerOfArgs {
    /**
     * unset or false will filter out expired approvals, you must set to true to see them
     */
    include_expired?: boolean | null;
    token_id: string;
}
export interface ApprovalArgs {
    include_expired?: boolean | null;
    spender: string;
    token_id: string;
}
export interface ApprovalsArgs {
    include_expired?: boolean | null;
    token_id: string;
}
export interface OperatorArgs {
    include_expired?: boolean | null;
    operator: string;
    owner: string;
}
export interface AllOperatorsArgs {
    /**
     * unset or false will filter out expired items, you must set to true to see them
     */
    include_expired?: boolean | null;
    limit?: number | null;
    owner: string;
    start_after?: string | null;
}
export interface NftInfoArgs {
    token_id: string;
}
export interface GetNftByExtensionArgs {
    extension?: Metadata | null;
    limit?: number | null;
    start_after?: string | null;
}
export interface AllNftInfoArgs {
    /**
     * unset or false will filter out expired approvals, you must set to true to see them
     */
    include_expired?: boolean | null;
    token_id: string;
}
export interface TokensArgs {
    limit?: number | null;
    owner: string;
    start_after?: string | null;
}
export interface AllTokensArgs {
    limit?: number | null;
    start_after?: string | null;
}
export interface ExtensionArgs {
    msg: Empty;
}
export interface GetCollectionExtensionArgs {
    msg: Empty;
}
export interface UpdateCollectionInfoArgs {
    collection_info: CollectionInfoMsgFor_Empty;
}
export interface CollectionInfoMsgFor_Empty {
    extension: Empty;
    name?: string | null;
    symbol?: string | null;
}
export interface TransferNftArgs {
    recipient: string;
    token_id: string;
}
export interface SendNftArgs {
    contract: string;
    msg: Binary;
    token_id: string;
}
export interface ApproveArgs {
    expires?: Expiration | null;
    spender: string;
    token_id: string;
}
export interface RevokeArgs {
    spender: string;
    token_id: string;
}
export interface ApproveAllArgs {
    expires?: Expiration | null;
    operator: string;
}
export interface RevokeAllArgs {
    operator: string;
}
export interface MintArgs {
    /**
     * Any custom extension used by this contract
     */
    extension?: Metadata | null;
    /**
     * The owner of the newly minter NFT
     */
    owner: string;
    /**
     * Unique ID of the NFT
     */
    token_id: string;
    /**
     * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
     */
    token_uri?: string | null;
}
export interface BurnArgs {
    token_id: string;
}
export interface UpdateExtensionArgs {
    msg: Empty;
}
export interface UpdateNftInfoArgs {
    extension?: Metadata | null;
    token_id: string;
    /**
     * NOTE: Empty string is handled as None
     */
    token_uri?: string | null;
}
export interface SetWithdrawAddressArgs {
    address: string;
}
export interface WithdrawFundsArgs {
    amount: Coin;
}
export interface Coin {
    amount: Uint128;
    denom: string;
}
export interface InstantiateMsg {
    /**
     * Optional extension of the collection metadata
     */
    collection_info_extension: Empty;
    /**
     * Sets the creator of collection. The creator is the only one eligible to update `CollectionInfo`.
     */
    creator?: string | null;
    /**
     * The minter is the only one who can create new NFTs. This is designed for a base NFT that is controlled by an external program or contract. You will likely replace this with custom logic in custom NFTs
     */
    minter?: string | null;
    /**
     * Name of the NFT contract
     */
    name: string;
    /**
     * Symbol of the NFT contract
     */
    symbol: string;
    withdraw_address?: string | null;
}
export declare class Client {
    private readonly client;
    contractAddress: string;
    constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string);
    mustBeSigningClient(): Error;
    static instantiate(client: SigningCosmWasmClient, sender: string, codeId: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    static instantiate2(client: SigningCosmWasmClient, sender: string, codeId: number, salt: number, initMsg: InstantiateMsg, label: string, fees: StdFee | 'auto' | number, initCoins?: readonly Coin[]): Promise<InstantiateResult>;
    queryOwnerOf: (args: OwnerOfArgs) => Promise<OwnerOfResponse>;
    queryApproval: (args: ApprovalArgs) => Promise<ApprovalResponse>;
    queryApprovals: (args: ApprovalsArgs) => Promise<ApprovalsResponse>;
    queryOperator: (args: OperatorArgs) => Promise<OperatorResponse>;
    queryAllOperators: (args: AllOperatorsArgs) => Promise<OperatorsResponse>;
    queryNumTokens: () => Promise<NumTokensResponse>;
    queryContractInfo: () => Promise<CollectionInfoAndExtensionResponseForEmpty>;
    queryGetConfig: () => Promise<ConfigResponseForEmpty>;
    queryGetCollectionInfoAndExtension: () => Promise<CollectionInfoAndExtensionResponseForEmpty>;
    queryGetAllInfo: () => Promise<AllInfoResponse>;
    queryGetCollectionExtensionAttributes: () => Promise<ArrayOfAttribute>;
    queryOwnership: () => Promise<OwnershipForAddr>;
    queryMinter: () => Promise<MinterResponse>;
    queryGetMinterOwnership: () => Promise<OwnershipForAddr>;
    queryGetCreatorOwnership: () => Promise<OwnershipForAddr>;
    queryNftInfo: (args: NftInfoArgs) => Promise<NftInfoResponseForNullableMetadata>;
    queryGetNftByExtension: (args: GetNftByExtensionArgs) => Promise<NullableNftInfoResponseForNullableMetadata>;
    queryAllNftInfo: (args: AllNftInfoArgs) => Promise<AllNftInfoResponseForNullableMetadata>;
    queryTokens: (args: TokensArgs) => Promise<TokensResponse>;
    queryAllTokens: (args: AllTokensArgs) => Promise<TokensResponse>;
    queryExtension: (args: ExtensionArgs) => Promise<Null>;
    queryGetCollectionExtension: (args: GetCollectionExtensionArgs) => Promise<Null>;
    queryGetWithdrawAddress: () => Promise<NullableString>;
    updateOwnership: (sender: string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateMinterOwnership: (sender: string, args: UpdateMinterOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateCreatorOwnership: (sender: string, args: UpdateCreatorOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateCollectionInfo: (sender: string, args: UpdateCollectionInfoArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    transferNft: (sender: string, args: TransferNftArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    sendNft: (sender: string, args: SendNftArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    approve: (sender: string, args: ApproveArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    revoke: (sender: string, args: RevokeArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    approveAll: (sender: string, args: ApproveAllArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    revokeAll: (sender: string, args: RevokeAllArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    mint: (sender: string, args: MintArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    burn: (sender: string, args: BurnArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateExtension: (sender: string, args: UpdateExtensionArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    updateNftInfo: (sender: string, args: UpdateNftInfoArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    setWithdrawAddress: (sender: string, args: SetWithdrawAddressArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    removeWithdrawAddress: (sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
    withdrawFunds: (sender: string, args: WithdrawFundsArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
