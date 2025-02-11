import {
  CosmWasmClient,
  SigningCosmWasmClient,
  ExecuteResult,
  InstantiateResult,
} from '@cosmjs/cosmwasm-stargate';
import { StdFee } from '@cosmjs/amino';
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
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
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
export type Transaction =
  | {
      undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
      };
    }
  | {
      redelegate: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator_from: string;
        validator_to: string;
      };
    }
  | {
      withdraw_reward: {
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      tokenize_share: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      redeem_shares: {
        items: RedeemShareItem[];
      };
    }
  | {
      claim_rewards_and_optionaly_transfer: {
        denom: string;
        interchain_account_id: string;
        transfer?: TransferReadyBatchesMsg | null;
        validators: string[];
      };
    }
  | {
      i_b_c_transfer: {
        amount: number;
        denom: string;
        real_amount: number;
        reason: IBCTransferReason;
        recipient: string;
      };
    }
  | {
      stake: {
        amount: Uint128;
      };
    }
  | {
      transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
      };
    }
  | {
      setup_protocol: {
        interchain_account_id: string;
        rewards_withdraw_address: string;
      };
    };
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
export type IBCTransferReason = 'l_s_m_share' | 'delegate';
export type ArrayOfTransaction = Transaction[];
export type QueryExtMsg =
  | {
      delegations: {};
    }
  | {
      balances: {};
    }
  | {
      unbonding_delegations: {};
    };
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

export interface DropPuppeteerNativeSchema {
  responses: Config | Binary | OwnershipForString | ArrayOfTransaction;
  query: ExtensionArgs;
  execute:
    | SetupProtocolArgs
    | DelegateArgs
    | UndelegateArgs
    | ClaimRewardsAndOptionalyTransferArgs
    | UpdateConfigArgs
    | RegisterBalanceAndDelegatorDelegationsQueryArgs
    | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  allowed_senders: Addr[];
  distribution_module_contract: Addr;
  native_bond_provider: Addr;
  remote_denom: string;
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
export interface RedeemShareItem {
  amount: Uint128;
  local_denom: string;
  remote_denom: string;
}
export interface TransferReadyBatchesMsg {
  amount: Uint128;
  batch_ids: number[];
  emergency: boolean;
  recipient: string;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface ExtensionArgs {
  msg: QueryExtMsg;
}
export interface SetupProtocolArgs {
  rewards_withdraw_address: string;
}
export interface DelegateArgs {
  items: [string, Uint128][];
  reply_to: string;
}
export interface UndelegateArgs {
  batch_id: number;
  items: [string, Uint128][];
  reply_to: string;
}
export interface ClaimRewardsAndOptionalyTransferArgs {
  reply_to: string;
  transfer?: TransferReadyBatchesMsg | null;
  validators: string[];
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  allowed_senders?: string[] | null;
  distribution_module_contract?: string | null;
  native_bond_provider?: string | null;
  remote_denom?: string | null;
}
export interface RegisterBalanceAndDelegatorDelegationsQueryArgs {
  validators: string[];
}
export interface InstantiateMsg {
  allowed_senders: string[];
  distribution_module_contract: string;
  native_bond_provider: string;
  owner?: string | null;
  remote_denom: string;
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
  mustBeSigningClient(): Error {
    return new Error('This client is not a SigningCosmWasmClient');
  }
  static async instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
    admin?: string,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
      ...(admin && { admin: admin }),
    });
    return res;
  }
  static async instantiate2(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    salt: Uint8Array,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
    admin?: string,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate2(
      sender,
      codeId,
      salt,
      initMsg,
      label,
      fees,
      {
        ...(initCoins && initCoins.length && { funds: initCoins }),
        ...(admin && { admin: admin }),
      },
    );
    return res;
  }
  queryConfig = async (): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  };
  queryTransactions = async (): Promise<ArrayOfTransaction> => {
    return this.client.queryContractSmart(this.contractAddress, {
      transactions: {},
    });
  };
  queryExtension = async (args: ExtensionArgs): Promise<Binary> => {
    return this.client.queryContractSmart(this.contractAddress, {
      extension: args,
    });
  };
  queryOwnership = async (): Promise<OwnershipForString> => {
    return this.client.queryContractSmart(this.contractAddress, {
      ownership: {},
    });
  };
  setupProtocol = async (
    sender: string,
    args: SetupProtocolArgs,
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
      this.setupProtocolMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  setupProtocolMsg = (
    args: SetupProtocolArgs,
  ): { setup_protocol: SetupProtocolArgs } => {
    return { setup_protocol: args };
  };
  delegate = async (
    sender: string,
    args: DelegateArgs,
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
      this.delegateMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  delegateMsg = (args: DelegateArgs): { delegate: DelegateArgs } => {
    return { delegate: args };
  };
  undelegate = async (
    sender: string,
    args: UndelegateArgs,
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
      this.undelegateMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  undelegateMsg = (args: UndelegateArgs): { undelegate: UndelegateArgs } => {
    return { undelegate: args };
  };
  claimRewardsAndOptionalyTransfer = async (
    sender: string,
    args: ClaimRewardsAndOptionalyTransferArgs,
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
      this.claimRewardsAndOptionalyTransferMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  claimRewardsAndOptionalyTransferMsg = (
    args: ClaimRewardsAndOptionalyTransferArgs,
  ): {
    claim_rewards_and_optionaly_transfer: ClaimRewardsAndOptionalyTransferArgs;
  } => {
    return { claim_rewards_and_optionaly_transfer: args };
  };
  updateConfig = async (
    sender: string,
    args: UpdateConfigArgs,
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
      this.updateConfigMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  updateConfigMsg = (
    args: UpdateConfigArgs,
  ): { update_config: UpdateConfigArgs } => {
    return { update_config: args };
  };
  registerBalanceAndDelegatorDelegationsQuery = async (
    sender: string,
    args: RegisterBalanceAndDelegatorDelegationsQueryArgs,
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
      this.registerBalanceAndDelegatorDelegationsQueryMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  registerBalanceAndDelegatorDelegationsQueryMsg = (
    args: RegisterBalanceAndDelegatorDelegationsQueryArgs,
  ): {
    register_balance_and_delegator_delegations_query: RegisterBalanceAndDelegatorDelegationsQueryArgs;
  } => {
    return { register_balance_and_delegator_delegations_query: args };
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
      this.updateOwnershipMsg(args),
      fee || 'auto',
      memo,
      funds,
    );
  };
  updateOwnershipMsg = (
    args: UpdateOwnershipArgs,
  ): { update_ownership: UpdateOwnershipArgs } => {
    return { update_ownership: args };
  };
}
