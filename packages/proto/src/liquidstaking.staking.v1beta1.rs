// @generated
/// StakeAuthorization defines authorization for delegate/undelegate/redelegate.
// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Clone, PartialEq, ::prost::Message)]
// pub struct StakeAuthorization {
//     /// max_tokens specifies the maximum amount of tokens can be delegate to a validator. If it is
//     /// empty, there is no spend limit and any amount of coins can be delegated.
//     #[prost(message, optional, tag = "1")]
//     pub max_tokens: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
//     /// authorization_type defines one of AuthorizationType.
//     #[prost(enumeration = "AuthorizationType", tag = "4")]
//     pub authorization_type: i32,
//     /// validators is the oneof that represents either allow_list or deny_list
//     #[prost(oneof = "stake_authorization::Validators", tags = "2, 3")]
//     pub validators: ::core::option::Option<stake_authorization::Validators>,
// }
/// Nested message and enum types in `StakeAuthorization`.
// pub mod stake_authorization {
//     /// Validators defines list of validator addresses.
//     #[allow(clippy::derive_partial_eq_without_eq)]
//     #[derive(Clone, PartialEq, ::prost::Message)]
//     pub struct Validators {
//         #[prost(string, repeated, tag = "1")]
//         pub address: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
//     }
//     /// validators is the oneof that represents either allow_list or deny_list
//     #[allow(clippy::derive_partial_eq_without_eq)]
//     #[derive(Clone, PartialEq, ::prost::Oneof)]
//     pub enum Validators {
//         /// allow_list specifies list of validator addresses to whom grantee can delegate tokens on behalf of granter's
//         /// account.
//         #[prost(message, tag = "2")]
//         AllowList(Validators),
//         /// deny_list specifies list of validator addresses to whom grantee can not delegate tokens.
//         #[prost(message, tag = "3")]
//         DenyList(Validators),
//     }
// }
/// AuthorizationType defines the type of staking module authorization type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AuthorizationType {
    /// AUTHORIZATION_TYPE_UNSPECIFIED specifies an unknown authorization type
    Unspecified = 0,
    /// AUTHORIZATION_TYPE_DELEGATE defines an authorization type for Msg/Delegate
    Delegate = 1,
    /// AUTHORIZATION_TYPE_UNDELEGATE defines an authorization type for Msg/Undelegate
    Undelegate = 2,
    /// AUTHORIZATION_TYPE_REDELEGATE defines an authorization type for Msg/BeginRedelegate
    Redelegate = 3,
}
impl AuthorizationType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            AuthorizationType::Unspecified => "AUTHORIZATION_TYPE_UNSPECIFIED",
            AuthorizationType::Delegate => "AUTHORIZATION_TYPE_DELEGATE",
            AuthorizationType::Undelegate => "AUTHORIZATION_TYPE_UNDELEGATE",
            AuthorizationType::Redelegate => "AUTHORIZATION_TYPE_REDELEGATE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AUTHORIZATION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "AUTHORIZATION_TYPE_DELEGATE" => Some(Self::Delegate),
            "AUTHORIZATION_TYPE_UNDELEGATE" => Some(Self::Undelegate),
            "AUTHORIZATION_TYPE_REDELEGATE" => Some(Self::Redelegate),
            _ => None,
        }
    }
}
// NOTE: After generating the protos, the `BondStatus` field in the `Validator` struct
// should be modified in `staking.pb.go` so that it leverages the type from the sdk
// instead of LSM

