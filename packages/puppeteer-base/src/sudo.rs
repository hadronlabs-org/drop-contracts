use std::{ops::Div, str::FromStr};

use crate::{
    msg::OpenAckVersion,
    state::{BaseConfig, PuppeteerBase, Transfer},
};
use cosmos_sdk_proto::cosmos::{
    bank::v1beta1::MsgSend,
    base::v1beta1::Coin as CosmosCoin,
    staking::v1beta1::{Delegation, Validator as CosmosValidator},
    tx::v1beta1::{TxBody, TxRaw},
};
use cosmwasm_std::{from_json, Addr, Binary, Decimal, DepsMut, Env, Response, StdError, Uint128};
use cw_storage_plus::Index;
use neutron_sdk::{
    bindings::{
        query::{NeutronQuery, QueryRegisteredQueryResponse},
        types::Height,
    },
    interchain_queries::{
        get_registered_query, query_kv_result,
        types::{KVReconstruct, QueryType},
        v045::{
            queries::query_unbonding_delegations,
            types::{Balances, Delegations, COSMOS_SDK_TRANSFER_MSG_URL},
        },
    },
    NeutronError, NeutronResult,
};
use prost::Message;
use serde::{de::DeserializeOwned, Serialize};

pub const DECIMAL_PLACES: u32 = 18;
const DECIMAL_FRACTIONAL: u128 = 10u128.pow(DECIMAL_PLACES);

