use cosmwasm_std::{attr, ensure_eq, to_json_binary, Attribute, Deps, Order};
use cosmwasm_std::{Binary, DepsMut, Env, MessageInfo, Response, StdResult};
use cw_ownable::{get_ownership, update_ownership};
use drop_helpers::answer::response;
use drop_staking_base::error::validatorset::{ContractError, ContractResult};
use drop_staking_base::msg::validatorset::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ValidatorData, ValidatorInfoUpdate,
    ValidatorResponse,
};
use drop_staking_base::state::provider_proposals::ProposalInfo;
use drop_staking_base::state::validatorset::{
    Config, ConfigOptional, ValidatorInfo, CONFIG, VALIDATORS_LIST, VALIDATORS_SET,
};
use neutron_sdk::bindings::msg::NeutronMsg;
use neutron_sdk::bindings::query::NeutronQuery;

const CONTRACT_NAME: &str = concat!("crates.io:drop-staking__", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(msg.owner.as_ref()))?;

    let stats_contract = deps.api.addr_validate(&msg.stats_contract)?;
    let config = &Config {
        stats_contract: stats_contract.clone(),
        provider_proposals_contract: None,
    };
    CONFIG.save(deps.storage, config)?;

    Ok(response(
        "instantiate",
        CONTRACT_NAME,
        [attr("stats_contract", stats_contract)],
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
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
    let validators = VALIDATORS_LIST.load(deps.storage)?;
    Ok(to_json_binary(&validators)?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
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

    if let Some(stats_contract) = new_config.stats_contract {
        state.stats_contract = deps.api.addr_validate(&stats_contract)?;
        attrs.push(attr("stats_contract", stats_contract))
    }

    if let Some(provider_proposals_contract) = new_config.provider_proposals_contract {
        state.provider_proposals_contract =
            Some(deps.api.addr_validate(&provider_proposals_contract)?);
        attrs.push(attr(
            "provider_proposals_contract",
            provider_proposals_contract,
        ));
    }

    CONFIG.save(deps.storage, &state)?;

    Ok(response("update_config", CONTRACT_NAME, Vec::<Attribute>::new()).add_attributes(attrs))
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

    update_validators_list(deps)?;

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

        if let Some(last_processed_local_height) = update.last_processed_local_height {
            validator.last_processed_local_height = Some(
                last_processed_local_height
                    .max(validator.last_processed_local_height.unwrap_or_default()),
            );
        }

        if let Some(last_processed_remote_height) = update.last_processed_remote_height {
            validator.last_processed_remote_height = Some(
                last_processed_remote_height
                    .max(validator.last_processed_remote_height.unwrap_or_default()),
            );
        }

        if let Some(last_validated_height) = update.last_validated_height {
            validator.last_validated_height = Some(
                last_validated_height.max(validator.last_validated_height.unwrap_or_default()),
            );
        }

        if let Some(jailed_number) = update.jailed_number {
            validator.jailed_number =
                Some(jailed_number.max(validator.jailed_number.unwrap_or_default()));
        }

        validator.uptime = update.uptime;
        validator.tombstone = update.tombstone;

        VALIDATORS_SET.save(deps.storage, validator.valoper_address.clone(), &validator)?;
    }

    update_validators_list(deps)?;

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

    update_validators_list(deps)?;

    Ok(response(
        "execute_update_validators_voting",
        CONTRACT_NAME,
        [attr(
            "proposal_id",
            proposal.proposal.proposal_id.to_string(),
        )],
    ))
}

fn update_validators_list(deps: DepsMut<NeutronQuery>) -> StdResult<()> {
    let validators: StdResult<Vec<_>> = VALIDATORS_SET
        .range_raw(deps.storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_key, value)| value))
        .collect();

    VALIDATORS_LIST.save(deps.storage, &validators.unwrap_or_default())?;

    Ok(())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let version: semver::Version = CONTRACT_VERSION.parse()?;
    let storage_version: semver::Version =
        cw2::get_contract_version(deps.storage)?.version.parse()?;

    if storage_version < version {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    }

    Ok(Response::new())
}