/// HistoricalInfo contains header and validator information for a given block.
/// It is stored as part of staking module's state, which persists the `n` most
/// recent HistoricalInfo
/// (`n` is set by the staking module's `historical_entries` parameter).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HistoricalInfo {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<tendermint_proto::types::Header>,
    #[prost(message, repeated, tag = "2")]
    pub valset: ::prost::alloc::vec::Vec<Validator>,
}
/// CommissionRates defines the initial commission rates to be used for creating
/// a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommissionRates {
    /// rate is the commission rate charged to delegators, as a fraction.
    #[prost(string, tag = "1")]
    pub rate: ::prost::alloc::string::String,
    /// max_rate defines the maximum commission rate which validator can ever charge, as a fraction.
    #[prost(string, tag = "2")]
    pub max_rate: ::prost::alloc::string::String,
    /// max_change_rate defines the maximum daily increase of the validator commission, as a fraction.
    #[prost(string, tag = "3")]
    pub max_change_rate: ::prost::alloc::string::String,
}
/// Commission defines commission parameters for a given validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commission {
    /// commission_rates defines the initial commission rates to be used for creating a validator.
    #[prost(message, optional, tag = "1")]
    pub commission_rates: ::core::option::Option<CommissionRates>,
    /// update_time is the last time the commission rate was changed.
    #[prost(message, optional, tag = "2")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Description defines a validator description.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Description {
    /// moniker defines a human-readable name for the validator.
    #[prost(string, tag = "1")]
    pub moniker: ::prost::alloc::string::String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string, tag = "2")]
    pub identity: ::prost::alloc::string::String,
    /// website defines an optional website link.
    #[prost(string, tag = "3")]
    pub website: ::prost::alloc::string::String,
    /// security_contact defines an optional email for security contact.
    #[prost(string, tag = "4")]
    pub security_contact: ::prost::alloc::string::String,
    /// details define other optional details.
    #[prost(string, tag = "5")]
    pub details: ::prost::alloc::string::String,
}
/// Validator defines a validator, together with the total amount of the
/// Validator's bond shares and their exchange rate to coins. Slashing results in
/// a decrease in the exchange rate, allowing correct calculation of future
/// undelegations without iterating over delegators. When coins are delegated to
/// this validator, the validator is credited with a delegation whose number of
/// bond shares is based on the amount of coins delegated divided by the current
/// exchange rate. Voting power can be calculated as total bonded shares
/// multiplied by exchange rate.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    /// operator_address defines the address of the validator's operator; bech encoded in JSON.
    #[prost(string, tag = "1")]
    pub operator_address: ::prost::alloc::string::String,
    /// consensus_pubkey is the consensus public key of the validator, as a Protobuf Any.
    #[prost(message, optional, tag = "2")]
    pub consensus_pubkey: ::core::option::Option<::prost_types::Any>,
    /// jailed defined whether the validator has been jailed from bonded status or not.
    #[prost(bool, tag = "3")]
    pub jailed: bool,
    /// status is the validator status (bonded/unbonding/unbonded).
    #[prost(enumeration = "BondStatus", tag = "4")]
    pub status: i32,
    /// tokens define the delegated tokens (incl. self-delegation).
    #[prost(string, tag = "5")]
    pub tokens: ::prost::alloc::string::String,
    /// delegator_shares defines total shares issued to a validator's delegators.
    #[prost(string, tag = "6")]
    pub delegator_shares: ::prost::alloc::string::String,
    /// description defines the description terms for the validator.
    #[prost(message, optional, tag = "7")]
    pub description: ::core::option::Option<Description>,
    /// unbonding_height defines, if unbonding, the height at which this validator has begun unbonding.
    #[prost(int64, tag = "8")]
    pub unbonding_height: i64,
    /// unbonding_time defines, if unbonding, the min time for the validator to complete unbonding.
    #[prost(message, optional, tag = "9")]
    pub unbonding_time: ::core::option::Option<::prost_types::Timestamp>,
    /// commission defines the commission parameters.
    #[prost(message, optional, tag = "10")]
    pub commission: ::core::option::Option<Commission>,
    /// Number of shares self bonded from the validator
    #[prost(string, tag = "11")]
    pub total_validator_bond_shares: ::prost::alloc::string::String,
    /// Total number of shares either tokenized or owned by a liquid staking provider
    #[prost(string, tag = "12")]
    pub total_liquid_shares: ::prost::alloc::string::String,
}
/// ValAddresses defines a repeated set of validator addresses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValAddresses {
    #[prost(string, repeated, tag = "1")]
    pub addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// DVPair is struct that just has a delegator-validator pair with no other data.
