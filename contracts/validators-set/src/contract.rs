use cosmwasm_std::{attr, ensure_eq, entry_point, to_json_binary, Attribute, Deps, Order};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use cw_ownable::{get_ownership, update_ownership};
use lido_helpers::answer::response;
use lido_staking_base::error::validatorset::{ContractError, ContractResult};
use lido_staking_base::msg::validatorset::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ValidatorData, ValidatorInfoUpdate,
    ValidatorResponse,
};
use lido_staking_base::state::provider_proposals::ProposalInfo;
use lido_staking_base::state::validatorset::{
    Config, ConfigOptional, ValidatorInfo, CONFIG, VALIDATORS_SET,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

const CONTRACT_NAME: &str = concat!("crates.io:lido-staking__", env!("CARGO_PKG_NAME"));

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let stats_contract = deps.api.addr_validate(&msg.stats_contract)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let config = &Config {
        owner: owner.clone(),
        stats_contract: stats_contract.clone(),
        provider_proposals_contract: None,
    };

    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("owner", owner), attr("stats_contract", stats_contract)],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<NeutronQuery>, env: Env, msg: QueryMsg) -> ContractResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => Ok(to_json_binary(&get_ownership(deps.storage)?)?),
        QueryMsg::Config {} => query_config(deps, env),
        QueryMsg::Validator { valoper } => query_validator(deps, valoper),
        QueryMsg::Validators {} => query_validators(deps),
    }
}

fn query_config(deps: Deps<NeutronQuery>, _env: Env) -> ContractResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_json_binary(&config)?)
}

fn query_validator(deps: Deps<NeutronQuery>, valoper: String) -> ContractResult<Binary> {
    let validator = VALIDATORS_SET.may_load(deps.storage, valoper)?;

    Ok(to_json_binary(&ValidatorResponse { validator })?)
}

fn query_validators(deps: Deps<NeutronQuery>) -> ContractResult<Binary> {
    let validators: StdResult<Vec<_>> = VALIDATORS_SET
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    Ok(to_json_binary(&validators?)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps.into_empty(), &env.block, &info.sender, action)?;
            Ok(Response::new())
        }
        ExecuteMsg::UpdateConfig { new_config } => execute_update_config(deps, info, new_config),
        ExecuteMsg::UpdateValidators { validators } => {
            execute_update_validators(deps, info, validators)
        }
        ExecuteMsg::UpdateValidator { validator } => {
            execute_update_validator(deps, info, validator)
        }
        ExecuteMsg::UpdateValidatorsInfo { validators } => {
            execute_update_validators_info(deps, info, validators)
        }
        ExecuteMsg::UpdateValidatorsVoting { proposal } => {
            execute_update_validators_voting(deps, info, proposal)
        }
    }
}

fn execute_update_config(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    new_config: ConfigOptional,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut state = CONFIG.load(deps.storage)?;

    let mut attrs: Vec<Attribute> = Vec::new();

    if let Some(owner) = new_config.owner {
        if owner != state.owner {
            state.owner = owner.clone();
            cw_ownable::initialize_owner(deps.storage, deps.api, Some(state.owner.as_ref()))?;
        }
        attrs.push(attr("owner", owner.to_string()))
    }

    if let Some(stats_contract) = new_config.stats_contract {
        state.stats_contract = stats_contract.clone();
        attrs.push(attr("stats_contract", stats_contract))
    }

    if new_config.provider_proposals_contract.is_some() {
        state.provider_proposals_contract = new_config.provider_proposals_contract.clone();
        attrs.push(attr(
            "provider_proposals_contract",
            new_config.provider_proposals_contract.unwrap().to_string(),
        ))
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, Vec::<Attribute>::new()).add_attributes(attrs))
}

