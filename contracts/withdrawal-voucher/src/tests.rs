use cosmwasm_std::{
    attr, coin,
    testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
    to_json_binary, Addr, Binary, CosmosMsg, Event, QueryRequest, Reply, ReplyOn, SubMsgResult,
    Uint128,
};