impl<'a, T, U> PuppeteerBase<'a, T, U>
where
    T: BaseConfig + Serialize + DeserializeOwned + Clone,
    U: Serialize + DeserializeOwned + Clone,
{
    pub fn sudo_tx_query_result(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        query_id: u64,
        _height: Height,
        data: Binary,
    ) -> NeutronResult<Response> {
        let _config: T = self.config.load(deps.storage)?;
        let tx: TxRaw = TxRaw::decode(data.as_slice())?;
        let body: TxBody = TxBody::decode(tx.body_bytes.as_slice())?;
        let registered_query: QueryRegisteredQueryResponse =
            get_registered_query(deps.as_ref(), query_id)?;
        #[allow(clippy::single_match)]
        match registered_query.registered_query.query_type {
            QueryType::TX => {
                let ica = self.ica.get_address(deps.storage)?;
                let deposits = self.recipient_deposits_from_tx_body(body, &ica)?;
                if deposits.is_empty() {
                    return Err(NeutronError::Std(StdError::generic_err(
                        "failed to find a matching transaction message",
                    )));
                }
                let mut txs = self.recipient_transfers.load(deps.storage)?;
                txs.extend(deposits);
                self.recipient_transfers.save(deps.storage, &txs)?;
            }
            _ => {}
        }
        Ok(Response::new())
    }

    /// parses tx body and retrieves transactions to the given recipient.
    fn recipient_deposits_from_tx_body(
        &self,
        tx_body: TxBody,
        recipient: &str,
    ) -> NeutronResult<Vec<Transfer>> {
        let mut deposits: Vec<Transfer> = vec![];
        // for msg in tx_body.messages.iter().take(MAX_ALLOWED_MESSAGES) {
        for msg in tx_body.messages.iter() {
            #[allow(clippy::single_match)]
            match msg.type_url.as_str() {
                COSMOS_SDK_TRANSFER_MSG_URL => {
                    // Parse a Send message and check that it has the required recipient.
                    let transfer_msg: MsgSend = MsgSend::decode(msg.value.as_slice())?;
                    if transfer_msg.to_address == recipient {
                        for coin in transfer_msg.amount {
                            deposits.push(Transfer {
                                sender: transfer_msg.from_address.clone(),
                                amount: coin.amount.clone(),
                                denom: coin.denom,
                                recipient: recipient.to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(deposits)
    }

    pub fn sudo_kv_query_result(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        query_id: u64,
        storage: cw_storage_plus::Item<'a, (Delegations, Balances, u64)>,
    ) -> NeutronResult<Response> {
        let data: BalancesAndDelegations = query_kv_result(deps.as_ref(), query_id)?;
        let height = env.block.height;
        storage.save(deps.storage, &(data.delegations, data.balances, height))?;
        Ok(Response::default())
    }

    pub fn sudo_unbonding_delegations_kv_query_result(
        &self,
        deps: DepsMut<NeutronQuery>,
        env: Env,
        query_id: u64,
    ) -> NeutronResult<Response> {
        if let Some(mut item) = self
            .unbonding_delegations
            .idx
            .query_id
            .item(deps.storage, query_id)?
        {
            self.unbonding_delegations
                .idx
                .query_id
                .remove(deps.storage, &item.0, &item.1)?;

            item.1.unbonding_delegations =
                query_unbonding_delegations(deps.as_ref(), env.clone(), query_id)?
                    .unbonding_delegations
                    .unbonding_responses
                    .pop()
                    .unwrap()
                    .entries;
            item.1.last_updated_height = env.block.height;

            self.unbonding_delegations
                .save(deps.storage, &item.1.validator_address, &item.1)?;
        }

        Ok(Response::default())
    }

    pub fn sudo_open_ack(
        &self,
        deps: DepsMut<NeutronQuery>,
        _env: Env,
        _port_id: String,
        _channel_id: String,
        _counterparty_channel_id: String,
        counterparty_version: String,
    ) -> NeutronResult<Response> {
        let parsed_version: Result<OpenAckVersion, _> =
            serde_json_wasm::from_str(counterparty_version.as_str());
        if let Ok(parsed_version) = parsed_version {
            self.ica.set_address(deps.storage, parsed_version.address)?;
            Ok(Response::default())
        } else {
            Err(NeutronError::Std(StdError::generic_err(
                "can't parse version",
            )))
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct BalancesAndDelegations {
    pub balances: Balances,
    pub delegations: Delegations,
}

impl KVReconstruct for BalancesAndDelegations {
    fn reconstruct(
        storage_values: &[neutron_sdk::bindings::types::StorageValue],
    ) -> NeutronResult<Self> {
        let mut coins: Vec<cosmwasm_std::Coin> = Vec::with_capacity(storage_values.len());
        let kv = &storage_values[0];
        if kv.value.len() > 0 {
            let balance = CosmosCoin::decode(kv.value.as_slice())?;
            let amount = Uint128::from_str(balance.amount.as_str())?;
            coins.push(cosmwasm_std::Coin::new(amount.u128(), balance.denom));
        }
        let mut delegations: Vec<cosmwasm_std::Delegation> =
            Vec::with_capacity((storage_values.len() - 2) / 2);

        if storage_values.is_empty() {
            return Err(NeutronError::InvalidQueryResultFormat(
                "storage_values length is 0".into(),
            ));
        }
        // first StorageValue is denom
        if storage_values[1].value.is_empty() {
            // Incoming denom cannot be empty, it should always be configured on chain.
            // If we receive empty denom, that means incoming data structure is corrupted
            // and we cannot build `cosmwasm_std::Delegation`'s using this data.
            return Err(NeutronError::InvalidQueryResultFormat(
                "denom is empty".into(),
            ));
        }
        let denom: String = from_json(&storage_values[1].value)?;
        // the rest are delegations and validators alternately
        for chunk in storage_values[2..].chunks(2) {
            if chunk[0].value.is_empty() {
                // Incoming delegation can actually be empty, this just means that delegation
                // is not present on remote chain, which is to be expected. So, if it doesn't
                // exist, we can safely skip this and following chunk.
                continue;
            }
            let delegation_sdk: Delegation = Delegation::decode(chunk[0].value.as_slice())?;

            let mut delegation_std = cosmwasm_std::Delegation {
                delegator: Addr::unchecked(delegation_sdk.delegator_address.as_str()),
                validator: delegation_sdk.validator_address,
                amount: Default::default(),
            };

            if chunk[1].value.is_empty() {
                // At this point, incoming validator cannot be empty, that would be invalid,
                // because delegation is already defined, so, building `cosmwasm_std::Delegation`
                // from this data is impossible, incoming data is corrupted.post
                return Err(NeutronError::InvalidQueryResultFormat(
                    "validator is empty".into(),
                ));
            }
            let validator: CosmosValidator = CosmosValidator::decode(chunk[1].value.as_slice())?;

            let delegation_shares =
                Decimal::from_atomics(Uint128::from_str(&delegation_sdk.shares)?, DECIMAL_PLACES)?;

            let delegator_shares = Decimal::from_atomics(
                Uint128::from_str(&validator.delegator_shares)?,
                DECIMAL_PLACES,
            )?;

            let validator_tokens = Decimal::from_atomics(Uint128::from_str(&validator.tokens)?, 0)?;

            // https://github.com/cosmos/cosmos-sdk/blob/35ae2c4c72d4aeb33447d5a7af23ca47f786606e/x/staking/keeper/querier.go#L463
            // delegated_tokens = quotient(delegation.shares * validator.tokens / validator.total_shares);
            let delegated_tokens = delegation_shares
                .checked_mul(validator_tokens)?
                .div(delegator_shares)
                .atomics()
                .u128()
                .div(DECIMAL_FRACTIONAL);

            delegation_std.amount = cosmwasm_std::Coin::new(delegated_tokens, &denom);

            delegations.push(delegation_std);
        }
        Ok(BalancesAndDelegations {
            delegations: Delegations { delegations },
            balances: Balances { coins },
        })
    }
}
