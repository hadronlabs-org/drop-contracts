// @generated
/// Params defines the set of params for the distribution module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    #[prost(string, tag = "1")]
    pub community_tax: ::prost::alloc::string::String,
    /// The base_proposer_reward and bonus_proposer_reward fields are deprecated
    /// and are no longer used in the x/distribution module's reward mechanism.
    #[deprecated]
    #[prost(string, tag = "2")]
    pub base_proposer_reward: ::prost::alloc::string::String,
    #[deprecated]
    #[prost(string, tag = "3")]
    pub bonus_proposer_reward: ::prost::alloc::string::String,
    #[prost(bool, tag = "4")]
    pub withdraw_addr_enabled: bool,
}
/// ValidatorHistoricalRewards represents historical rewards for a validator.
/// Height is implicit within the store key.
/// Cumulative reward ratio is the sum from the zeroeth period
/// until this period of rewards / tokens, per the spec.
/// The reference count indicates the number of objects
/// which might need to reference this historical entry at any point.
/// ReferenceCount =
///     number of outstanding delegations which ended the associated period (and
///     might need to read that record)
///   + number of slashes which ended the associated period (and might need to
///   read that record)
///   + one per validator for the zeroeth period, set on initialization
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorHistoricalRewards {
    #[prost(message, repeated, tag = "1")]
    pub cumulative_reward_ratio: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    #[prost(uint32, tag = "2")]
    pub reference_count: u32,
}
/// ValidatorCurrentRewards represents current rewards and current
/// period for a validator kept as a running counter and incremented
/// each block as long as the validator's tokens remain constant.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorCurrentRewards {
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    #[prost(uint64, tag = "2")]
    pub period: u64,
}
/// ValidatorAccumulatedCommission represents accumulated commission
/// for a validator kept as a running counter, can be withdrawn at any time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorAccumulatedCommission {
    #[prost(message, repeated, tag = "1")]
    pub commission: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// ValidatorOutstandingRewards represents outstanding (un-withdrawn) rewards
/// for a validator inexpensive to track, allows simple sanity checks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorOutstandingRewards {
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// ValidatorSlashEvent represents a validator slash event.
/// Height is implicit within the store key.
/// This is needed to calculate appropriate amount of staking tokens
/// for delegations which are withdrawn after a slash has occurred.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSlashEvent {
    #[prost(uint64, tag = "1")]
    pub validator_period: u64,
    #[prost(string, tag = "2")]
    pub fraction: ::prost::alloc::string::String,
}
/// ValidatorSlashEvents is a collection of ValidatorSlashEvent messages.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSlashEvents {
    #[prost(message, repeated, tag = "1")]
    pub validator_slash_events: ::prost::alloc::vec::Vec<ValidatorSlashEvent>,
}
/// FeePool is the global fee pool for distribution.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeePool {
    #[prost(message, repeated, tag = "1")]
    pub community_pool: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// CommunityPoolSpendProposal details a proposal for use of community funds,