/// It is intended to be used as a marshalable pointer. For example, a DVPair can
/// be used to construct the key to getting an UnbondingDelegation from state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvPair {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
/// DVPairs defines an array of DVPair objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvPairs {
    #[prost(message, repeated, tag = "1")]
    pub pairs: ::prost::alloc::vec::Vec<DvPair>,
}
/// DVVTriplet is struct that just has a delegator-validator-validator triplet
/// with no other data. It is intended to be used as a marshalable pointer. For
/// example, a DVVTriplet can be used to construct the key to getting a
/// Redelegation from state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvvTriplet {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
}
/// DVVTriplets defines an array of DVVTriplet objects.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DvvTriplets {
    #[prost(message, repeated, tag = "1")]
    pub triplets: ::prost::alloc::vec::Vec<DvvTriplet>,
}
/// Delegation represents the bond with tokens held by an account. It is
/// owned by one delegator, and is associated with the voting power of one
/// validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delegation {
    /// delegator_address is the bech32-encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address is the bech32-encoded address of the validator.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// shares define the delegation shares received.
    #[prost(string, tag = "3")]
    pub shares: ::prost::alloc::string::String,
    /// has this delegation been marked as a validator self bond.
    #[prost(bool, tag = "4")]
    pub validator_bond: bool,
}
/// UnbondingDelegation stores all of a single delegator's unbonding bonds
/// for a single validator in an time-ordered list.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnbondingDelegation {
    /// delegator_address is the bech32-encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address is the bech32-encoded address of the validator.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// entries are the unbonding delegation entries.
    ///
    /// unbonding delegation entries
    #[prost(message, repeated, tag = "3")]
    pub entries: ::prost::alloc::vec::Vec<UnbondingDelegationEntry>,
}
/// UnbondingDelegationEntry defines an unbonding object with relevant metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnbondingDelegationEntry {
    /// creation_height is the height which the unbonding took place.
    #[prost(int64, tag = "1")]
    pub creation_height: i64,
    /// completion_time is the unix time for unbonding completion.
    #[prost(message, optional, tag = "2")]
    pub completion_time: ::core::option::Option<::prost_types::Timestamp>,
    /// initial_balance defines the tokens initially scheduled to receive at completion.
    #[prost(string, tag = "3")]
    pub initial_balance: ::prost::alloc::string::String,
    /// balance defines the tokens to receive at completion.
    #[prost(string, tag = "4")]
    pub balance: ::prost::alloc::string::String,
}
/// RedelegationEntry defines a redelegation object with relevant metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationEntry {
    /// creation_height  defines the height which the redelegation took place.
    #[prost(int64, tag = "1")]
    pub creation_height: i64,
    /// completion_time defines the unix time for redelegation completion.
    #[prost(message, optional, tag = "2")]
    pub completion_time: ::core::option::Option<::prost_types::Timestamp>,
    /// initial_balance defines the initial balance when redelegation started.
    #[prost(string, tag = "3")]
    pub initial_balance: ::prost::alloc::string::String,
    /// shares_dst is the amount of destination-validator shares created by redelegation.
    #[prost(string, tag = "4")]
    pub shares_dst: ::prost::alloc::string::String,
}
/// Redelegation contains the list of a particular delegator's redelegating bonds
/// from a particular source validator to a particular destination validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Redelegation {
    /// delegator_address is the bech32-encoded address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_src_address is the validator redelegation source operator address.
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    /// validator_dst_address is the validator redelegation destination operator address.
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
    /// entries are the redelegation entries.
    ///
    /// redelegation entries
    #[prost(message, repeated, tag = "4")]
    pub entries: ::prost::alloc::vec::Vec<RedelegationEntry>,
}
/// Params defines the parameters for the staking module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// unbonding_time is the time duration of unbonding.
    #[prost(message, optional, tag = "1")]
    pub unbonding_time: ::core::option::Option<::prost_types::Duration>,
    /// max_validators is the maximum number of validators.
    #[prost(uint32, tag = "2")]
    pub max_validators: u32,
    /// max_entries is the max entries for either unbonding delegation or redelegation (per pair/trio).
    #[prost(uint32, tag = "3")]
    pub max_entries: u32,
    /// historical_entries is the number of historical entries to persist.
    #[prost(uint32, tag = "4")]
    pub historical_entries: u32,
    /// bond_denom defines the bondable coin denomination.
    #[prost(string, tag = "5")]
    pub bond_denom: ::prost::alloc::string::String,
    /// min_commission_rate is the chain-wide minimum commission rate that a validator can charge their delegators
    #[prost(string, tag = "6")]
    pub min_commission_rate: ::prost::alloc::string::String,
    /// validator_bond_factor is required as a safety check for tokenizing shares and
    /// delegations from liquid staking providers
    #[prost(string, tag = "7")]
    pub validator_bond_factor: ::prost::alloc::string::String,
    /// global_liquid_staking_cap represents a cap on the portion of stake that
    /// comes from liquid staking providers
    #[prost(string, tag = "8")]
    pub global_liquid_staking_cap: ::prost::alloc::string::String,
}
/// DelegationResponse is equivalent to Delegation except that it contains a
/// balance in addition to shares which is more suitable for client responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationResponse {
    #[prost(message, optional, tag = "1")]
    pub delegation: ::core::option::Option<Delegation>,
    #[prost(message, optional, tag = "2")]
    pub balance: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// RedelegationEntryResponse is equivalent to a RedelegationEntry except that it
