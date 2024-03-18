import {
  CosmWasmClient,
  SigningCosmWasmClient,
  ExecuteResult,
  InstantiateResult,
} from '@cosmjs/cosmwasm-stargate';
import { StdFee } from '@cosmjs/amino';
import { Coin } from '@cosmjs/amino';
export interface InstantiateMsg {
  /**
   * The minter is the only one who can create new NFTs. This is designed for a base NFT that is controlled by an external program or contract. You will likely replace this with custom logic in custom NFTs
   */
  minter: string;
  /**
   * Name of the NFT contract
   */
  name: string;
  /**
   * Symbol of the NFT contract
   */
  symbol: string;
}
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration =
  | {
      at_height: number;
    }
  | {
      at_time: Timestamp;
    }
  | {
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
export type Null = null;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
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
export type UpdateOwnershipArgs =
  | {
      transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
      };
    }
  | 'accept_ownership'
  | 'renounce_ownership';

export interface DropWithdrawalVoucherSchema {
  responses:
    | AllNftInfoResponseFor_Empty
    | OperatorsResponse
    | TokensResponse
    | ApprovalResponse
    | ApprovalsResponse
    | ContractInfoResponse
    | Null
    | MinterResponse
    | NftInfoResponseFor_Empty1
    | NumTokensResponse
    | OperatorResponse
    | OwnerOfResponse1
    | OwnershipFor_String
    | TokensResponse1;
  query:
    | OwnerOfArgs
    | ApprovalArgs
    | ApprovalsArgs
    | OperatorArgs
    | AllOperatorsArgs
    | NftInfoArgs
    | AllNftInfoArgs
    | TokensArgs
    | AllTokensArgs
    | ExtensionArgs;
  execute:
    | TransferNftArgs
    | SendNftArgs
    | ApproveArgs
    | RevokeArgs
    | ApproveAllArgs
    | RevokeAllArgs
    | MintArgs
    | BurnArgs
    | ExtensionArgs1
    | UpdateOwnershipArgs;
  [k: string]: unknown;
}
export interface AllNftInfoResponseFor_Empty {
  /**
   * Who can transfer the token
   */
  access: OwnerOfResponse;
  /**
   * Data on the token itself,
   */
  info: NftInfoResponseFor_Empty;
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
  spender: string;
}
export interface NftInfoResponseFor_Empty {
  /**
   * You can add any custom metadata here when you extend cw721-base
   */
  extension: Empty;
  /**
   * Universal resource identifier for this NFT Should point to a JSON file that conforms to the ERC721 Metadata JSON Schema
   */
  token_uri?: string | null;
}
/**
 * An empty struct that serves as a placeholder in different places, such as contracts that don't set a custom message.
 *
 * It is designed to be expressable in correct JSON and JSON Schema but contains no meaningful data. Previously we used enums without cases, but those cannot represented as valid JSON Schema (https://github.com/CosmWasm/cosmwasm/issues/451)
 */
export interface Empty {
  [k: string]: unknown;
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
export interface ContractInfoResponse {
  name: string;
  symbol: string;
}
/**
 * Shows who can mint these tokens
 */
export interface MinterResponse {
  minter?: string | null;
}
export interface NftInfoResponseFor_Empty1 {
  /**
   * You can add any custom metadata here when you extend cw721-base
   */
  extension: Empty;
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
export interface OwnershipFor_String {
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
export interface Metadata {
  amount: Uint128;
  attributes?: Trait[] | null;
  batch_id: string;
  description?: string | null;
  expected_amount: Uint128;
  name: string;
}
export interface Trait {
  display_type?: string | null;
  trait_type: string;
  value: string;
}
export interface BurnArgs {
  token_id: string;
}
export interface ExtensionArgs1 {
  msg: Empty;
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
  queryOwnerOf = async (args: OwnerOfArgs): Promise<OwnerOfResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      owner_of: args,
    });
  };
  queryApproval = async (args: ApprovalArgs): Promise<ApprovalResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      approval: args,
    });
  };
  queryApprovals = async (args: ApprovalsArgs): Promise<ApprovalsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      approvals: args,
    });
  };
  queryOperator = async (args: OperatorArgs): Promise<OperatorResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      operator: args,
    });
  };
  queryAllOperators = async (
    args: AllOperatorsArgs,
  ): Promise<OperatorsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_operators: args,
    });
  };
  queryNumTokens = async (): Promise<NumTokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      num_tokens: {},
    });
  };
  queryContractInfo = async (): Promise<ContractInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      contract_info: {},
    });
  };
  queryNftInfo = async (
    args: NftInfoArgs,
  ): Promise<NftInfoResponse_for_Empty> => {
    return this.client.queryContractSmart(this.contractAddress, {
      nft_info: args,
    });
  };
  queryAllNftInfo = async (
    args: AllNftInfoArgs,
  ): Promise<AllNftInfoResponse_for_Empty> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_nft_info: args,
    });
  };
  queryTokens = async (args: TokensArgs): Promise<TokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      tokens: args,
    });
  };
  queryAllTokens = async (args: AllTokensArgs): Promise<TokensResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      all_tokens: args,
    });
  };
  queryMinter = async (): Promise<MinterResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { minter: {} });
  };
  queryExtension = async (args: ExtensionArgs): Promise<Null> => {
    return this.client.queryContractSmart(this.contractAddress, {
      extension: args,
    });
  };
  queryOwnership = async (): Promise<Ownership_for_String> => {
    return this.client.queryContractSmart(this.contractAddress, {
      ownership: {},
    });
  };
  transferNft = async (
    sender: string,
    args: TransferNftArgs,
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
      { transfer_nft: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  sendNft = async (
    sender: string,
    args: SendNftArgs,
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
      { send_nft: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  approve = async (
    sender: string,
    args: ApproveArgs,
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
      { approve: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  revoke = async (
    sender: string,
    args: RevokeArgs,
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
      { revoke: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  approveAll = async (
    sender: string,
    args: ApproveAllArgs,
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
      { approve_all: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  revokeAll = async (
    sender: string,
    args: RevokeAllArgs,
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
      { revoke_all: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  mint = async (
    sender: string,
    args: MintArgs,
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
      { mint: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  burn = async (
    sender: string,
    args: BurnArgs,
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
      { burn: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  extension = async (
    sender: string,
    args: ExtensionArgs,
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
      { extension: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
  updateOwnership = async (
    sender: string,
    args: UpdateOwnershipArgs,
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
      { update_ownership: args },
      fee || 'auto',
      memo,
      funds,
    );
  };
}