/// together with how many coins are proposed to be spent, and to which
/// recipient account.
///
/// Deprecated: Do not use. As of the Cosmos SDK release v0.47.x, there is no
/// longer a need for an explicit CommunityPoolSpendProposal. To spend community
/// pool funds, a simple MsgCommunityPoolSpend can be invoked from the x/gov
/// module via a v1 governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolSpendProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
/// DelegatorStartingInfo represents the starting info for a delegator reward
/// period. It tracks the previous validator period, the delegation's amount of
/// staking token, and the creation height (to check later on if any slashes have
/// occurred). NOTE: Even though validators are slashed to whole staking tokens,
/// the delegators within the validator may be left with less than a full token,
/// thus sdk.Dec is used.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegatorStartingInfo {
    #[prost(uint64, tag = "1")]
    pub previous_period: u64,
    #[prost(string, tag = "2")]
    pub stake: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    pub height: u64,
}
/// DelegationDelegatorReward represents the properties
/// of a delegator's delegation reward.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelegationDelegatorReward {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub reward: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// CommunityPoolSpendProposalWithDeposit defines a CommunityPoolSpendProposal
/// with a deposit
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommunityPoolSpendProposalWithDeposit {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub recipient: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub amount: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub deposit: ::prost::alloc::string::String,
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// QueryValidatorDistributionInfoRequest is the request type for the Query/ValidatorDistributionInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDistributionInfoRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
/// QueryValidatorDistributionInfoResponse is the response type for the Query/ValidatorDistributionInfo RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorDistributionInfoResponse {
    /// operator_address defines the validator operator address.
    #[prost(string, tag = "1")]
    pub operator_address: ::prost::alloc::string::String,
    /// self_bond_rewards defines the self delegations rewards.
    #[prost(message, repeated, tag = "2")]
    pub self_bond_rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
    /// commission defines the commision the validator received.
    #[prost(message, repeated, tag = "3")]
    pub commission: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// QueryValidatorOutstandingRewardsRequest is the request type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorOutstandingRewardsRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
/// QueryValidatorOutstandingRewardsResponse is the response type for the
/// Query/ValidatorOutstandingRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorOutstandingRewardsResponse {
    #[prost(message, optional, tag = "1")]
    pub rewards: ::core::option::Option<ValidatorOutstandingRewards>,
}
/// QueryValidatorCommissionRequest is the request type for the
/// Query/ValidatorCommission RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorCommissionRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
}
/// QueryValidatorCommissionResponse is the response type for the
/// Query/ValidatorCommission RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorCommissionResponse {
    /// commission defines the commision the validator received.
    #[prost(message, optional, tag = "1")]
    pub commission: ::core::option::Option<ValidatorAccumulatedCommission>,
}
/// QueryValidatorSlashesRequest is the request type for the
/// Query/ValidatorSlashes RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorSlashesRequest {
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// starting_height defines the optional starting height to query the slashes.
    #[prost(uint64, tag = "2")]
    pub starting_height: u64,
    /// starting_height defines the optional ending height to query the slashes.
    #[prost(uint64, tag = "3")]
    pub ending_height: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
/// QueryValidatorSlashesResponse is the response type for the
/// Query/ValidatorSlashes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryValidatorSlashesResponse {
    /// slashes defines the slashes the validator received.
    #[prost(message, repeated, tag = "1")]
    pub slashes: ::prost::alloc::vec::Vec<ValidatorSlashEvent>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
/// QueryDelegationRewardsRequest is the request type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    /// validator_address defines the validator address to query for.
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
}
/// QueryDelegationRewardsResponse is the response type for the
/// Query/DelegationRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationRewardsResponse {
    /// rewards defines the rewards accrued by a delegation.
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// QueryDelegationTotalRewardsRequest is the request type for the
/// Query/DelegationTotalRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationTotalRewardsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
/// QueryDelegationTotalRewardsResponse is the response type for the
/// Query/DelegationTotalRewards RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegationTotalRewardsResponse {
    /// rewards defines all the rewards accrued by a delegator.
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<DelegationDelegatorReward>,
    /// total defines the sum of all the rewards.
    #[prost(message, repeated, tag = "2")]
    pub total: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
/// QueryDelegatorValidatorsRequest is the request type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
/// QueryDelegatorValidatorsResponse is the response type for the
/// Query/DelegatorValidators RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorValidatorsResponse {
    /// validators defines the validators a delegator is delegating for.
    #[prost(string, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// QueryDelegatorWithdrawAddressRequest is the request type for the
/// Query/DelegatorWithdrawAddress RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorWithdrawAddressRequest {
    /// delegator_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
/// QueryDelegatorWithdrawAddressResponse is the response type for the
/// Query/DelegatorWithdrawAddress RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDelegatorWithdrawAddressResponse {
    /// withdraw_address defines the delegator address to query for.
    #[prost(string, tag = "1")]
    pub withdraw_address: ::prost::alloc::string::String,
}
/// QueryCommunityPoolRequest is the request type for the Query/CommunityPool RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCommunityPoolRequest {}
/// QueryCommunityPoolResponse is the response type for the Query/CommunityPool
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryCommunityPoolResponse {
    /// pool defines community pool's coins.
    #[prost(message, repeated, tag = "1")]
    pub pool: ::prost::alloc::vec::Vec<super::super::base::v1beta1::DecCoin>,
}
include!("cosmos.distribution.v1beta1.tonic.rs");
// @@protoc_insertion_point(module)