/// contains a balance in addition to shares which is more suitable for client
/// responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationEntryResponse {
    #[prost(message, optional, tag = "1")]
    pub redelegation_entry: ::core::option::Option<RedelegationEntry>,
    #[prost(string, tag = "4")]
    pub balance: ::prost::alloc::string::String,
}
/// RedelegationResponse is equivalent to a Redelegation except that its entries
/// contain a balance in addition to shares which is more suitable for client
/// responses.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RedelegationResponse {
    #[prost(message, optional, tag = "1")]
    pub redelegation: ::core::option::Option<Redelegation>,
    #[prost(message, repeated, tag = "2")]
    pub entries: ::prost::alloc::vec::Vec<RedelegationEntryResponse>,
}
/// Pool is used for tracking bonded and not-bonded token supply of the bond
/// denomination.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pool {
    #[prost(string, tag = "1")]
    pub not_bonded_tokens: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub bonded_tokens: ::prost::alloc::string::String,
}
/// TokenizeShareRecord represents a tokenized delegation
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TokenizeShareRecord {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    /// module account take the role of delegator
    #[prost(string, tag = "3")]
    pub module_account: ::prost::alloc::string::String,
    /// validator delegated to for tokenize share record creation
    #[prost(string, tag = "4")]
    pub validator: ::prost::alloc::string::String,
}
/// BondStatus is the status of a validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BondStatus {
    /// UNSPECIFIED defines an invalid validator status.
    Unspecified = 0,
    /// UNBONDED defines a validator that is not bonded.
    Unbonded = 1,
    /// UNBONDING defines a validator that is unbonding.
    Unbonding = 2,
    /// BONDED defines a validator that is bonded.
    Bonded = 3,
}
impl BondStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BondStatus::Unspecified => "BOND_STATUS_UNSPECIFIED",
            BondStatus::Unbonded => "BOND_STATUS_UNBONDED",
            BondStatus::Unbonding => "BOND_STATUS_UNBONDING",
            BondStatus::Bonded => "BOND_STATUS_BONDED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BOND_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "BOND_STATUS_UNBONDED" => Some(Self::Unbonded),
            "BOND_STATUS_UNBONDING" => Some(Self::Unbonding),
            "BOND_STATUS_BONDED" => Some(Self::Bonded),
            _ => None,
        }
    }
}
/// GenesisState defines the staking module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// params defines all the paramaters of related to deposit.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// last_total_power tracks the total amounts of bonded tokens recorded during
    /// the previous end block.
    #[prost(bytes = "vec", tag = "2")]
    pub last_total_power: ::prost::alloc::vec::Vec<u8>,
    /// last_validator_powers is a special index that provides a historical list
    /// of the last-block's bonded validators.
    #[prost(message, repeated, tag = "3")]
    pub last_validator_powers: ::prost::alloc::vec::Vec<LastValidatorPower>,
    /// delegations defines the validator set at genesis.
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// delegations defines the delegations active at genesis.
    #[prost(message, repeated, tag = "5")]
    pub delegations: ::prost::alloc::vec::Vec<Delegation>,
    /// unbonding_delegations defines the unbonding delegations active at genesis.
    #[prost(message, repeated, tag = "6")]
    pub unbonding_delegations: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// redelegations defines the redelegations active at genesis.
    #[prost(message, repeated, tag = "7")]
    pub redelegations: ::prost::alloc::vec::Vec<Redelegation>,
    #[prost(bool, tag = "8")]
    pub exported: bool,
    /// store tokenize share records to provide reward to record owners
    #[prost(message, repeated, tag = "9")]
    pub tokenize_share_records: ::prost::alloc::vec::Vec<TokenizeShareRecord>,
    /// last tokenize share record id, used for next share record id calculation
    #[prost(uint64, tag = "10")]
    pub last_tokenize_share_record_id: u64,
}
/// LastValidatorPower required for validator set update logic.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LastValidatorPower {
    /// address is the address of the validator.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// power defines the power of the validator.
    #[prost(int64, tag = "2")]
    pub power: i64,
}
/// QueryValidatorsRequest is request type for Query/Validators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorsRequest {
    /// status enables to query for validators matching a given status.
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryValidatorsResponse is response type for the Query/Validators RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorsResponse {
    /// validators contains all the queried validators.
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryValidatorRequest is response type for the Query/Validator RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
}
/// QueryValidatorResponse is response type for the Query/Validator RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorResponse {
    /// validator defines the validator info.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
}
/// QueryValidatorDelegationsRequest is request type for the
/// Query/ValidatorDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDelegationsRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryValidatorDelegationsResponse is response type for the
/// Query/ValidatorDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub delegation_responses: ::prost::alloc::vec::Vec<DelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryValidatorUnbondingDelegationsRequest is required type for the
/// Query/ValidatorUnbondingDelegations RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorUnbondingDelegationsRequest {
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryValidatorUnbondingDelegationsResponse is response type for the
/// Query/ValidatorUnbondingDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorUnbondingDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub unbonding_responses: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryDelegationRequest is request type for the Query/Delegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
/// QueryDelegationResponse is response type for the Query/Delegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationResponse {
    /// delegation_responses defines the delegation info of a delegation.
    #[prost(message, optional, tag = "1")]
    pub delegation_response: ::core::option::Option<DelegationResponse>,
}
/// QueryUnbondingDelegationRequest is request type for the
/// Query/UnbondingDelegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnbondingDelegationRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
/// QueryDelegationResponse is response type for the Query/UnbondingDelegation
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUnbondingDelegationResponse {
    /// unbond defines the unbonding information of a delegation.
    #[prost(message, optional, tag = "1")]
    pub unbond: ::core::option::Option<UnbondingDelegation>,
}
/// QueryDelegatorDelegationsRequest is request type for the
/// Query/DelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorDelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryDelegatorDelegationsResponse is response type for the
/// Query/DelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorDelegationsResponse {
    /// delegation_responses defines all the delegations' info of a delegator.
    #[prost(message, repeated, tag = "1")]
    pub delegation_responses: ::prost::alloc::vec::Vec<DelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryDelegatorUnbondingDelegationsRequest is request type for the
