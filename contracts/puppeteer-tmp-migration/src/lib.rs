use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use drop_puppeteer_base::{
    error::ContractResult,
    msg::QueryMsg,
    state::{PuppeteerBase, TxState, TxStateStatus},
};
use drop_staking_base::{
    msg::puppeteer::{ExecuteMsg, InstantiateMsg, MigrateMsg},
    state::puppeteer::{Config, KVQueryType},
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

pub type Puppeteer<'a> = PuppeteerBase<'a, Config, KVQueryType>;

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(_deps: Deps<NeutronQuery>, _env: Env, _msg: QueryMsg) -> ContractResult<Binary> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> ContractResult<Response<NeutronMsg>> {
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let puppeteer_base = Puppeteer::default();

    puppeteer_base.tx_state.save(
        deps.storage,
        &TxState {
            status: TxStateStatus::Idle,
            seq_id: None,
            transaction: None,
            reply_to: None,
        },
    )?;

    Ok(Response::new())
}