fn execute_update_validator(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validator: ValidatorData,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    // TODO: implement notification of the validator stats contract about new validator
    let valoper_address = validator.valoper_address.clone();

    VALIDATORS_SET.save(
        deps.storage,
        valoper_address.clone(),
        &ValidatorInfo {
            valoper_address: validator.valoper_address,
            weight: validator.weight,
            last_processed_remote_height: None,
            last_processed_local_height: None,
            last_validated_height: None,
            last_commission_in_range: None,
            uptime: Default::default(),
            tombstone: false,
            jailed_number: None,
            init_proposal: None,
            total_passed_proposals: 0,
            total_voted_proposals: 0,
        },
    )?;

    Ok(response(
        "update_validator",
        CONTRACT_NAME,
        [
            attr("address", valoper_address),
            attr("weight", validator.weight.to_string()),
        ],
    ))
}

fn execute_update_validators(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators: Vec<ValidatorData>,
) -> ContractResult<Response<NeutronMsg>> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let total_count = validators.len();

    // TODO: implement notification of the validator stats contract about new validators set
    VALIDATORS_SET.clear(deps.storage);

    for validator in validators {
        let valoper_address = validator.valoper_address.clone();

        VALIDATORS_SET.save(
            deps.storage,
            valoper_address,
            &ValidatorInfo {
                valoper_address: validator.valoper_address,
                weight: validator.weight,
                last_processed_remote_height: None,
                last_processed_local_height: None,
                last_validated_height: None,
                last_commission_in_range: None,
                uptime: Default::default(),
                tombstone: false,
                jailed_number: None,
                init_proposal: None,
                total_passed_proposals: 0,
                total_voted_proposals: 0,
            },
        )?;
    }

    Ok(response(
        "update_validators",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

fn execute_update_validators_info(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    validators_update: Vec<ValidatorInfoUpdate>,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        config.stats_contract,
        info.sender,
        ContractError::Unauthorized {}
    );

    let total_count = validators_update.len();

    for update in validators_update {
        // TODO: Implement logic to modify validator set based in incoming validator info
        let validator =
            VALIDATORS_SET.may_load(deps.storage, update.valoper_address.to_string())?;
        if validator.is_none() {
            continue;
        }
        let mut validator = validator.unwrap();

        if update.last_commission_in_range.is_some() {
            validator.last_commission_in_range = update.last_commission_in_range;
        }
        if update.last_processed_local_height.is_some() {
            validator.last_processed_local_height = update.last_processed_local_height;
        }
        if update.last_processed_remote_height.is_some() {
            validator.last_processed_remote_height = update.last_processed_remote_height;
        }
        if update.last_validated_height.is_some() {
            validator.last_validated_height = update.last_validated_height;
        }
        if update.jailed_number.is_some() {
            validator.jailed_number = update.jailed_number;
        }

        validator.uptime = update.uptime;
        validator.tombstone = update.tombstone;

        VALIDATORS_SET.save(deps.storage, validator.valoper_address.clone(), &validator)?;
    }

    Ok(response(
        "update_validators_info",
        CONTRACT_NAME,
        [attr("total_count", total_count.to_string())],
    ))
}

fn execute_update_validators_voting(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    proposal: ProposalInfo,
) -> ContractResult<Response<NeutronMsg>> {
    let config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        config.provider_proposals_contract,
        Some(info.sender),
        ContractError::Unauthorized {}
    );

    if proposal.is_spam {
        return Ok(response(
            "update_validators_info",
            CONTRACT_NAME,
            [attr(
                "spam_proposal",
                proposal.proposal.proposal_id.to_string(),
            )],
        ));
    }

    if let Some(votes) = proposal.votes {
        for vote in votes {
            let validator = VALIDATORS_SET.may_load(deps.storage, vote.voter.to_string())?;

            if let Some(validator) = validator {
                let mut validator = validator;

                if validator.init_proposal.is_none() {
                    validator.init_proposal = Some(proposal.proposal.proposal_id);
                }

                if !vote.options.is_empty() {
                    validator.total_voted_proposals += 1;
                }

                validator.total_passed_proposals += 1;

                VALIDATORS_SET.save(deps.storage, validator.valoper_address.clone(), &validator)?;
            }
        }
    }

    Ok(response(
        "execute_update_validators_voting",
        CONTRACT_NAME,
        [attr(
            "proposal_id",
            proposal.proposal.proposal_id.to_string(),
        )],
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    deps.api.debug("WASMDEBUG: migrate");
    Ok(Response::default())
}