/// Query/DelegatorUnbondingDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorUnbondingDelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryUnbondingDelegatorDelegationsResponse is response type for the
/// Query/UnbondingDelegatorDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorUnbondingDelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub unbonding_responses: ::prost::alloc::vec::Vec<UnbondingDelegation>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryRedelegationsRequest is request type for the Query/Redelegations RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRedelegationsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// src_validator_addr defines the validator address to redelegate from.
    #[prost(string, tag = "2")]
    pub src_validator_addr: ::prost::alloc::string::String,
    /// dst_validator_addr defines the validator address to redelegate to.
    #[prost(string, tag = "3")]
    pub dst_validator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryRedelegationsResponse is response type for the Query/Redelegations RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRedelegationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub redelegation_responses: ::prost::alloc::vec::Vec<RedelegationResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryDelegatorValidatorsRequest is request type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryDelegatorValidatorsResponse is response type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsResponse {
    /// validators defines the validators' info of a delegator.
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryDelegatorValidatorRequest is request type for the
/// Query/DelegatorValidator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorRequest {
    /// delegator_addr defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_addr: ::prost::alloc::string::String,
    /// validator_addr defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_addr: ::prost::alloc::string::String,
}
/// QueryDelegatorValidatorResponse response type for the
/// Query/DelegatorValidator RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorResponse {
    /// validator defines the validator info.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<Validator>,
}
/// QueryHistoricalInfoRequest is request type for the Query/HistoricalInfo RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHistoricalInfoRequest {
    /// height defines at which height to query the historical info.
    #[prost(int64, tag = "1")]
    pub height: i64,
}
/// QueryHistoricalInfoResponse is response type for the Query/HistoricalInfo RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryHistoricalInfoResponse {
    /// hist defines the historical info at the given height.
    #[prost(message, optional, tag = "1")]
    pub hist: ::core::option::Option<HistoricalInfo>,
}
/// QueryPoolRequest is request type for the Query/Pool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPoolRequest {}
/// QueryPoolResponse is response type for the Query/Pool RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryPoolResponse {
    /// pool defines the pool info.
    #[prost(message, optional, tag = "1")]
    pub pool: ::core::option::Option<Pool>,
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// QueryTokenizeShareRecordByIdRequest is request type for the
/// Query/QueryTokenizeShareRecordById RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordByIdRequest {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
/// QueryTokenizeShareRecordByIdRequest is response type for the
/// Query/QueryTokenizeShareRecordById RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub record: ::core::option::Option<TokenizeShareRecord>,
}
/// QueryTokenizeShareRecordByDenomRequest is request type for the
/// Query/QueryTokenizeShareRecordByDenom RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordByDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
/// QueryTokenizeShareRecordByDenomResponse is response type for the
/// Query/QueryTokenizeShareRecordByDenom RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordByDenomResponse {
    #[prost(message, optional, tag = "1")]
    pub record: ::core::option::Option<TokenizeShareRecord>,
}
/// QueryTokenizeShareRecordsOwnedRequest is request type for the
/// Query/QueryTokenizeShareRecordsOwned RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordsOwnedRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
}
/// QueryTokenizeShareRecordsOwnedResponse is response type for the
/// Query/QueryTokenizeShareRecordsOwned RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTokenizeShareRecordsOwnedResponse {
    #[prost(message, repeated, tag = "1")]
    pub records: ::prost::alloc::vec::Vec<TokenizeShareRecord>,
}
/// QueryAllTokenizeShareRecordsRequest is request type for the
/// Query/QueryAllTokenizeShareRecords RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllTokenizeShareRecordsRequest {}
/// QueryAllTokenizeShareRecordsResponse is response type for the
/// Query/QueryAllTokenizeShareRecords RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryAllTokenizeShareRecordsResponse {
    #[prost(message, repeated, tag = "1")]
    pub records: ::prost::alloc::vec::Vec<TokenizeShareRecord>,
}
/// QueryLastTokenizeShareRecordIdRequest is request type for the
/// Query/QueryLastTokenizeShareRecordId RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLastTokenizeShareRecordIdRequest {}
/// QueryLastTokenizeShareRecordIdResponse is response type for the
/// Query/QueryLastTokenizeShareRecordId RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLastTokenizeShareRecordIdResponse {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}
/// QueryTotalTokenizeSharedAssetsRequest is request type for the
/// Query/QueryTotalTokenizeSharedAssets RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTokenizeSharedAssetsRequest {}
/// QueryTotalTokenizeSharedAssetsResponse is response type for the
/// Query/QueryTotalTokenizeSharedAssets RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTotalTokenizeSharedAssetsResponse {
    #[prost(message, optional, tag = "1")]
    pub value: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgCreateValidator defines a SDK message for creating a new validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateValidator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<Description>,
    #[prost(message, optional, tag = "2")]
    pub commission: ::core::option::Option<CommissionRates>,
    #[prost(string, tag = "3")]
    pub min_self_delegation: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "6")]
    pub pubkey: ::core::option::Option<::prost_types::Any>,
    #[prost(message, optional, tag = "7")]
    pub value: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgCreateValidatorResponse defines the Msg/CreateValidator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateValidatorResponse {}
