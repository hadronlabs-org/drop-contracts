import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
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
export type UpdateConfigArgs =
  | {
      core: ConfigOptional;
    }
  | {
      validators_set: ConfigOptional2;
    };
export type ProxyArgs = {
  validator_set: ValidatorSetMsg;
};
export type ValidatorSetMsg = {
  update_validators: {
    validators: ValidatorData[];
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
export type CosmosMsgFor_NeutronMsg =
  | {
      bank: BankMsg;
    }
  | {
      custom: NeutronMsg;
    }
  | {
      staking: StakingMsg;
    }
  | {
      distribution: DistributionMsg;
    }
  | {
      stargate: {
        type_url: string;
        value: Binary;
      };
    }
  | {
      any: AnyMsg;
    }
  | {
      ibc: IbcMsg;
    }
  | {
      wasm: WasmMsg;
    }
  | {
      gov: GovMsg;
    };
/**
 * The message types of the bank module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/bank/v1beta1/tx.proto
 */
export type BankMsg =
  | {
      send: {
        amount: Coin[];
        to_address: string;
      };
    }
  | {
      burn: {
        amount: Coin[];
      };
    };
/**
 * A number of Custom messages that can call into the Neutron bindings.
 */
export type NeutronMsg =
  | {
      register_interchain_account: {
        /**
         * *connection_id** is an IBC connection identifier between Neutron and remote chain.
         */
        connection_id: string;
        /**
         * **interchain_account_id** is an identifier of your new interchain account. Can be any string. This identifier allows contracts to have multiple interchain accounts on remote chains.
         */
        interchain_account_id: string;
        /**
         * *register_fee** is a fees required to be payed to register interchain account
         */
        register_fee?: Coin[] | null;
      };
    }
  | {
      submit_tx: {
        /**
         * *connection_id** is an IBC connection identifier between Neutron and remote chain.
         */
        connection_id: string;
        /**
         * **fee** is an ibc fee for the transaction.
         */
        fee: IbcFee;
        /**
         * *interchain_account_id** is an identifier of your interchain account from which you want to execute msgs.
         */
        interchain_account_id: string;
        /**
         * *memo** is a memo you want to attach to your interchain transaction.It behaves like a memo in usual Cosmos transaction.
         */
        memo: string;
        /**
         * *msgs** is a list of protobuf encoded Cosmos-SDK messages you want to execute on remote chain.
         */
        msgs: ProtobufAny[];
        /**
         * *timeout** is a timeout in seconds after which the packet times out.
         */
        timeout: number;
      };
    }
  | {
      register_interchain_query: {
        /**
         * *connection_id** is an IBC connection identifier between Neutron and remote chain.
         */
        connection_id: string;
        /**
         * *keys** is the KV-storage keys for which we want to get values from remote chain.
         */
        keys: KVKey[];
        /**
         * *query_type** is a query type identifier ('tx' or 'kv' for now).
         */
        query_type: string;
        /**
         * *transactions_filter** is the filter for transaction search ICQ.
         */
        transactions_filter: string;
        /**
         * *update_period** is used to say how often the query must be updated.
         */
        update_period: number;
      };
    }
  | {
      update_interchain_query: {
        /**
         * *new_keys** is the new query keys to retrive.
         */
        new_keys?: KVKey[] | null;
        /**
         * *new_transactions_filter** is a new transactions filter of the query.
         */
        new_transactions_filter?: string | null;
        /**
         * *new_update_period** is a new update period of the query.
         */
        new_update_period?: number | null;
        /**
         * *query_id** is the ID of the query we want to update.
         */
        query_id: number;
      };
    }
  | {
      remove_interchain_query: {
        /**
         * *query_id** is ID of the query we want to remove.
         */
        query_id: number;
      };
    }
  | {
      ibc_transfer: {
        fee: IbcFee;
        memo: string;
        receiver: string;
        sender: string;
        source_channel: string;
        source_port: string;
        timeout_height: RequestPacketTimeoutHeight;
        timeout_timestamp: number;
        token: Coin;
      };
    }
  | {
      submit_admin_proposal: {
        admin_proposal: AdminProposal;
      };
    }
  | {
      create_denom: {
        subdenom: string;
      };
    }
  | {
      change_admin: {
        denom: string;
        new_admin_address: string;
      };
    }
  | {
      mint_tokens: {
        amount: Uint128;
        denom: string;
        mint_to_address: string;
      };
    }
  | {
      burn_tokens: {
        amount: Uint128;
        /**
         * Must be set to `""` for now
         */
        burn_from_address: string;
        denom: string;
      };
    }
  | {
      set_before_send_hook: {
        contract_addr: string;
        denom: string;
      };
    }
  | {
      force_transfer: {
        amount: Uint128;
        denom: string;
        transfer_from_address: string;
        transfer_to_address: string;
      };
    }
  | {
      set_denom_metadata: {
        /**
         * *base** represents the base denom (should be the DenomUnit with exponent = 0).
         */
        base: string;
        /**
         * *denom_units** represents the list of DenomUnit's for a given coin
         */
        denom_units: DenomUnit[];
        /**
         * *description** description of a token
         */
        description: string;
        /**
         * **display** indicates the suggested denom that should be displayed in clients.
         */
        display: string;
        /**
         * *name** defines the name of the token (eg: Cosmos Atom)
         */
        name: string;
        /**
         * **symbol** is the token symbol usually shown on exchanges (eg: ATOM). This can be the same as the display.
         */
        symbol: string;
        /**
         * *uri** to a document (on or off-chain) that contains additional information. Optional.
         */
        uri: string;
        /**
         * **uri_hash** is a sha256 hash of a document pointed by URI. It's used to verify that the document didn't change. Optional.
         */
        uri_hash: string;
      };
    }
  | {
      add_schedule: {
        /**
         * list of cosmwasm messages to be executed
         */
        msgs: MsgExecuteContract[];
        /**
         * Name of a new schedule. Needed to be able to `RemoveSchedule` and to log information about it
         */
        name: string;
        /**
         * period in blocks with which `msgs` will be executed
         */
        period: number;
      };
    }
  | {
      remove_schedule: {
        name: string;
      };
    }
  | {
      resubmit_failure: {
        failure_id: number;
      };
    }
  | {
      dex: DexMsg;
    };
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
/**
 * AdminProposal defines the struct for various proposals which Neutron's Admin Module may accept.
 */
export type AdminProposal =
  | {
      param_change_proposal: ParamChangeProposal;
    }
  | {
      upgrade_proposal: UpgradeProposal;
    }
  | {
      client_update_proposal: ClientUpdateProposal;
    }
  | {
      proposal_execute_message: ProposalExecuteMessage;
    }
  | {
      software_upgrade_proposal: SoftwareUpgradeProposal;
    }
  | {
      cancel_software_upgrade_proposal: CancelSoftwareUpgradeProposal;
    }
  | {
      pin_codes_proposal: PinCodesProposal;
    }
  | {
      unpin_codes_proposal: UnpinCodesProposal;
    }
  | {
      sudo_contract_proposal: SudoContractProposal;
    }
  | {
      update_admin_proposal: UpdateAdminProposal;
    }
  | {
      clear_admin_proposal: ClearAdminProposal;
    };
export type DexMsg =
  | {
      deposit: {
        /**
         * Amounts of tokenA to deposit
         */
        amounts_a: Uint128[];
        /**
         * Amounts of tokenB to deposit
         */
        amounts_b: Uint128[];
        /**
         * Fees to use for each deposit
         */
        fees: number[];
        /**
         * Additional deposit options
         */
        options: DepositOption[];
        /**
         * The account to which PoolShares will be issued
         */
        receiver: string;
        /**
         * Tick indexes to deposit at defined in terms of TokenA to TokenB (ie. TokenA is on the left)
         */
        tick_indexes_a_to_b: number[];
        /**
         * Denom for one side of the deposit
         */
        token_a: string;
        /**
         * Denom for the opposing side of the deposit
         */
        token_b: string;
      };
    }
  | {
      withdrawal: {
        /**
         * Fee for the target LiquidityPools
         */
        fees: number[];
        /**
         * The account to which the tokens are credited
         */
        receiver: string;
        /**
         * Amount of shares to remove from each pool
         */
        shares_to_remove: Uint128[];
        /**
         * Tick indexes of the target LiquidityPools defined in terms of TokenA to TokenB (ie. TokenA is on the left)
         */
        tick_indexes_a_to_b: number[];
        /**
         * Denom for one side of the deposit
         */
        token_a: string;
        /**
         * Denom for the opposing side of the deposit
         */
        token_b: string;
      };
    }
  | {
      place_limit_order: {
        /**
         * Amount of TokenIn to be traded
         */
        amount_in: Uint128;
        /**
         * Expiration time for order. Only valid for GOOD_TIL_TIME limit orders
         */
        expiration_time?: number | null;
        /**
         * Accepts standard decimals and decimals with scientific notation (ie. 1234.23E-7)
         */
        limit_sell_price: string;
        /**
         * Maximum amount of TokenB can be bought. For everything except JUST_IN_TIME OrderType
         */
        max_amount_out?: Uint128 | null;
        /**
         * Type of limit order to be used. Must be one of: GOOD_TIL_CANCELLED, FILL_OR_KILL, IMMEDIATE_OR_CANCEL, JUST_IN_TIME, or GOOD_TIL_TIME
         */
        order_type: LimitOrderType;
        /**
         * Account to which TokenOut is credited or that will be allowed to withdraw or cancel a maker order
         */
        receiver: string;
        /**
         * Limit tick for a limit order, specified in terms of TokenIn to TokenOut
         */
        tick_index_in_to_out: number;
        /**
         * Token being “sold”
         */
        token_in: string;
        /**
         * Token being “bought”
         */
        token_out: string;
      };
    }
  | {
      withdraw_filled_limit_order: {
        /**
         * TrancheKey for the target limit order
         */
        tranche_key: string;
      };
    }
  | {
      cancel_limit_order: {
        /**
         * TrancheKey for the target limit order
         */
        tranche_key: string;
      };
    }
  | {
      multi_hop_swap: {
        /**
         * Amount of TokenIn to swap
         */
        amount_in: Uint128;
        /**
         * Minimum price that that must be satisfied for a route to succeed
         */
        exit_limit_price: PrecDec;
        /**
         * If true all routes are run and the route with the best price is used
         */
        pick_best_route: boolean;
        /**
         * Account to which TokenOut is credited
         */
        receiver: string;
        /**
         * Array of possible routes
         */
        routes: MultiHopRoute[];
      };
    };
export type LimitOrderType =
  | "GOOD_TIL_CANCELLED"
  | "FILL_OR_KILL"
  | "IMMEDIATE_OR_CANCEL"
  | "JUST_IN_TIME"
  | "GOOD_TIL_TIME";
/**
 * The message types of the staking module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/cosmos/staking/v1beta1/tx.proto
 */
export type StakingMsg =
  | {
      delegate: {
        amount: Coin;
        validator: string;
      };
    }
  | {
      undelegate: {
        amount: Coin;
        validator: string;
      };
    }
  | {
      redelegate: {
        amount: Coin;
        dst_validator: string;
        src_validator: string;
      };
    };
/**
 * The message types of the distribution module.
 *
 * See https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto
 */
export type DistributionMsg =
  | {
      set_withdraw_address: {
        /**
         * The `withdraw_address`
         */
        address: string;
      };
    }
  | {
      withdraw_delegator_reward: {
        /**
         * The `validator_address`
         */
        validator: string;
      };
    }
  | {
      fund_community_pool: {
        /**
         * The amount to spend
         */
        amount: Coin[];
      };
    };
/**
 * These are messages in the IBC lifecycle. Only usable by IBC-enabled contracts (contracts that directly speak the IBC protocol via 6 entry points)
 */
export type IbcMsg =
  | {
      transfer: {
        /**
         * packet data only supports one coin https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
         */
        amount: Coin;
        /**
         * existing channel to send the tokens over
         */
        channel_id: string;
        /**
         * An optional memo. See the blog post ["Moving Beyond Simple Token Transfers"](https://medium.com/the-interchain-foundation/moving-beyond-simple-token-transfers-d42b2b1dc29b) for more information.
         *
         * There is no difference between setting this to `None` or an empty string.
         *
         * This field is only supported on chains with CosmWasm >= 2.0 and silently ignored on older chains. If you need support for both 1.x and 2.x chain with the same codebase, it is recommended to use `CosmosMsg::Stargate` with a custom MsgTransfer protobuf encoder instead.
         */
        memo?: string | null;
        /**
         * when packet times out, measured on remote chain
         */
        timeout: IbcTimeout;
        /**
         * address on the remote chain to receive these tokens
         */
        to_address: string;
      };
    }
  | {
      send_packet: {
        channel_id: string;
        data: Binary;
        /**
         * when packet times out, measured on remote chain
         */
        timeout: IbcTimeout;
      };
    }
  | {
      close_channel: {
        channel_id: string;
      };
    };
/**
 * The message types of the wasm module.
 *
 * See https://github.com/CosmWasm/wasmd/blob/v0.14.0/x/wasm/internal/types/tx.proto
 */
export type WasmMsg =
  | {
      execute: {
        contract_addr: string;
        funds: Coin[];
        /**
         * msg is the json-encoded ExecuteMsg struct (as raw Binary)
         */
        msg: Binary;
      };
    }
  | {
      instantiate: {
        admin?: string | null;
        code_id: number;
        funds: Coin[];
        /**
         * A human-readable label for the contract.
         *
         * Valid values should: - not be empty - not be bigger than 128 bytes (or some chain-specific limit) - not start / end with whitespace
         */
        label: string;
        /**
         * msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
         */
        msg: Binary;
      };
    }
  | {
      instantiate2: {
        admin?: string | null;
        code_id: number;
        funds: Coin[];
        /**
         * A human-readable label for the contract.
         *
         * Valid values should: - not be empty - not be bigger than 128 bytes (or some chain-specific limit) - not start / end with whitespace
         */
        label: string;
        /**
         * msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
         */
        msg: Binary;
        salt: Binary;
      };
    }
  | {
      migrate: {
        contract_addr: string;
        /**
         * msg is the json-encoded MigrateMsg struct that will be passed to the new code
         */
        msg: Binary;
        /**
         * the code_id of the new logic to place in the given contract
         */
        new_code_id: number;
      };
    }
  | {
      update_admin: {
        admin: string;
        contract_addr: string;
      };
    }
  | {
      clear_admin: {
        contract_addr: string;
      };
    };
/**
 * This message type allows the contract interact with the [x/gov] module in order to cast votes.
 *
 * [x/gov]: https://github.com/cosmos/cosmos-sdk/tree/v0.45.12/x/gov
 *
 * ## Examples
 *
 * Cast a simple vote:
 *
 * ``` # use cosmwasm_std::{ #     HexBinary, #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo, #     Response, QueryResponse, # }; # type ExecuteMsg = (); use cosmwasm_std::{GovMsg, VoteOption};
 *
 * #[entry_point] pub fn execute( deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg, ) -> Result<Response, StdError> { // ... Ok(Response::new().add_message(GovMsg::Vote { proposal_id: 4, option: VoteOption::Yes, })) } ```
 *
 * Cast a weighted vote:
 *
 * ``` # use cosmwasm_std::{ #     HexBinary, #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo, #     Response, QueryResponse, # }; # type ExecuteMsg = (); # #[cfg(feature = "cosmwasm_1_2")] use cosmwasm_std::{Decimal, GovMsg, VoteOption, WeightedVoteOption};
 *
 * # #[cfg(feature = "cosmwasm_1_2")] #[entry_point] pub fn execute( deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg, ) -> Result<Response, StdError> { // ... Ok(Response::new().add_message(GovMsg::VoteWeighted { proposal_id: 4, options: vec![ WeightedVoteOption { option: VoteOption::Yes, weight: Decimal::percent(65), }, WeightedVoteOption { option: VoteOption::Abstain, weight: Decimal::percent(35), }, ], })) } ```
 */
export type GovMsg =
  | {
      vote: {
        /**
         * The vote option.
         *
         * This used to be called "vote", but was changed for consistency with Cosmos SDK.
         */
        option: VoteOption;
        proposal_id: number;
      };
    }
  | {
      vote_weighted: {
        options: WeightedVoteOption[];
        proposal_id: number;
      };
    };
export type VoteOption = "yes" | "no" | "abstain" | "no_with_veto";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
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
  | "accept_ownership"
  | "renounce_ownership";
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

export interface DropFactorySchema {
  responses: OwnershipForString | MapOfString;
  execute: UpdateConfigArgs | ProxyArgs | AdminExecuteArgs | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
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
export interface MapOfString {}
export interface ConfigOptional {
  base_denom?: string | null;
  emergency_address?: string | null;
  factory_contract?: string | null;
  idle_min_interval?: number | null;
  pump_ica_address?: string | null;
  remote_denom?: string | null;
  rewards_receiver?: string | null;
  unbond_batch_switch_time?: number | null;
  unbonding_period?: number | null;
  unbonding_safe_period?: number | null;
}
export interface ConfigOptional2 {
  provider_proposals_contract?: string | null;
  stats_contract?: string | null;
  val_ref_contract?: string | null;
}
export interface ValidatorData {
  on_top?: Uint128 | null;
  valoper_address: string;
  weight: number;
}
export interface AdminExecuteArgs {
  msgs: CosmosMsgFor_NeutronMsg[];
}
export interface Coin {
  amount: Uint128;
  denom: string;
}
/**
 * IbcFee defines struct for fees that refund the relayer for `SudoMsg` messages submission. Unused fee kind will be returned back to message sender. Please refer to these links for more information: IBC transaction structure - <https://docs.neutron.org/neutron/interchain-txs/messages/#msgsubmittx> General mechanics of fee payments - <https://docs.neutron.org/neutron/feerefunder/overview/#general-mechanics>
 */
export interface IbcFee {
  /**
   * *ack_fee** is an amount of coins to refund relayer for submitting ack message for a particular IBC packet.
   */
  ack_fee: Coin[];
  /**
   * **recv_fee** currently is used for compatibility with ICS-29 interface only and must be set to zero (i.e. 0untrn), because Neutron's fee module can't refund relayer for submission of Recv IBC packets due to compatibility with target chains.
   */
  recv_fee: Coin[];
  /**
   * *timeout_fee** amount of coins to refund relayer for submitting timeout message for a particular IBC packet.
   */
  timeout_fee: Coin[];
}
/**
 * Type for wrapping any protobuf message
 */
export interface ProtobufAny {
  /**
   * *type_url** describes the type of the serialized message
   */
  type_url: string;
  /**
   * *value** must be a valid serialized protocol buffer of the above specified type
   */
  value: Binary;
}
/**
 * Describes a KV key for which you want to get value from the storage on remote chain
 */
export interface KVKey {
  /**
   * *key** is a key you want to read from the storage
   */
  key: Binary;
  /**
   * *path** is a path to the storage (storage prefix) where you want to read value by key (usually name of cosmos-packages module: 'staking', 'bank', etc.)
   */
  path: string;
}
export interface RequestPacketTimeoutHeight {
  revision_height?: number | null;
  revision_number?: number | null;
}
/**
 * ParamChangeProposal defines the struct for single parameter change proposal.
 */
export interface ParamChangeProposal {
  /**
   * *description** is a text description of proposal. Non unique.
   */
  description: string;
  /**
   * *param_changes** is a vector of params to be changed. Non unique.
   */
  param_changes: ParamChange[];
  /**
   * *title** is a text title of proposal. Non unique.
   */
  title: string;
}
/**
 * ParamChange defines the struct for parameter change request.
 */
export interface ParamChange {
  /**
   * *key** is a name of parameter. Unique for subspace.
   */
  key: string;
  /**
   * *subspace** is a key of module to which the parameter to change belongs. Unique for each module.
   */
  subspace: string;
  /**
   * *value** is a new value for given parameter. Non unique.
   */
  value: string;
}
/**
 * @deprecated
 * UpgradeProposal defines the struct for IBC upgrade proposal.
 */
export interface UpgradeProposal {
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * *plan** is a plan of upgrade.
   */
  plan: Plan;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
  /**
   * *upgraded_client_state** is an upgraded client state.
   */
  upgraded_client_state: ProtobufAny;
}
/**
 * Plan defines the struct for planned upgrade.
 */
export interface Plan {
  /**
   * *height** is a height at which the upgrade must be performed
   */
  height: number;
  /**
   * *info** is any application specific upgrade info to be included on-chain
   */
  info: string;
  /**
   * *name** is a name for the upgrade
   */
  name: string;
}
/**
 * @deprecated
 * ClientUpdateProposal defines the struct for client update proposal.
 */
export interface ClientUpdateProposal {
  /**
   * *description** is a text description of proposal. Non unique.
   */
  description: string;
  /**
   * *subject_client_id** is a subject client id.
   */
  subject_client_id: string;
  /**
   * *substitute_client_id** is a substitute client id.
   */
  substitute_client_id: string;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * ProposalExecuteMessage defines the struct for sdk47 compatible admin proposal.
 */
export interface ProposalExecuteMessage {
  /**
   * *message** is a json representing an sdk message passed to admin module to execute.
   */
  message: string;
}
/**
 * @deprecated
 * Deprecated. SoftwareUpgradeProposal defines the struct for software upgrade proposal.
 */
export interface SoftwareUpgradeProposal {
  /**
   * *description** is a text description of proposal. Non unique.
   */
  description: string;
  /**
   * *plan** is a plan of upgrade.
   */
  plan: Plan;
  /**
   * *title** is a text title of proposal. Non unique.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. CancelSoftwareUpgradeProposal defines the struct for cancel software upgrade proposal.
 */
export interface CancelSoftwareUpgradeProposal {
  /**
   * *description** is a text description of proposal. Non unique.
   */
  description: string;
  /**
   * *title** is a text title of proposal. Non unique.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. PinCodesProposal defines the struct for pin contract codes proposal.
 */
export interface PinCodesProposal {
  /**
   * *code_ids** is an array of codes to be pined.
   */
  code_ids: number[];
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. UnpinCodesProposal defines the struct for unpin contract codes proposal.
 */
export interface UnpinCodesProposal {
  /**
   * *code_ids** is an array of codes to be unpined.
   */
  code_ids: number[];
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. SudoContractProposal defines the struct for sudo execution proposal.
 */
export interface SudoContractProposal {
  /**
   * *contract** is an address of contract to be executed.
   */
  contract: string;
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * **msg*** is a sudo message.
   */
  msg: Binary;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. UpdateAdminProposal defines the struct for update admin proposal.
 */
export interface UpdateAdminProposal {
  /**
   * *contract** is an address of contract to update admin.
   */
  contract: string;
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * **new_admin*** is an address of new admin
   */
  new_admin: string;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * @deprecated
 * Deprecated. SudoContractProposal defines the struct for clear admin proposal.
 */
export interface ClearAdminProposal {
  /**
   * *contract** is an address of contract admin will be removed.
   */
  contract: string;
  /**
   * *description** is a text description of proposal.
   */
  description: string;
  /**
   * *title** is a text title of proposal.
   */
  title: string;
}
/**
 * Replicates the cosmos-sdk bank module DenomUnit type
 */
export interface DenomUnit {
  aliases: string[];
  denom: string;
  exponent: number;
}
/**
 * MsgExecuteContract defines a call to the contract execution
 */
export interface MsgExecuteContract {
  /**
   * *contract** is a contract address that will be called
   */
  contract: string;
  /**
   * *msg** is a contract call message
   */
  msg: string;
}
export interface DepositOption {
  disable_swap: boolean;
}
export interface PrecDec {
  i: string;
}
export interface MultiHopRoute {
  hops: string[];
}
/**
 * A message encoded the same way as a protobuf [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto). This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
 */
export interface AnyMsg {
  type_url: string;
  value: Binary;
}
/**
 * In IBC each package must set at least one type of timeout: the timestamp or the block height. Using this rather complex enum instead of two timeout fields we ensure that at least one timeout is set.
 */
export interface IbcTimeout {
  block?: IbcTimeoutBlock | null;
  timestamp?: Timestamp | null;
}
/**
 * IBCTimeoutHeight Height is a monotonically increasing data type that can be compared against another Height for the purposes of updating and freezing clients. Ordering is (revision_number, timeout_height)
 */
export interface IbcTimeoutBlock {
  /**
   * block height after which the packet times out. the height within the given revision
   */
  height: number;
  /**
   * the version that the client is currently on (e.g. after resetting the chain this could increment 1 as height drops to 0)
   */
  revision: number;
}
export interface WeightedVoteOption {
  option: VoteOption;
  weight: Decimal;
}
export interface InstantiateMsg {
  base_denom: string;
  code_ids: CodeIds;
  core_params: CoreParams;
  fee_params?: FeeParams | null;
  local_denom: string;
  pre_instantiated_contracts: PreInstantiatedContracts;
  remote_opts: RemoteOpts;
  salt: string;
  subdenom: string;
  token_metadata: DenomMetadata;
}
export interface CodeIds {
  core_code_id: number;
  distribution_code_id: number;
  rewards_manager_code_id: number;
  splitter_code_id: number;
  strategy_code_id: number;
  token_code_id: number;
  validators_set_code_id: number;
  withdrawal_manager_code_id: number;
  withdrawal_voucher_code_id: number;
}
export interface CoreParams {
  icq_update_delay: number;
  idle_min_interval: number;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
}
export interface FeeParams {
  fee: Decimal;
  fee_address: string;
}
export interface PreInstantiatedContracts {
  lsm_share_bond_provider_address?: Addr | null;
  native_bond_provider_address: Addr;
  puppeteer_address: Addr;
  rewards_pump_address?: Addr | null;
  unbonding_pump_address?: Addr | null;
  val_ref_address?: Addr | null;
}
export interface RemoteOpts {
  connection_id: string;
  denom: string;
  timeout: Timeout;
  transfer_channel_id: string;
}
export interface Timeout {
  local: number;
  remote: number;
}
export interface DenomMetadata {
  /**
   * Even longer description, example: "The native staking token of the Cosmos Hub"
   */
  description: string;
  /**
   * Lowercase moniker to be displayed in clients, example: "atom"
   */
  display: string;
  /**
   * Number of decimals
   */
  exponent: number;
  /**
   * Descriptive token name, example: "Cosmos Hub Atom"
   */
  name: string;
  /**
   * Symbol to be displayed on exchanges, example: "ATOM"
   */
  symbol: string;
  /**
   * URI to a document that contains additional information
   */
  uri?: string | null;
  /**
   * SHA256 hash of a document pointed by URI
   */
  uri_hash?: string | null;
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
  mustBeSigningClient(): Error {
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
    admin?: string,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
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
    const res = await client.instantiate2(sender, codeId, salt, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
    });
    return res;
  }
  queryState = async(): Promise<MapOfString> => {
    return this.client.queryContractSmart(this.contractAddress, { state: {} });
  }
  queryOwnership = async(): Promise<OwnershipForString> => {
    return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.updateConfigMsg(args), fee || "auto", memo, funds);
  }
  updateConfigMsg = (args: UpdateConfigArgs): { update_config: UpdateConfigArgs } => { return { update_config: args }; }
  proxy = async(sender:string, args: ProxyArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.proxyMsg(args), fee || "auto", memo, funds);
  }
  proxyMsg = (args: ProxyArgs): { proxy: ProxyArgs } => { return { proxy: args }; }
  adminExecute = async(sender:string, args: AdminExecuteArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.adminExecuteMsg(args), fee || "auto", memo, funds);
  }
  adminExecuteMsg = (args: AdminExecuteArgs): { admin_execute: AdminExecuteArgs } => { return { admin_execute: args }; }
  updateOwnership = async(sender:string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
  }
  updateOwnershipMsg = (args: UpdateOwnershipArgs): { update_ownership: UpdateOwnershipArgs } => { return { update_ownership: args }; }
}
