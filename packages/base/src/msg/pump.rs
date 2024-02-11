use crate::state::pump::{IBCFees, PumpTimeout};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Coin;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(crate::state::pump::Config)]
    Config {},
    #[returns(lido_helpers::ica::IcaState)]
    Ica {},
}

#[cw_serde]
pub struct OpenAckVersion {
    pub version: String,
    pub controller_connection_id: String,
    pub host_connection_id: String,
    pub address: String,
    pub encoding: String,
    pub tx_type: String,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub dest_address: Option<String>,
    pub dest_channel: Option<String>,
    pub dest_port: Option<String>,
    pub connection_id: Option<String>,
    pub refundee: Option<String>,
    pub admin: Option<String>,
    pub ibc_fees: Option<IBCFees>,
    pub timeout: Option<PumpTimeout>,
    pub local_denom: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterICA {},
    Push { coins: Vec<Coin> },
    Refund {},
    UpdateConfig { new_config: Box<UpdateConfigMsg> },
}

#[cw_serde]
pub struct InstantiateMsg {
    pub dest_address: Option<String>,
    pub dest_channel: Option<String>,
    pub dest_port: Option<String>,
    pub connection_id: String,
    pub ibc_fees: IBCFees,
    pub refundee: Option<String>,
    pub timeout: PumpTimeout,
    pub local_denom: String,
    pub owner: Option<String>,
}

#[cw_serde]
pub enum MigrateMsg {}