/// MsgEditValidator defines a SDK message for editing an existing validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditValidator {
    #[prost(message, optional, tag = "1")]
    pub description: ::core::option::Option<Description>,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// We pass a reference to the new commission rate and min self delegation as
    /// it's not mandatory to update. If not updated, the deserialized rate will be
    /// zero with no way to distinguish if an update was intended.
    /// REF: #2373
    #[prost(string, tag = "3")]
    pub commission_rate: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub min_self_delegation: ::prost::alloc::string::String,
}
/// MsgEditValidatorResponse defines the Msg/EditValidator response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgEditValidatorResponse {}
/// MsgDelegate defines a SDK message for performing a delegation of coins
/// from a delegator to a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgDelegateResponse defines the Msg/Delegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDelegateResponse {}
/// MsgBeginRedelegate defines a SDK message for performing a redelegation
/// of coins from a delegator and source validator to a destination validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginRedelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_src_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub validator_dst_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgBeginRedelegateResponse defines the Msg/BeginRedelegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgBeginRedelegateResponse {
    #[prost(message, optional, tag = "1")]
    pub completion_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// MsgUndelegate defines a SDK message for performing an undelegation from a
/// delegate and a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegate {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgUndelegateResponse defines the Msg/Undelegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUndelegateResponse {
    #[prost(message, optional, tag = "1")]
    pub completion_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// MsgUnbondValidator defines a method for performing the status transition for
/// a validator from bonded to unbonded
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnbondValidator {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnbondValidatorResponse {}
/// MsgCancelUnbondingDelegation defines the SDK message for performing a cancel unbonding delegation for delegator
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUnbondingDelegation {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// amount is always less than or equal to unbonding delegation entry balance
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    /// creation_height is the height which the unbonding took place.
    #[prost(int64, tag = "4")]
    pub creation_height: i64,
}
/// MsgCancelUnbondingDelegationResponse
///
/// Since: cosmos-sdk 0.46
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelUnbondingDelegationResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTokenizeShares {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "4")]
    pub tokenized_share_owner: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTokenizeSharesResponse {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRedeemTokensforShares {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRedeemTokensforSharesResponse {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransferTokenizeShareRecord {
    #[prost(uint64, tag = "1")]
    pub tokenize_share_record_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub new_owner: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgTransferTokenizeShareRecordResponse {}
/// MsgValidatorBond defines a SDK message for performing validator self-bond of delegated coins
/// from a delegator to a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgValidatorBond {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgValidatorBondResponse {}
include!("liquidstaking.staking.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)