use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use drop_staking_base::{
    error::core::ContractResult,
    msg::{
        core::{ExecuteMsg, InstantiateMsg, QueryMsg},
        withdrawal_voucher::ExecuteMsg as VoucherExecuteMsg,
    },
};
use neutron_sdk::bindings::{msg::NeutronMsg, query::NeutronQuery};

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

#[cw_serde]
pub struct TransferMsg {
    pub recipient: String,
    pub token_id: String,
}

#[cw_serde]
pub struct MigrateMsg {
    pub nfts_transfers: Vec<TransferMsg>,
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    _deps: DepsMut<NeutronQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> ContractResult<Response<NeutronMsg>> {
    let mut msgs = vec![];
    for transfer in msg.nfts_transfers.iter() {
        let transfer_msg = VoucherExecuteMsg::TransferNft {
            recipient: transfer.recipient.clone(),
            token_id: transfer.token_id.clone(),
        };
        let exec_msg = cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
            contract_addr: "neutron1atrxup8mj3dky7jcch3e3524t97hgdzfud9kc6zrkw3dwmgf69ws37ruc5"
                .to_string(),
            msg: cosmwasm_std::to_json_binary(&transfer_msg)?,
            funds: vec![],
        });
        msgs.push(exec_msg);
    }

    Ok(Response::new().add_messages(msgs))
}
