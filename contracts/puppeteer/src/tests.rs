use crate::contract::{query, Puppeteer};
use cosmwasm_schema::schemars;
use cosmwasm_std::{
    coin, coins, from_json,
    testing::{mock_env, mock_info},
    to_json_binary, Addr, Binary, CosmosMsg, Delegation, DepsMut, Event, Response, StdError,
    SubMsg, Timestamp, Uint128, Uint64,
};
use drop_helpers::{
    ibc_client_state::{
        ChannelClientStateResponse, ClientState, Fraction, Height, IdentifiedClientState,
    },
    testing::mock_dependencies,
};
use drop_puppeteer_base::{
    msg::IBCTransferReason,
    state::{BalancesAndDelegations, BalancesAndDelegationsState, PuppeteerBase, ReplyMsg},
};
use drop_staking_base::{
    msg::puppeteer::InstantiateMsg,
    state::puppeteer::{Config, ConfigOptional, KVQueryType, NON_NATIVE_REWARD_BALANCES},
};
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::{NeutronQuery, QueryRegisteredQueryResultResponse},
        types::{InterchainQueryResult, StorageValue},
    },
    interchain_queries::v045::types::{Balances, Delegations},
    query::min_ibc_fee::MinIbcFeeResponse,
    sudo::msg::{RequestPacketTimeoutHeight, SudoMsg},
    NeutronError,
};
use prost::Message;
use schemars::_serde_json::to_string;

use std::vec;

fn build_interchain_query_response() -> Binary {
    let res: Vec<StorageValue> = from_json(
        r#"[
        {
          "storage_prefix": "bank",
          "key": "AiCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZnN0YWtl",
          "value": "Mjk1NTg3Nzg=",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "AiCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZnN0YWtl",
                "data": "CsMECicCIJ8kSLPxG50cEOtJwtHykgxSUId2610KUji0QDIhS5dmc3Rha2USCDI5NTU4Nzc4Gg0IARgBIAEqBQACyvt2Ii0IARIGBAbK+3YgGiEgYp8lWwF/sxZ2CRevS1dI7cvW5N6Ohw5BUeYXWhorg44iLQgBEgYGDIrOeiAaISAs8eFQgsZKmbrRyaxeRR5NsdnroM0L6n+KouSnXOpXXiIrCAESJwgSis56IBqCWnaLDTRyL4nYWOeHejnQG7Qh9uShdlUBsBee44NwICIrCAESJwoois56IOuMhiK9zQlPcG2kLJvf1tZ0cNuAE9kvRo0JtX1ZbVEJICIrCAESJwxWis56IIx9VuX+2+FGZjakYFMhqt+jIhoOv33SYk8v1FH6jjl2ICIsCAESKBDMAYrOeiDtvkHasikYrbnv7pE8TyzQekgOWVD1ItBpfxlgwb3o6yAiLggBEgcSpAOKznogGiEg0Yv3s5T1+PuLFxjQgjvi8dfT7/e8Rnb9/h3e96suvPAiLggBEgcW0AeKznogGiEgJPiLrkZ+d4Lf4RrvmpMf+PPy8s8jKAEjVLj1o6Uh6xMiLggBEgcYrA6KznogGiEg5H5D/hVYxx8AxmChjNCm81ca0fWNx17LSzs1xax/Py4iLggBEgcawhqKznogGiEg+CtNadV1e34QFqMdAvFcQXRNkyII5AhvDnPIFy0mpkoiLAgBEigeskqi1Xog01iuI3+E8qcOKmBfQ3+8nWlrYPD86SDu9/z+Qa0JgKwg"
              },
              {
                "type": "ics23:simple",
                "key": "YmFuaw==",
                "data": "Cv4BCgRiYW5rEiDl8U7j6ksUgCGCN8VRwi0PGjuxwFipNF1+sR1D+hUI0xoJCAEYASABKgEAIicIARIBARog/UO8D5BFC5Vn//1k5W4cudYTGMXuILD4W3fD2TwS7MUiJQgBEiEByGE88YTy7IzlOl604TkdhnrvQJ9/KhwVmar9bl68uHMiJwgBEgEBGiAco1IMsTvufglAIzy63zwXrTuaaaOY+rWklwvpRdQ5jCInCAESAQEaII9mfCYnRdxjYUSCf2BcvEVhKalc0pCqEVU9AoSlqPg9IicIARIBARogajRhqPhahj7EWQGqWSi8i4TvfkeOHVw57LSLUyJs6Lg="
              }
            ]
          }
        },
        {
          "storage_prefix": "staking",
          "key": "UQ==",
          "value": "CgMImCoQZBgHIJBOKgVzdGFrZTIBMDoVNTAwMDAwMDAwMDAwMDAwMDAwMDAwQhMxMDAwMDAwMDAwMDAwMDAwMDAwShMxMDAwMDAwMDAwMDAwMDAwMDAw",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "UQ==",
                "data": "CoQGCgFRElcKAwiYKhBkGAcgkE4qBXN0YWtlMgEwOhU1MDAwMDAwMDAwMDAwMDAwMDAwMDBCEzEwMDAwMDAwMDAwMDAwMDAwMDBKEzEwMDAwMDAwMDAwMDAwMDAwMDAaDQgBGAEgASoFAAK8pTgiLQgBEgYCBKLVeiAaISD0z8SFslDr14MvQR9N/dlCRe7AgoJ0BWsPTMbV9Lvx+CItCAESBgQIotV6IBohILMcqKBejLXu/loiRMNNxxvtyqPN75jVjoHiFEggIRm7Ii0IARIGBhCi1XogGiEgPt6P0sJf8OXxokuhpRYOr2TevHiR1uhSGZQG1F60PtIiLQgBEgYIGKLVeiAaISCNq3GeR3dxioeBlpnR08daimACRcAMs0CJSdk74NWWiSItCAESBgowotV6IBohIEhMhnxBc+YZZaY6o9/KXi0WuFtnBjO3FS7/1DyS9HKvIi0IARIGDGCi1XogGiEgS+nyI4XiaRt6WMGVaoyq5MyOncZ0XxrytN+9YCyzLskiLggBEgcOwAGi1XogGiEgSkqNP/UlVB1cvU1WvBWEjEexyGXy/v1T5ztgPzquobAiLggBEgcQ/AKi1XogGiEgpTUeKymdUvyl5RiNqZ31Mno8jOYe+QgFNJzYW9PTYBkiLggBEgcS/AWi1XogGiEgtSKXXf8W50F9eO0ddVc/e6g2GNDPJngR/YGak6Td1dsiLggBEgcU/Aui1XogGiEgURAWC6bloLG6lz3GkMDBh6OeWSVkBHdRj/UVaQYls5oiLggBEgcY2Bii1XogGiEgHhO56NJnZtA200HA7W4bhMvUxP9rLqgzcWLeoqT4UToiLggBEgcakDGi1XogGiEgfpW7L9d8GXmMYA5qfwKz0b60yHPcyx4eTnqO7y7igxYiLggBEgcc2kmi1XogGiEgw/Ptq1X8Sq985PtHvUq2CBWaLoyxu45omenrJa8MANwiLQgBEikg4pkCotV6IBRQy9CPFKCd5LWg4ZAS2LVDxRUeF83hwhLYbiOiD5YeIA=="
              },
              {
                "type": "ics23:simple",
                "key": "c3Rha2luZw==",
                "data": "Cq0BCgdzdGFraW5nEiCkO7A33k4ofrlVxpXYp2znmjCjsspPzy3hhMXhD3i8JxoJCAEYASABKgEAIiUIARIhAUItnB3KEB621b5Jm8wmw2rWrU3oNVJyQl/ENDf3CI2vIicIARIBARog2qV2hEwd21IQKQIX5cmlxr7g0DOIzeBOD6Aqqjn9MSAiJQgBEiEBN3Y/z8W03TzEbDpWXZYQYM8LpitgpK8AzHhyLPy7g5I="
              }
            ]
          }
        },
        {
          "storage_prefix": "staking",
          "key": "MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhQc2kl1CUPnDLfq4SkyqZCEz9lSeg==",
          "value": "CkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFybmR5amFnZmcwbnNlZGwydXk1bjkydnNzbjhhajVuNjd0MG5meBodMTM1ODI0NjUxNTIwMDAwMDAwMDAwMDAwMDAwMDA=",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhQc2kl1CUPnDLfq4SkyqZCEz9lSeg==",
                "data": "CqsHCjcxIJ8kSLPxG50cEOtJwtHykgxSUId2610KUji0QDIhS5dmFBzaSXUJQ+cMt+rhKTKpkITP2VJ6EpgBCkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFybmR5amFnZmcwbnNlZGwydXk1bjkydnNzbjhhajVuNjd0MG5meBodMTM1ODI0NjUxNTIwMDAwMDAwMDAwMDAwMDAwMDAaDQgBGAEgASoFAAKe73YiLQgBEgYCBJ7vdiAaISCHatBKmLP/CigOzdIPv9b/Onku02N/X7xPsvUNG9PfGyItCAESBgQGnu92IBohIH/P4HCZJL8E7savbyjCiBxflr5nECuiYiS81U+0G48aIisIARInBg6e73Ygf5sitASH+atnjvKMiV7gRMfuFrKS/v0igF7nqNEg3EEgIi0IARIGCBae73YgGiEgCeNmBtcLKuqi/FiEvT0yfP8hY+B2Sra+mMgmYRpRL9ciLQgBEgYKLJ7vdiAaISCW1/SR4O9f464ujquaXsfeEo1Zz8IbRegPW22YYEhLIiItCAESBgxGnu92IBohIBKm9hZUwDZos317NJlx/KLAcUyiUF+jAsWwqNLq7nwBIi0IARIGDnye73YgGiEgFA6e95aPc6FfVTl5gE+ZrVk6X7JPHaEDGM/ma7CCMFciLggBEgcQiALkmXcgGiEgXCTLKkeOf7RHN5KYRSE64XSQOXcmsuC/z3EMBD718VoiLggBEgcUwgSa/HcgGiEg7ckQnyToe015kgq4jVcv0kroiUCa07j6treUewJoSuMiLggBEgcW3gma/HcgGiEgVnfbPoyVHoH6maUTLftf4GXlbLV3R0IxbCa5s0o/oUQiLggBEgcY3BngmnogGiEgstZsdo2RKzMj8E1EjI8DKW4m50gRGx4xvYJ2tiq9/dciLAgBEiga/ijgunog4xrRXCgmXUXLqpaOrV7C3kjMXHyl2pKmRvCOnTDIZS4gIi8IARIIHIyAAaLVeiAaISCs/utEZeR2svvvndJWzJBFmqWMpWBtr0pgCwUWgwkhAyIvCAESCB6I0AGi1XogGiEg+bwZWravF0K3pdjJAKUREHzIJstwCFUiu3VLt8RFsz8iLwgBEggg4pkCotV6IBohIHjRrBtVdOyYGO8ru1T1BukaP6BLJ8H17c2JI4LqSAub"
              },
              {
                "type": "ics23:simple",
                "key": "c3Rha2luZw==",
                "data": "Cq0BCgdzdGFraW5nEiCkO7A33k4ofrlVxpXYp2znmjCjsspPzy3hhMXhD3i8JxoJCAEYASABKgEAIiUIARIhAUItnB3KEB621b5Jm8wmw2rWrU3oNVJyQl/ENDf3CI2vIicIARIBARog2qV2hEwd21IQKQIX5cmlxr7g0DOIzeBOD6Aqqjn9MSAiJQgBEiEBN3Y/z8W03TzEbDpWXZYQYM8LpitgpK8AzHhyLPy7g5I="
              }
            ]
          }
        },
        {
          "storage_prefix": "staking",
          "key": "IRQc2kl1CUPnDLfq4SkyqZCEz9lSeg==",
          "value": "CjRjb3Ntb3N2YWxvcGVyMXJuZHlqYWdmZzBuc2VkbDJ1eTVuOTJ2c3NuOGFqNW42N3QwbmZ4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIB4wQBwhrBNl3pDg7PIpHeuQVlEhXOPPKLVWN7JJSp+MIAMqCzI1MjE0MTg4Nzk2Mh0yNTIxNDE4ODc5NjAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMUoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQyMTQxODg3MjEwMDAwMDAwMDAwMDAwMDAwMDA=",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "IRQc2kl1CUPnDLfq4SkyqZCEz9lSeg==",
                "data": "CrgIChYhFBzaSXUJQ+cMt+rhKTKpkITP2VJ6EsMCCjRjb3Ntb3N2YWxvcGVyMXJuZHlqYWdmZzBuc2VkbDJ1eTVuOTJ2c3NuOGFqNW42N3QwbmZ4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIB4wQBwhrBNl3pDg7PIpHeuQVlEhXOPPKLVWN7JJSp+MIAMqCzI1MjE0MTg4Nzk2Mh0yNTIxNDE4ODc5NjAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMUoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQyMTQxODg3MjEwMDAwMDAwMDAwMDAwMDAwMDAaDQgBGAEgASoFAALOiHoiLQgBEgYCBM6IeiAaISC6p3vsMZLZxoRC3nLlfzNgdYQ+zxxCkETl08WchFK12iItCAESBgQIzoh6IBohIJAvWYHfUlvdU6yaQEmH2GChFu5i3z1tTUNZjmrHtk4ZIisIARInBhDOiHogupclTB+Gw+Rgyt51FTcCBgW/BIWsRIJAP/G7VjY3losgIi0IARIGCBrOiHogGiEg5BXK3xbTQR3EpVQHaBzTciynO0c1IL08f74I6wSXgzwiLQgBEgYKLM6IeiAaISD2ir0zX7WVVhjvS7N0BL1tH0aFJTMJ4l06Ac59EoW1XiItCAESBgxOzoh6IBohIIRj7BzPNzHK/liCNNedZVAPog6Xobs9TAzlZm+z0B/bIi4IARIHELoBzoh6IBohIJ3gK86AIMjG0w7e3z5bRtbOqPPCrxcgkSrhsHajUxXKIi4IARIHEtQCzoh6IBohINFu/Os3zgXVS8r3JIoqRUYSglCrxLktoKlLzMrc/7XZIi4IARIHFM4Fzoh6IBohIKhMdlh+gV8ntNR+cNicQ7xC6qDk9neaw0e+X3gGW8ikIi4IARIHFoYKzoh6IBohIJvZ53IQxQdZAY71JdjJ2rGcLqUvpSDcsr4pgQ3oS5nOIi4IARIHGKIPzoh6IBohIEVvUhj6LZWVcImSQEdMl05RzkUIn/zDFr+ojO9FXFENIi4IARIHGv4o4Lp6IBohIIq7LCUraTygCVvA1sqM513yKIxkteKY3y/RzCJp0B2PIi8IARIIHIyAAaLVeiAaISCs/utEZeR2svvvndJWzJBFmqWMpWBtr0pgCwUWgwkhAyIvCAESCB6I0AGi1XogGiEg+bwZWravF0K3pdjJAKUREHzIJstwCFUiu3VLt8RFsz8iLwgBEggg4pkCotV6IBohIHjRrBtVdOyYGO8ru1T1BukaP6BLJ8H17c2JI4LqSAub"
              },
              {
                "type": "ics23:simple",
                "key": "c3Rha2luZw==",
                "data": "Cq0BCgdzdGFraW5nEiCkO7A33k4ofrlVxpXYp2znmjCjsspPzy3hhMXhD3i8JxoJCAEYASABKgEAIiUIARIhAUItnB3KEB621b5Jm8wmw2rWrU3oNVJyQl/ENDf3CI2vIicIARIBARog2qV2hEwd21IQKQIX5cmlxr7g0DOIzeBOD6Aqqjn9MSAiJQgBEiEBN3Y/z8W03TzEbDpWXZYQYM8LpitgpK8AzHhyLPy7g5I="
              }
            ]
          }
        },
        {
          "storage_prefix": "staking",
          "key": "MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhRF6sE4roJR9V4+ACeV5W/dXDFvzA==",
          "value": "CkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFnaDR2enc5d3NmZ2wyaDM3cXFuZXRldDBtNHdyem03djd4M2o5eBodMTM1ODI0NjUxNTIwMDAwMDAwMDAwMDAwMDAwMDA=",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "MSCfJEiz8RudHBDrScLR8pIMUlCHdutdClI4tEAyIUuXZhRF6sE4roJR9V4+ACeV5W/dXDFvzA==",
                "data": "CqkHCjcxIJ8kSLPxG50cEOtJwtHykgxSUId2610KUji0QDIhS5dmFEXqwTiuglH1Xj4AJ5Xlb91cMW/MEpgBCkFjb3Ntb3MxbnVqeTN2bDNyd3czY3k4dGY4cGRydTVqcDNmOXBwbWthZHdzNTUzY2szcXJ5ZzJ0amFucXQzOXhudhI0Y29zbW9zdmFsb3BlcjFnaDR2enc5d3NmZ2wyaDM3cXFuZXRldDBtNHdyem03djd4M2o5eBodMTM1ODI0NjUxNTIwMDAwMDAwMDAwMDAwMDAwMDAaDQgBGAEgASoFAAKe73YiKwgBEicCBJ7vdiALKzEtlNXm+FSXFxS2dvQc96bQGrOYbfKVsbSjCPGnkSAiLQgBEgYEBp7vdiAaISB/z+BwmSS/BO7Gr28owogcX5a+ZxAromIkvNVPtBuPGiIrCAESJwYOnu92IH+bIrQEh/mrZ47yjIle4ETH7haykv79IoBe56jRINxBICItCAESBggWnu92IBohIAnjZgbXCyrqovxYhL09Mnz/IWPgdkq2vpjIJmEaUS/XIi0IARIGCiye73YgGiEgltf0keDvX+OuLo6rml7H3hKNWc/CG0XoD1ttmGBISyIiLQgBEgYMRp7vdiAaISASpvYWVMA2aLN9ezSZcfyiwHFMolBfowLFsKjS6u58ASItCAESBg58nu92IBohIBQOnveWj3OhX1U5eYBPma1ZOl+yTx2hAxjP5muwgjBXIi4IARIHEIgC5Jl3IBohIFwkyypHjn+0RzeSmEUhOuF0kDl3JrLgv89xDAQ+9fFaIi4IARIHFMIEmvx3IBohIO3JEJ8k6HtNeZIKuI1XL9JK6IlAmtO4+ra3lHsCaErjIi4IARIHFt4Jmvx3IBohIFZ32z6MlR6B+pmlEy37X+Bl5Wy1d0dCMWwmubNKP6FEIi4IARIHGNwZ4Jp6IBohILLWbHaNkSszI/BNRIyPAyluJudIERseMb2CdrYqvf3XIiwIARIoGv4o4Lp6IOMa0VwoJl1Fy6qWjq1ewt5IzFx8pdqSpkbwjp0wyGUuICIvCAESCByMgAGi1XogGiEgrP7rRGXkdrL7753SVsyQRZqljKVgba9KYAsFFoMJIQMiLwgBEggeiNABotV6IBohIPm8GVq2rxdCt6XYyQClERB8yCbLcAhVIrt1S7fERbM/Ii8IARIIIOKZAqLVeiAaISB40awbVXTsmBjvK7tU9QbpGj+gSyfB9e3NiSOC6kgLmw=="
              },
              {
                "type": "ics23:simple",
                "key": "c3Rha2luZw==",
                "data": "Cq0BCgdzdGFraW5nEiCkO7A33k4ofrlVxpXYp2znmjCjsspPzy3hhMXhD3i8JxoJCAEYASABKgEAIiUIARIhAUItnB3KEB621b5Jm8wmw2rWrU3oNVJyQl/ENDf3CI2vIicIARIBARog2qV2hEwd21IQKQIX5cmlxr7g0DOIzeBOD6Aqqjn9MSAiJQgBEiEBN3Y/z8W03TzEbDpWXZYQYM8LpitgpK8AzHhyLPy7g5I="
              }
            ]
          }
        },
        {
          "storage_prefix": "staking",
          "key": "IRRF6sE4roJR9V4+ACeV5W/dXDFvzA==",
          "value": "CjRjb3Ntb3N2YWxvcGVyMWdoNHZ6dzl3c2ZnbDJoMzdxcW5ldGV0MG00d3J6bTd2N3gzajl4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIBBp8oY6i7JDYLibq1ifiAsifWSXqMG943z+kPiorWjhIAMqCzI1MjE0MTc4MDUwMh0yNTIxNDE3ODA1MDAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMEoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQyMTQxNzgwNDkwMDAwMDAwMDAwMDAwMDAwMDA=",
          "Proof": {
            "ops": [
              {
                "type": "ics23:iavl",
                "key": "IRRF6sE4roJR9V4+ACeV5W/dXDFvzA==",
                "data": "CrYIChYhFEXqwTiuglH1Xj4AJ5Xlb91cMW/MEsMCCjRjb3Ntb3N2YWxvcGVyMWdoNHZ6dzl3c2ZnbDJoMzdxcW5ldGV0MG00d3J6bTd2N3gzajl4EkMKHS9jb3Ntb3MuY3J5cHRvLmVkMjU1MTkuUHViS2V5EiIKIBBp8oY6i7JDYLibq1ifiAsifWSXqMG943z+kPiorWjhIAMqCzI1MjE0MTc4MDUwMh0yNTIxNDE3ODA1MDAwMDAwMDAwMDAwMDAwMDAwMDoKCgh2YWxnYWlhMEoAUkoKOwoSMTAwMDAwMDAwMDAwMDAwMDAwEhIyMDAwMDAwMDAwMDAwMDAwMDAaETEwMDAwMDAwMDAwMDAwMDAwEgsIn/DYsgYQtvORQFoBMHIcMTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMHodMjQyMTQxNzgwNDkwMDAwMDAwMDAwMDAwMDAwMDAaDQgBGAEgASoFAAKa/HciKwgBEicCBM6IeiASopL/GJR0NWTylqmyGZhgruFGx20Il2frOHDEEvtLiCAiLQgBEgYECM6IeiAaISCQL1mB31Jb3VOsmkBJh9hgoRbuYt89bU1DWY5qx7ZOGSIrCAESJwYQzoh6ILqXJUwfhsPkYMredRU3AgYFvwSFrESCQD/xu1Y2N5aLICItCAESBggazoh6IBohIOQVyt8W00EdxKVUB2gc03IspztHNSC9PH++COsEl4M8Ii0IARIGCizOiHogGiEg9oq9M1+1lVYY70uzdAS9bR9GhSUzCeJdOgHOfRKFtV4iLQgBEgYMTs6IeiAaISCEY+wczzcxyv5YgjTXnWVQD6IOl6G7PUwM5WZvs9Af2yIuCAESBxC6Ac6IeiAaISCd4CvOgCDIxtMO3t8+W0bWzqjzwq8XIJEq4bB2o1MVyiIuCAESBxLUAs6IeiAaISDRbvzrN84F1UvK9ySKKkVGEoJQq8S5LaCpS8zK3P+12SIuCAESBxTOBc6IeiAaISCoTHZYfoFfJ7TUfnDYnEO8Quqg5PZ3msNHvl94BlvIpCIuCAESBxaGCs6IeiAaISCb2edyEMUHWQGO9SXYydqxnC6lL6Ug3LK+KYEN6EuZziIuCAESBxiiD86IeiAaISBFb1IY+i2VlXCJkkBHTJdOUc5FCJ/8wxa/qIzvRVxRDSIuCAESBxr+KOC6eiAaISCKuywlK2k8oAlbwNbKjOdd8iiMZLXimN8v0cwiadAdjyIvCAESCByMgAGi1XogGiEgrP7rRGXkdrL7753SVsyQRZqljKVgba9KYAsFFoMJIQMiLwgBEggeiNABotV6IBohIPm8GVq2rxdCt6XYyQClERB8yCbLcAhVIrt1S7fERbM/Ii8IARIIIOKZAqLVeiAaISB40awbVXTsmBjvK7tU9QbpGj+gSyfB9e3NiSOC6kgLmw=="
              },
              {
                "type": "ics23:simple",
                "key": "c3Rha2luZw==",
                "data": "Cq0BCgdzdGFraW5nEiCkO7A33k4ofrlVxpXYp2znmjCjsspPzy3hhMXhD3i8JxoJCAEYASABKgEAIiUIARIhAUItnB3KEB621b5Jm8wmw2rWrU3oNVJyQl/ENDf3CI2vIicIARIBARog2qV2hEwd21IQKQIX5cmlxr7g0DOIzeBOD6Aqqjn9MSAiJQgBEiEBN3Y/z8W03TzEbDpWXZYQYM8LpitgpK8AzHhyLPy7g5I="
              }
            ]
          }
        }
      ]"#,
    ).unwrap();

    Binary::from(
        to_string(&QueryRegisteredQueryResultResponse {
            result: InterchainQueryResult {
                kv_results: res,
                height: 123456,
                revision: 2,
            },
        })
        .unwrap()
        .as_bytes(),
    )
}

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies(&[]);
    let msg = InstantiateMsg {
        delegations_queries_chunk_size: Some(2u32),
        owner: Some("owner".to_string()),
        connection_id: "connection_id".to_string(),
        port_id: "port_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec!["allowed_sender".to_string()],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.47.10".to_string(),
        timeout: 100u64,
    };
    let env = mock_env();
    let res =
        crate::contract::instantiate(deps.as_mut(), env, mock_info("sender", &[]), msg).unwrap();
    assert_eq!(res, Response::new());
    let puppeteer_base = Puppeteer::default();
    let config = puppeteer_base.config.load(deps.as_ref().storage).unwrap();
    assert_eq!(config, get_base_config());
    assert_eq!(
        cosmwasm_std::Addr::unchecked("owner"),
        cw_ownable::get_ownership(deps.as_mut().storage)
            .unwrap()
            .owner
            .unwrap()
    );
}

#[test]
fn test_execute_update_config_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = Puppeteer::default();
    puppeteer_base
        .config
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::UpdateConfig {
        new_config: ConfigOptional {
            update_period: Some(121u64),
            remote_denom: Some("new_remote_denom".to_string()),
            allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
            transfer_channel_id: Some("new_transfer_channel_id".to_string()),
            connection_id: Some("new_connection_id".to_string()),
            port_id: Some("new_port_id".to_string()),
            sdk_version: Some("0.47.0".to_string()),
            timeout: Some(101u64),
        },
    };
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        msg.clone(),
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    )
}

#[test]
fn test_execute_update_config() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = Puppeteer::default();
    puppeteer_base
        .config
        .save(deps.as_mut().storage, &get_base_config())
        .unwrap();
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::UpdateConfig {
            new_config: ConfigOptional {
                update_period: Some(121u64),
                remote_denom: Some("new_remote_denom".to_string()),
                allowed_senders: Some(vec!["new_allowed_sender".to_string()]),
                transfer_channel_id: Some("new_transfer_channel_id".to_string()),
                connection_id: Some("new_connection_id".to_string()),
                port_id: Some("new_port_id".to_string()),
                sdk_version: Some("0.47.0".to_string()),
                timeout: Some(101u64),
            },
        },
    )
    .unwrap();
    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("crates.io:drop-neutron-contracts__drop-puppeteer-config_update")
                .add_attributes(vec![
                    ("remote_denom", "new_remote_denom"),
                    ("connection_id", "new_connection_id"),
                    ("port_id", "new_port_id"),
                    ("update_period", "121"),
                    ("allowed_senders", "1"),
                    ("transfer_channel_id", "new_transfer_channel_id"),
                    ("sdk_version", "0.47.0"),
                    ("timeout", "101"),
                ])
        )
    );

    let config = puppeteer_base.config.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        config,
        Config {
            delegations_queries_chunk_size: 2u32,
            port_id: "new_port_id".to_string(),
            connection_id: "new_connection_id".to_string(),
            update_period: 121u64,
            remote_denom: "new_remote_denom".to_string(),
            allowed_senders: vec![Addr::unchecked("new_allowed_sender")],
            transfer_channel_id: "new_transfer_channel_id".to_string(),
            sdk_version: "0.47.0".to_string(),
            timeout: 101u64,
        }
    );
}

#[test]
fn test_execute_setup_protocol_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::SetupProtocol {
            delegate_grantee: "delegate_grantee".to_string(),
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    );
    assert_eq!(
        res.unwrap_err(),
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_setup_protocol() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::SetupProtocol {
            delegate_grantee: "delegate_grantee".to_string(),
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    )
    .unwrap();
    let authz_msg = {
        neutron_sdk::bindings::types::ProtobufAny {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary::from(
                cosmos_sdk_proto::cosmos::authz::v1beta1::MsgGrant {
                    granter: "ica_address".to_string(),
                    grantee: "delegate_grantee".to_string(),
                    grant: Some(cosmos_sdk_proto::cosmos::authz::v1beta1::Grant {
                        expiration: Some(prost_types::Timestamp {
                            seconds: mock_env()
                                .block
                                .time
                                .plus_days(365 * 120 + 30)
                                .seconds()
                                .try_into()
                                .unwrap(),
                            nanos: 0,
                        }),
                        authorization: Some(cosmos_sdk_proto::Any {
                            type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                            value: cosmos_sdk_proto::cosmos::authz::v1beta1::GenericAuthorization {
                                msg: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                            }
                            .encode_to_vec(),
                        }),
                    }),
                }
                .encode_to_vec(),
            ),
        }
    };
    let distribution_msg = {
        neutron_sdk::bindings::types::ProtobufAny {
            type_url: "/cosmos.distribution.v1beta1.MsgSetWithdrawAddress".to_string(),
            value: Binary::from(
                cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgSetWithdrawAddress {
                    delegator_address: "ica_address".to_string(),
                    withdraw_address: "rewards_withdraw_address".to_string(),
                }
                .encode_to_vec(),
            ),
        }
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![authz_msg, distribution_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
    let tx_state = pupeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                interchain_account_id: "ica_address".to_string(),
                delegate_grantee: "delegate_grantee".to_string(),
                rewards_withdraw_address: "rewards_withdraw_address".to_string(),
            })
        }
    );
}

#[test]
fn test_execute_setup_protocol_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::SetupProtocol {
            delegate_grantee: "delegate_grantee".to_string(),
            rewards_withdraw_address: "rewards_withdraw_address".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_undelegate_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_undelegate_sender_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
            batch_id: 0u128,
            items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_undelegate() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::Undelegate {
        batch_id: 0u128,
        items: vec![("valoper1".to_string(), Uint128::from(1000u128))],
        reply_to: "some_reply_to".to_string(),
    };
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        msg,
    )
    .unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
        value: Binary::from(
            cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate {
                delegator_address: "ica_address".to_string(),
                validator_address: "valoper1".to_string(),
                amount: Some(cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                    denom: "remote_denom".to_string(),
                    amount: "1000".to_string(),
                }),
            }
            .encode_to_vec(),
        ),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        ))
    );
    let tx_state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("some_reply_to".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::Undelegate {
                batch_id: 0u128,
                interchain_account_id: "DROP".to_string(),
                denom: "remote_denom".to_string(),
                items: vec![("valoper1".to_string(), Uint128::from(1000u128))]
            })
        }
    );
}

#[test]
fn test_execute_redelegate_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Redelegate {
            validator_from: "validator_from".to_string(),
            validator_to: "validator_to".to_string(),
            amount: Uint128::from(0u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_redelegate_sender_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Redelegate {
            validator_from: "validator_from".to_string(),
            validator_to: "validator_to".to_string(),
            amount: Uint128::from(0u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_redelegate() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Redelegate {
            validator_from: "validator_from".to_string(),
            validator_to: "validator_to".to_string(),
            amount: Uint128::from(0u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 65536u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![drop_helpers::interchain::prepare_any_msg(
                    crate::proto::liquidstaking::staking::v1beta1::MsgBeginRedelegate {
                        delegator_address: puppeteer_base
                            .ica
                            .get_address(deps.as_mut().storage)
                            .unwrap(),
                        validator_src_address: "validator_from".to_string(),
                        validator_dst_address: "validator_to".to_string(),
                        amount: Some(crate::proto::cosmos::base::v1beta1::Coin {
                            denom: puppeteer_base
                                .config
                                .load(deps.as_mut().storage)
                                .unwrap()
                                .remote_denom,
                            amount: "0".to_string(),
                        }),
                    },
                    "/cosmos.staking.v1beta1.MsgBeginRedelegate",
                )
                .unwrap()],
                "".to_string(),
                100u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }],
                },
            )),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        }),
    );
}

#[test]
fn test_execute_tokenize_share_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::TokenizeShare {
            validator: "validator".to_string(),
            amount: Uint128::from(123u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_tokenize_share_sender_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::TokenizeShare {
            validator: "validator".to_string(),
            amount: Uint128::from(123u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_tokenize_share() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::TokenizeShare {
            validator: "validator".to_string(),
            amount: Uint128::from(123u64),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();
    let delegator = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 65536u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![drop_helpers::interchain::prepare_any_msg(
                    crate::proto::liquidstaking::staking::v1beta1::MsgTokenizeShares {
                        delegator_address: delegator.clone(),
                        validator_address: "validator".to_string(),
                        amount: Some(crate::proto::cosmos::base::v1beta1::Coin {
                            denom: puppeteer_base
                                .config
                                .load(deps.as_mut().storage)
                                .unwrap()
                                .remote_denom,
                            amount: "123".to_string(),
                        }),
                        tokenized_share_owner: delegator
                    },
                    "/cosmos.staking.v1beta1.MsgTokenizeShares",
                )
                .unwrap()],
                "".to_string(),
                100u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }],
                },
            )),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        }),
    );
}

#[test]
fn test_execute_redeem_shares_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
            items: vec![drop_puppeteer_base::state::RedeemShareItem {
                amount: Uint128::from(1000u128),
                remote_denom: "remote_denom".to_string(),
                local_denom: "local_denom".to_string(),
            }],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_redeeem_shares_sender_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
            items: vec![drop_puppeteer_base::state::RedeemShareItem {
                amount: Uint128::from(1000u128),
                remote_denom: "remote_denom".to_string(),
                local_denom: "local_denom".to_string(),
            }],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_redeem_share() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RedeemShares {
            items: vec![drop_puppeteer_base::state::RedeemShareItem {
                amount: Uint128::from(1000u128),
                remote_denom: "remote_denom".to_string(),
                local_denom: "local_denom".to_string(),
            }],
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();
    let any_msg = neutron_sdk::bindings::types::ProtobufAny {
        type_url: "/cosmos.staking.v1beta1.MsgRedeemTokensForShares".to_string(),
        value: Binary::from(
            crate::proto::liquidstaking::staking::v1beta1::MsgRedeemTokensforShares {
                amount: Some(crate::proto::cosmos::base::v1beta1::Coin {
                    denom: "remote_denom".to_string(),
                    amount: "1000".to_string(),
                }),
                delegator_address: "ica_address".to_string(),
            }
            .encode_to_vec(),
        ),
    };
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![any_msg],
                "".to_string(),
                100u64,
                get_standard_fees()
            )),
            ReplyMsg::SudoPayload.to_reply_id()
        )).add_attributes(vec![("action", "redeem_share"), ("items", "[RedeemShareItem { amount: Uint128(1000), remote_denom: \"remote_denom\", local_denom: \"local_denom\" }]")])
    );
    let tx_state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        tx_state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::InProgress,
            reply_to: Some("some_reply_to".to_string()),
            transaction: Some(drop_puppeteer_base::msg::Transaction::RedeemShares {
                interchain_account_id: "DROP".to_string(),
                items: vec![drop_puppeteer_base::state::RedeemShareItem {
                    amount: Uint128::from(1000u128),
                    remote_denom: "remote_denom".to_string(),
                    local_denom: "local_denom".to_string()
                }]
            })
        }
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer_not_idle() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_claim_rewards_and_optionaly_transfer() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::ClaimRewardsAndOptionalyTransfer {
            validators: vec!["validator1".to_string(), "validator2".to_string()],
            transfer: Some(drop_puppeteer_base::msg::TransferReadyBatchesMsg {
                batch_ids: vec![0u128, 1u128, 2u128],
                emergency: true,
                amount: Uint128::from(123u64),
                recipient: "some_recipient".to_string(),
            }),
            reply_to: "some_reply_to".to_string(),
        },
    )
    .unwrap();
    let ica_address = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 65536u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![
                    drop_helpers::interchain::prepare_any_msg(
                        cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
                            from_address: ica_address.clone(),
                            to_address: "some_recipient".to_string(),
                            amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                amount: "123".to_string(),
                                denom: puppeteer_base
                                    .config
                                    .load(deps.as_mut().storage)
                                    .unwrap()
                                    .remote_denom
                            }]
                        },
                        "/cosmos.bank.v1beta1.MsgSend",
                    )
                    .unwrap(),
                    drop_helpers::interchain::prepare_any_msg(
                        crate::proto::liquidstaking::distribution::v1beta1::MsgWithdrawDelegatorReward {
                            delegator_address: ica_address.clone(),
                            validator_address: "validator1".to_string()
                        },
                        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
                    )
                    .unwrap(),
                    drop_helpers::interchain::prepare_any_msg(
                        crate::proto::liquidstaking::distribution::v1beta1::MsgWithdrawDelegatorReward {
                            delegator_address: ica_address.clone(),
                            validator_address: "validator2".to_string()
                        },
                        "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward",
                    )
                    .unwrap()
                ],
                "".to_string(),
                100u64,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }],
                },
            )),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        }),
    );
}

#[test]
fn test_execute_register_balance_and_delegator_delegations_query_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery{
        validators: vec!["neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(); 2]
    },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_execute_register_balance_and_delegator_delegations_query_too_many_validators() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery{
        validators: vec!["neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(); u16::MAX as usize]
    },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Too many validators provided"
        ))
    );
}

#[test]
fn test_execute_register_balance_and_delegator_delegations_query() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_validators = vec!["neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(); 2];
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery{
        validators: msg_validators.clone()
    },
    )
    .unwrap();
    let puppeteer_config = puppeteer_base.config.load(deps.as_mut().storage).unwrap();
    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 196608u64,
            msg: cosmwasm_std::CosmosMsg::Custom(
                drop_helpers::icq::new_delegations_and_balance_query_msg(
                    puppeteer_config.connection_id,
                    puppeteer_ica,
                    puppeteer_config.remote_denom,
                    msg_validators,
                    puppeteer_config.update_period,
                    puppeteer_config.sdk_version.as_str()
                )
                .unwrap()
            ),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        })
    );
}

#[test]
fn test_execute_register_unbonding_delegations_query_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery{
        validators: vec!["neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(); 2]
    },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_execute_register_unbonding_delegations_query_too_many_validators() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterDelegatorUnbondingDelegationsQuery {
            validators: vec![
                "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string();
                u16::MAX as usize
            ],
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Too many validators provided"
        ))
    );
}

#[test]
fn test_execute_register_unbonding_delegations_query() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_validators = vec!["neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(); 2];
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterDelegatorUnbondingDelegationsQuery {
            validators: msg_validators.clone(),
        },
    )
    .unwrap();
    let puppeteer_config = puppeteer_base.config.load(deps.as_mut().storage).unwrap();
    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessages(
            msg_validators.into_iter().enumerate().map(|(i, validator)| {
                cosmwasm_std::SubMsg {
                    id: 327680u64 + i as u64,
                    msg: cosmwasm_std::CosmosMsg::Custom(
                        neutron_sdk::interchain_queries::v045::new_register_delegator_unbonding_delegations_query_msg(
                            puppeteer_config.connection_id.clone(),
                            puppeteer_ica.clone(),
                            vec![validator],
                            puppeteer_config.update_period
                        )
                        .unwrap()
                    ),
                    gas_limit: None,
                    reply_on: cosmwasm_std::ReplyOn::Success
                }
            })
        )
    );
}

#[test]
fn test_execute_register_non_native_rewards_balances_query_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_an_owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
            denoms: vec![],
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::OwnershipError(
            cw_ownable::OwnershipError::NotOwner
        )
    );
}

#[test]
fn test_execute_register_non_native_rewards_balances_query_empty_kv_queries() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_denoms = vec!["denom1".to_string(), "denom2".to_string()];

    let puppeteer_config = puppeteer_base.config.load(deps.as_mut().storage).unwrap();
    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
            denoms: msg_denoms.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 262144u64,
            msg: cosmwasm_std::CosmosMsg::Custom(
                drop_helpers::icq::new_multiple_balances_query_msg(
                    puppeteer_config.connection_id,
                    puppeteer_ica,
                    msg_denoms.clone(),
                    puppeteer_config.update_period
                )
                .unwrap()
            ),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success
        })
    )
}

#[test]
fn test_execute_register_non_native_rewards_balances_query_not_empty_kv_queries() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_denoms = vec!["denom1".to_string(), "denom2".to_string()];

    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            0u64,
            &KVQueryType::NonNativeRewardsBalances,
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
            denoms: msg_denoms.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 0u64,
            msg: cosmwasm_std::CosmosMsg::Custom(
                drop_helpers::icq::update_multiple_balances_query_msg(
                    0u64,
                    puppeteer_ica,
                    msg_denoms.clone()
                )
                .unwrap()
            ),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never
        })
    )
}

#[test]
fn test_execute_register_non_native_rewards_balances_query_has_non_native_rewards_balances() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_denoms = vec!["denom1".to_string(), "denom2".to_string()];

    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            0u64,
            &KVQueryType::NonNativeRewardsBalances,
        )
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            1u64,
            &KVQueryType::DelegationsAndBalance,
        )
        .unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
            denoms: msg_denoms.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 0u64,
            msg: cosmwasm_std::CosmosMsg::Custom(
                drop_helpers::icq::update_multiple_balances_query_msg(
                    0u64,
                    puppeteer_ica,
                    msg_denoms.clone()
                )
                .unwrap()
            ),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Never
        })
    )
}

#[test]
fn test_execute_register_non_native_rewards_balances_query_has_several_non_native_rewards_balances()
{
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(
        deps_mut.storage,
        deps_mut.api,
        Some(Addr::unchecked("owner").as_ref()),
    )
    .unwrap();
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let msg_denoms = vec!["denom1".to_string(), "denom2".to_string()];

    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            0u64,
            &KVQueryType::NonNativeRewardsBalances,
        )
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            1u64,
            &KVQueryType::DelegationsAndBalance,
        )
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            2u64,
            &KVQueryType::NonNativeRewardsBalances,
        )
        .unwrap();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            3u64,
            &KVQueryType::DelegationsAndBalance,
        )
        .unwrap();

    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterNonNativeRewardsBalancesQuery {
            denoms: msg_denoms.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessages(vec![
            cosmwasm_std::SubMsg {
                id: 0u64,
                msg: cosmwasm_std::CosmosMsg::Custom(
                    drop_helpers::icq::update_multiple_balances_query_msg(
                        0u64,
                        puppeteer_ica.clone(),
                        msg_denoms.clone()
                    )
                    .unwrap()
                ),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Never
            },
            cosmwasm_std::SubMsg {
                id: 0u64,
                msg: cosmwasm_std::CosmosMsg::Custom(
                    drop_helpers::icq::update_multiple_balances_query_msg(
                        2u64,
                        puppeteer_ica,
                        msg_denoms.clone()
                    )
                    .unwrap()
                ),
                gas_limit: None,
                reply_on: cosmwasm_std::ReplyOn::Never
            }
        ])
    )
}

#[test]
fn test_execute_ibc_transfer_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
            reason: IBCTransferReason::LSMShare,
            reply_to: "owner".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_ibc_transfer_many_coins() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "allowed_sender",
            &[
                cosmwasm_std::Coin {
                    denom: "token1".to_string(),
                    amount: Uint128::from(123u64),
                },
                cosmwasm_std::Coin {
                    denom: "token2".to_string(),
                    amount: Uint128::from(123u64),
                },
            ],
        ),
        drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
            reason: IBCTransferReason::LSMShare,
            reply_to: "owner".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::InvalidFunds {
            reason: "Only one coin is allowed".to_string()
        }
    )
}

#[test]
fn test_execute_ibc_transfer_no_coins() {
    let mut deps = mock_dependencies(&[]);
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
            reason: IBCTransferReason::LSMShare,
            reply_to: "owner".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::InvalidFunds {
            reason: "Only one coin is allowed".to_string()
        }
    )
}

#[test]
fn test_execute_ibc_transfer_not_idle() {
    let mut deps = mock_dependencies(&[]);
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
            reason: IBCTransferReason::LSMShare,
            reply_to: "owner".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_ibc_transfer_one_coin() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    let puppeteer_config = puppeteer_base.config.load(deps.as_mut().storage).unwrap();
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();

    let env = mock_env();
    let res = crate::contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(
            "allowed_sender",
            &[cosmwasm_std::Coin {
                denom: "uatom".to_string(),
                amount: Uint128::from(123u64),
            }],
        ),
        drop_staking_base::msg::puppeteer::ExecuteMsg::IBCTransfer {
            reason: IBCTransferReason::LSMShare,
            reply_to: "owner".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 131072u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                source_port: "port_id".to_string(),
                source_channel: "transfer_channel_id".to_string(),
                token: cosmwasm_std::Coin {
                    denom: "uatom".to_string(),
                    amount: Uint128::from(123u64),
                },
                sender: "cosmos2contract".to_string(),
                receiver: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                timeout_height: RequestPacketTimeoutHeight {
                    revision_height: None,
                    revision_number: None
                },
                timeout_timestamp: env
                    .block
                    .time
                    .plus_seconds(puppeteer_config.timeout)
                    .nanos(),
                memo: "".to_string(),
                fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }]
                }
            }),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success,
        })
    )
}

#[test]
fn test_execute_transfer_sender_is_not_allowed() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    base_init(&mut deps.as_mut());
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("not_allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
            items: vec![],
            reply_to: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::Std(StdError::generic_err(
            "Sender is not allowed"
        ))
    );
}

#[test]
fn test_execute_transfer_not_idle() {
    let mut deps = mock_dependencies(&[]);
    let pupeteer_base = base_init(&mut deps.as_mut());
    pupeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::InProgress,
                reply_to: Some("".to_string()),
                transaction: Some(drop_puppeteer_base::msg::Transaction::SetupProtocol {
                    interchain_account_id: "ica_address".to_string(),
                    delegate_grantee: "delegate_grantee".to_string(),
                    rewards_withdraw_address: "rewards_withdraw_address".to_string(),
                }),
            },
        )
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("allowed_sender", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
            items: vec![],
            reply_to: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
        },
    )
    .unwrap_err();
    assert_eq!(
        res,
        drop_puppeteer_base::error::ContractError::NeutronError(NeutronError::Std(
            cosmwasm_std::StdError::generic_err(
                "Transaction txState is not equal to expected: Idle".to_string()
            )
        ))
    );
}

#[test]
fn test_execute_transfer() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.add_custom_query_response(|_| {
        to_json_binary(&MinIbcFeeResponse {
            min_fee: get_standard_fees(),
        })
        .unwrap()
    });
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .ica
        .set_address(
            deps.as_mut().storage,
            "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
            "transfer".to_string(),
            "channel-0".to_string(),
        )
        .unwrap();
    let puppeteer_ica = puppeteer_base
        .ica
        .get_address(deps.as_mut().storage)
        .unwrap();
    let res = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info(
            "allowed_sender",
            &[cosmwasm_std::Coin {
                denom: "uatom".to_string(),
                amount: Uint128::from(123u64),
            }],
        ),
        drop_staking_base::msg::puppeteer::ExecuteMsg::Transfer {
            items: vec![
                (
                    "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                    cosmwasm_std::Coin {
                        denom: "uatom".to_string(),
                        amount: Uint128::from(123u64),
                    },
                ),
                (
                    "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
                    cosmwasm_std::Coin {
                        denom: "uatom".to_string(),
                        amount: Uint128::from(321u64),
                    },
                ),
            ],
            reply_to: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6".to_string(),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        cosmwasm_std::Response::new().add_submessage(cosmwasm_std::SubMsg {
            id: 65536u64,
            msg: cosmwasm_std::CosmosMsg::Custom(NeutronMsg::submit_tx(
                "connection_id".to_string(),
                "DROP".to_string(),
                vec![
                    drop_helpers::interchain::prepare_any_msg(
                        cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
                            from_address: puppeteer_ica.clone(),
                            to_address: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6"
                                .to_string(),
                            amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                amount: "123".to_string(),
                                denom: "uatom".to_string()
                            }]
                        },
                        "/cosmos.bank.v1beta1.MsgSend",
                    )
                    .unwrap(),
                    drop_helpers::interchain::prepare_any_msg(
                        cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend {
                            from_address: puppeteer_ica.clone(),
                            to_address: "neutron1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhufaa6"
                                .to_string(),
                            amount: vec![cosmos_sdk_proto::cosmos::base::v1beta1::Coin {
                                amount: "321".to_string(),
                                denom: "uatom".to_string()
                            }]
                        },
                        "/cosmos.bank.v1beta1.MsgSend",
                    )
                    .unwrap()
                ],
                "".to_string(),
                100,
                IbcFee {
                    recv_fee: vec![],
                    ack_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(100u64),
                    }],
                    timeout_fee: vec![cosmwasm_std::Coin {
                        denom: "untrn".to_string(),
                        amount: Uint128::from(200u64),
                    }]
                }
            )),
            gas_limit: None,
            reply_on: cosmwasm_std::ReplyOn::Success,
        })
    )
}

#[test]
fn test_sudo_response_tx_state_wrong() {
    // Test that the contract returns an error if the tx state is not in progress
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = SudoMsg::Response {
        request: neutron_sdk::sudo::msg::RequestPacket {
            sequence: Some(1u64),
            source_port: Some("source_port".to_string()),
            source_channel: Some("source_channel".to_string()),
            destination_port: Some("destination_port".to_string()),
            destination_channel: Some("destination_channel".to_string()),
            data: None,
            timeout_height: None,
            timeout_timestamp: None,
        },
        data: Binary::from(vec![]),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::Idle,
                reply_to: None,
                transaction: None,
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg);
    assert_eq!(
        res.unwrap_err(),
        NeutronError::Std(StdError::generic_err(
            "Transaction txState is not equal to expected: WaitingForAck"
        ))
    );
}

#[test]
fn test_sudo_delegations_and_balance_kv_query_result() {
    let mut deps = mock_dependencies(&[]);

    let query_id = 1u64;

    deps.querier
        .add_query_response(query_id, build_interchain_query_response());

    let puppeteer_base = base_init(&mut deps.as_mut());

    let msg = SudoMsg::KVQueryResult { query_id };
    let env = mock_env();
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            query_id,
            &KVQueryType::DelegationsAndBalance {},
        )
        .unwrap();

    puppeteer_base
        .delegations_and_balances_query_id_chunk
        .save(deps.as_mut().storage, query_id, &0)
        .unwrap();

    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(res, Response::new());

    let state = puppeteer_base
        .delegations_and_balances
        .load(&deps.storage, &123456)
        .unwrap();

    assert_eq!(
        state,
        BalancesAndDelegationsState {
            data: BalancesAndDelegations {
                balances: Balances {
                    coins: vec![coin(29558778, "stake")]
                },
                delegations: Delegations {
                    delegations: vec![
                        Delegation {
                            delegator: Addr::unchecked(
                                "cosmos1nujy3vl3rww3cy8tf8pdru5jp3f9ppmkadws553ck3qryg2tjanqt39xnv"
                            ),
                            validator: "cosmosvaloper1rndyjagfg0nsedl2uy5n92vssn8aj5n67t0nfx"
                                .to_string(),
                            amount: coin(13582465152, "stake")
                        },
                        Delegation {
                            delegator: Addr::unchecked(
                                "cosmos1nujy3vl3rww3cy8tf8pdru5jp3f9ppmkadws553ck3qryg2tjanqt39xnv"
                            ),
                            validator: "cosmosvaloper1gh4vzw9wsfgl2h37qqnetet0m4wrzm7v7x3j9x"
                                .to_string(),
                            amount: coin(13582465152, "stake")
                        }
                    ]
                }
            },
            remote_height: 123456,
            local_height: 12345,
            timestamp: Timestamp::from_nanos(1571797419879305533),
            collected_chunks: vec![0]
        }
    );
}

#[test]
fn test_sudo_response_ok() {
    let mut deps = mock_dependencies(&[]);

    deps.querier.add_stargate_query_response(
        "/ibc.core.channel.v1.Query/ChannelClientState",
        |_data| {
            to_json_binary(&ChannelClientStateResponse {
                identified_client_state: Some(IdentifiedClientState {
                    client_id: "07-tendermint-0".to_string(),
                    client_state: ClientState {
                        chain_id: "test-1".to_string(),
                        type_url: "type_url".to_string(),
                        trust_level: Fraction {
                            numerator: Uint64::from(1u64),
                            denominator: Uint64::from(3u64),
                        },
                        trusting_period: Some("1000".to_string()),
                        unbonding_period: Some("1500".to_string()),
                        max_clock_drift: Some("1000".to_string()),
                        frozen_height: None,
                        latest_height: Some(Height {
                            revision_number: Uint64::from(0u64),
                            revision_height: Uint64::from(54321u64),
                        }),
                        proof_specs: vec![],
                        upgrade_path: vec![],
                        allow_update_after_expiry: true,
                        allow_update_after_misbehaviour: true,
                    },
                }),
                proof: None,
                proof_height: Height {
                    revision_number: Uint64::from(0u64),
                    revision_height: Uint64::from(33333u64),
                },
            })
            .unwrap()
        },
    );

    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Response {
        request: request.clone(),
        data: Binary::from(vec![]),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Success(
                        drop_puppeteer_base::msg::ResponseHookSuccessMsg {
                            request_id: 1,
                            local_height: 12345,
                            remote_height: 54321,
                            request,
                            transaction,
                            answers: vec![drop_puppeteer_base::msg::ResponseAnswer::IBCTransfer(
                                drop_puppeteer_base::proto::MsgIBCTransfer {}
                            )]
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(
                Event::new("puppeteer-sudo-response")
                    .add_attributes(vec![("action", "sudo_response"), ("request_id", "1")])
            )
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string(),
            port_id: "port".to_string(),
            channel_id: "channel".to_string(),
        }
    );
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

#[test]
fn test_sudo_response_error() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Error {
        request: request.clone(),
        details: "some shit happened".to_string(),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Error(
                        drop_puppeteer_base::msg::ResponseHookErrorMsg {
                            request_id: 1,
                            request,
                            transaction,
                            details: "some shit happened".to_string()
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(Event::new("puppeteer-sudo-error").add_attributes(vec![
                ("action", "sudo_error"),
                ("request_id", "1"),
                ("details", "some shit happened")
            ]))
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string(),
            port_id: "port".to_string(),
            channel_id: "channel".to_string(),
        }
    );
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

#[test]
fn test_sudo_open_ack() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let msg = SudoMsg::OpenAck {
        port_id: "port_id_1".to_string(),
        channel_id: "channel_1".to_string(),
        counterparty_channel_id: "counterparty_channel_id_1".to_string(),
        counterparty_version: "{\"version\": \"1\",\"controller_connection_id\": \"connection_id\",\"host_connection_id\": \"host_connection_id\",\"address\": \"ica_address\",\"encoding\": \"amino\",\"tx_type\": \"cosmos-sdk/MsgSend\"}".to_string(),
    };
    let env = mock_env();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(res, Response::new());
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        ica,
        drop_helpers::ica::IcaState::Registered {
            ica_address: "ica_address".to_string(),
            port_id: "port_id_1".to_string(),
            channel_id: "channel_1".to_string(),
        }
    );
}

#[test]
fn test_sudo_response_timeout() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let request = neutron_sdk::sudo::msg::RequestPacket {
        sequence: Some(1u64),
        source_port: Some("source_port".to_string()),
        source_channel: Some("source_channel".to_string()),
        destination_port: Some("destination_port".to_string()),
        destination_channel: Some("destination_channel".to_string()),
        data: None,
        timeout_height: None,
        timeout_timestamp: None,
    };
    let transaction = drop_puppeteer_base::msg::Transaction::IBCTransfer {
        denom: "remote_denom".to_string(),
        amount: 1000u128,
        recipient: "recipient".to_string(),
        reason: drop_puppeteer_base::msg::IBCTransferReason::Stake,
    };
    let msg = SudoMsg::Timeout {
        request: request.clone(),
    };
    let env = mock_env();
    puppeteer_base
        .tx_state
        .save(
            deps.as_mut().storage,
            &drop_puppeteer_base::state::TxState {
                seq_id: None,
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: Some("reply_to_contract".to_string()),
                transaction: Some(transaction.clone()),
            },
        )
        .unwrap();
    let res = crate::contract::sudo(deps.as_mut(), env, msg).unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute {
                contract_addr: "reply_to_contract".to_string(),
                msg: to_json_binary(&drop_staking_base::msg::core::ExecuteMsg::PuppeteerHook(
                    Box::new(drop_puppeteer_base::msg::ResponseHookMsg::Error(
                        drop_puppeteer_base::msg::ResponseHookErrorMsg {
                            request_id: 1,
                            request,
                            transaction,
                            details: "Timeout".to_string()
                        }
                    ))
                ))
                .unwrap(),
                funds: vec![]
            }))
            .add_event(
                Event::new("puppeteer-sudo-timeout")
                    .add_attributes(vec![("action", "sudo_timeout"), ("request_id", "1"),])
            )
    );
    let ica = puppeteer_base.ica.load(deps.as_ref().storage).unwrap();
    assert_eq!(ica, drop_helpers::ica::IcaState::Timeout);
    let state = puppeteer_base.tx_state.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        drop_puppeteer_base::state::TxState {
            seq_id: None,
            status: drop_puppeteer_base::state::TxStateStatus::Idle,
            reply_to: None,
            transaction: None,
        }
    );
}

#[test]
fn test_reply_sudo_payload() {
    let mut deps = mock_dependencies(&[]);
    {
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::SUDO_PAYLOAD,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: None,
                }),
            },
        )
        .unwrap_err();
        assert_eq!(res, StdError::generic_err("no result"))
    }
    {
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::SUDO_PAYLOAD,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: Some(Binary::from(
                        "{\"sequence_id\":0,\"channel\":\"channel-0\"}".as_bytes(),
                    )),
                }),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            StdError::NotFound {
                kind: format!(
                    "type: drop_puppeteer_base::state::TxState; key: {:X?}",
                    "sudo_payload".as_bytes()
                )
            }
        )
    }
    {
        let puppeteer_base = base_init(&mut deps.as_mut());
        puppeteer_base
            .tx_state
            .save(
                deps.as_mut().storage,
                &drop_puppeteer_base::state::TxState {
                    status: drop_puppeteer_base::state::TxStateStatus::Idle,
                    seq_id: None,
                    transaction: None,
                    reply_to: None,
                },
            )
            .unwrap();
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::SUDO_PAYLOAD,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: Some(Binary::from(
                        "{\"sequence_id\":0,\"channel\":\"channel-0\"}".as_bytes(),
                    )),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new().add_event(
                cosmwasm_std::Event::new("puppeteer-base-sudo-tx-payload-received".to_string())
                    .add_attributes(vec![
                        cosmwasm_std::attr("channel_id".to_string(), "channel-0".to_string()),
                        cosmwasm_std::attr("seq_id".to_string(), "0".to_string())
                    ])
            )
        );
        let tx_state = puppeteer_base.tx_state.load(deps.as_mut().storage).unwrap();
        assert_eq!(
            tx_state,
            drop_puppeteer_base::state::TxState {
                seq_id: Some(0),
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: None,
                transaction: None,
            }
        );
    }
}

#[test]
fn test_reply_ibc_transfer() {
    let mut deps = mock_dependencies(&[]);
    {
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::SUDO_PAYLOAD,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: None,
                }),
            },
        )
        .unwrap_err();
        assert_eq!(res, StdError::generic_err("no result"))
    }
    {
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::SUDO_PAYLOAD,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: Some(
                        to_json_binary(&neutron_sdk::bindings::msg::MsgIbcTransferResponse {
                            sequence_id: 0,
                            channel: "channel-0".to_string(),
                        })
                        .unwrap(),
                    ),
                }),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            StdError::NotFound {
                kind: format!(
                    "type: drop_puppeteer_base::state::TxState; key: {:X?}",
                    "sudo_payload".as_bytes()
                )
            }
        )
    }
    {
        let puppeteer_base = base_init(&mut deps.as_mut());
        puppeteer_base
            .tx_state
            .save(
                deps.as_mut().storage,
                &drop_puppeteer_base::state::TxState {
                    status: drop_puppeteer_base::state::TxStateStatus::Idle,
                    seq_id: None,
                    transaction: None,
                    reply_to: None,
                },
            )
            .unwrap();
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::IBC_TRANSFER,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: Some(
                        to_json_binary(&neutron_sdk::bindings::msg::MsgIbcTransferResponse {
                            sequence_id: 0,
                            channel: "channel-0".to_string(),
                        })
                        .unwrap(),
                    ),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new().add_event(
                cosmwasm_std::Event::new(
                    "puppeteer-base-sudo-ibc-transfer-payload-received".to_string()
                )
                .add_attributes(vec![
                    cosmwasm_std::attr("channel_id".to_string(), "channel-0".to_string()),
                    cosmwasm_std::attr("seq_id".to_string(), "0".to_string())
                ])
            )
        );
        let tx_state = puppeteer_base.tx_state.load(deps.as_mut().storage).unwrap();
        assert_eq!(
            tx_state,
            drop_puppeteer_base::state::TxState {
                seq_id: Some(0),
                status: drop_puppeteer_base::state::TxStateStatus::WaitingForAck,
                reply_to: None,
                transaction: None,
            }
        );
    }
}

#[test]
fn test_reply_kv_delegations_and_balance() {
    let response_id: u64 = 0;
    for i in drop_puppeteer_base::state::reply_msg::KV_DELEGATIONS_AND_BALANCE_LOWER_BOUND
        ..(drop_puppeteer_base::state::reply_msg::KV_DELEGATIONS_AND_BALANCE_UPPER_BOUND + 1)
    {
        let mut deps = mock_dependencies(&[]);
        {
            let res = crate::contract::reply(
                deps.as_mut().into_empty(),
                mock_env(),
                cosmwasm_std::Reply {
                    id: i,
                    result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                        events: vec![],
                        data: None,
                    }),
                },
            )
            .unwrap_err();
            assert_eq!(res, StdError::generic_err("no result"))
        }
        {
            let puppeteer_base = base_init(&mut deps.as_mut());
            let res = crate::contract::reply(
                deps.as_mut().into_empty(),
                mock_env(),
                cosmwasm_std::Reply {
                    id: i,
                    result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                        events: vec![],
                        data: Some(
                            to_json_binary(
                                &neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse {
                                    id: response_id,
                                },
                            )
                            .unwrap(),
                        ),
                    }),
                },
            )
            .unwrap();
            assert_eq!(res, cosmwasm_std::Response::new());
            let delegations_and_balances_query_id_chunk: u16 = puppeteer_base
                .delegations_and_balances_query_id_chunk
                .load(deps.as_mut().storage, response_id)
                .unwrap();
            assert_eq!(delegations_and_balances_query_id_chunk, i as u16);
            let kv_query = puppeteer_base
                .kv_queries
                .load(deps.as_mut().storage, response_id)
                .unwrap();
            assert_eq!(
                kv_query,
                drop_staking_base::state::puppeteer::KVQueryType::DelegationsAndBalance
            );
        }
    }
}

#[test]
fn test_reply_kv_non_native_rewards_balances() {
    let mut deps = mock_dependencies(&[]);
    {
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::KV_NON_NATIVE_REWARDS_BALANCES,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: None,
                }),
            },
        )
        .unwrap_err();
        assert_eq!(res, StdError::generic_err("no result"))
    }
    {
        let puppeteer_base = base_init(&mut deps.as_mut());
        let res = crate::contract::reply(
            deps.as_mut().into_empty(),
            mock_env(),
            cosmwasm_std::Reply {
                id: drop_puppeteer_base::state::reply_msg::KV_NON_NATIVE_REWARDS_BALANCES,
                result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                    events: vec![],
                    data: Some(
                        to_json_binary(
                            &neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse {
                                id: 0u64,
                            },
                        )
                        .unwrap(),
                    ),
                }),
            },
        )
        .unwrap();
        assert_eq!(
            res,
            cosmwasm_std::Response::new().add_event(
                cosmwasm_std::Event::new(
                    "puppeteer-base-sudo-kv-query-payload-received".to_string()
                )
                .add_attribute("query_id".to_string(), "0".to_string())
            )
        );
        let kv_query = puppeteer_base
            .kv_queries
            .load(deps.as_mut().storage, 0u64)
            .unwrap();
        assert_eq!(
            kv_query,
            drop_staking_base::state::puppeteer::KVQueryType::NonNativeRewardsBalances
        )
    }
}

#[test]
fn test_reply_kv_unbonding_delegations() {
    let response_id: u64 = 0;
    for i in drop_puppeteer_base::state::reply_msg::KV_UNBONDING_DELEGATIONS_LOWER_BOUND
        ..(drop_puppeteer_base::state::reply_msg::KV_UNBONDING_DELEGATIONS_UPPER_BOUND + 1)
    {
        let mut deps = mock_dependencies(&[]);
        {
            let _ = crate::contract::reply(
                deps.as_mut().into_empty(),
                mock_env(),
                cosmwasm_std::Reply {
                    id: i,
                    result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                        events: vec![],
                        data: Some(
                            to_json_binary(
                                &neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse {
                                    id: response_id,
                                },
                            )
                            .unwrap(),
                        ),
                    }),
                },
            )
            .unwrap_err();
        }
        {
            let puppeteer_base = base_init(&mut deps.as_mut());
            puppeteer_base
                .unbonding_delegations_reply_id_storage
                .save(
                    deps.as_mut().storage,
                    i as u16,
                    &drop_puppeteer_base::state::UnbondingDelegation {
                        validator_address: "validator".to_string(),
                        query_id: 0u64,
                        unbonding_delegations: vec![],
                        last_updated_height: 0u64,
                    },
                )
                .unwrap();
            let res = crate::contract::reply(
                deps.as_mut().into_empty(),
                mock_env(),
                cosmwasm_std::Reply {
                    id: i,
                    result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                        events: vec![],
                        data: None,
                    }),
                },
            )
            .unwrap_err();
            assert_eq!(res, StdError::generic_err("no result"))
        }
        {
            let puppeteer_base = base_init(&mut deps.as_mut());
            puppeteer_base
                .unbonding_delegations_reply_id_storage
                .save(
                    deps.as_mut().storage,
                    i as u16,
                    &drop_puppeteer_base::state::UnbondingDelegation {
                        validator_address: "validator".to_string(),
                        query_id: 0u64,
                        unbonding_delegations: vec![],
                        last_updated_height: 0u64,
                    },
                )
                .unwrap();
            let res = crate::contract::reply(
                deps.as_mut().into_empty(),
                mock_env(),
                cosmwasm_std::Reply {
                    id: i,
                    result: cosmwasm_std::SubMsgResult::Ok(cosmwasm_std::SubMsgResponse {
                        events: vec![],
                        data: Some(
                            to_json_binary(
                                &neutron_sdk::bindings::msg::MsgRegisterInterchainQueryResponse {
                                    id: response_id,
                                },
                            )
                            .unwrap(),
                        ),
                    }),
                },
            )
            .unwrap();
            assert_eq!(res, cosmwasm_std::Response::new());
            let _ = puppeteer_base
                .unbonding_delegations_reply_id_storage
                .load(deps.as_mut().storage, 0u16)
                .unwrap_err();
            let unbonding_delegation = puppeteer_base
                .unbonding_delegations
                .load(deps.as_mut().storage, "validator")
                .unwrap();
            assert_eq!(
                unbonding_delegation,
                drop_puppeteer_base::state::UnbondingDelegation {
                    validator_address: "validator".to_string(),
                    query_id: response_id,
                    unbonding_delegations: vec![],
                    last_updated_height: 0u64,
                }
            );
            let kv_query = puppeteer_base
                .kv_queries
                .load(deps.as_mut().storage, response_id)
                .unwrap();
            assert_eq!(
                kv_query,
                drop_staking_base::state::puppeteer::KVQueryType::UnbondingDelegations
            )
        }
    }
}

#[test]
#[allow(deprecated)]
fn test_get_answers_from_msg_data() {
    let deps = mock_dependencies(&[]);
    {
        let res = crate::contract::get_answers_from_msg_data(
            deps.as_ref(),
            cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData {
                msg_responses: vec![],
                data: vec![cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                    msg_type: "incorrect_cosmos_msg_type".to_string(),
                    data: vec![],
                }],
            },
        )
        .unwrap();
        assert_eq!(
            res,
            vec![drop_puppeteer_base::msg::ResponseAnswer::UnknownResponse {}]
        );
    }
    {
        let res = crate::contract::get_answers_from_msg_data(
            deps.as_ref(),
            cosmos_sdk_proto::cosmos::base::abci::v1beta1::TxMsgData {
                msg_responses: vec![],
                data: vec![
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.staking.v1beta1.MsgDelegate".to_string(),
                        data: (crate::proto::liquidstaking::staking::v1beta1::MsgDelegateResponse {})
                            .encode_to_vec(),
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.staking.v1beta1.MsgUndelegate".to_string(),
                        data: (crate::proto::liquidstaking::staking::v1beta1::MsgUndelegateResponse {
                            completion_time: None,
                        })
                        .encode_to_vec(),
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.staking.v1beta1.MsgTokenizeShares".to_string(),
                        data:
                            (crate::proto::liquidstaking::staking::v1beta1::MsgTokenizeSharesResponse {
                                amount: None,
                            })
                            .encode_to_vec(),
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.staking.v1beta1.MsgBeginRedelegate".to_string(),
                        data: (crate::proto::liquidstaking::staking::v1beta1::MsgBeginRedelegateResponse {
                            completion_time: None
                        })
                            .encode_to_vec(),
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
                        data: (cosmos_sdk_proto::cosmos::authz::v1beta1::MsgGrantResponse {}).encode_to_vec()
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.staking.v1beta1.MsgRedeemTokensForShares".to_string(),
                        data: (crate::proto::liquidstaking::staking::v1beta1::MsgRedeemTokensforSharesResponse {
                            amount: None
                        }).encode_to_vec()
                    },
                    cosmos_sdk_proto::cosmos::base::abci::v1beta1::MsgData {
                        msg_type: "/cosmos.bank.v1beta1.MsgSend".to_string(),
                        data: (cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSendResponse {}).encode_to_vec()
                    },
                ],
            },
        ).unwrap();
        assert_eq!(
            res,
            vec![
                drop_puppeteer_base::msg::ResponseAnswer::DelegateResponse(
                    drop_puppeteer_base::proto::MsgDelegateResponse {}
                ),
                drop_puppeteer_base::msg::ResponseAnswer::UndelegateResponse(
                    drop_puppeteer_base::proto::MsgUndelegateResponse {
                        completion_time: None,
                    }
                ),
                drop_puppeteer_base::msg::ResponseAnswer::TokenizeSharesResponse(
                    drop_puppeteer_base::proto::MsgTokenizeSharesResponse { amount: None }
                ),
                drop_puppeteer_base::msg::ResponseAnswer::BeginRedelegateResponse(
                    drop_puppeteer_base::proto::MsgBeginRedelegateResponse {
                        completion_time: None
                    }
                ),
                drop_puppeteer_base::msg::ResponseAnswer::GrantDelegateResponse(
                    drop_puppeteer_base::proto::MsgGrantResponse {},
                ),
                drop_puppeteer_base::msg::ResponseAnswer::RedeemTokensforSharesResponse(
                    drop_puppeteer_base::proto::MsgRedeemTokensforSharesResponse { amount: None }
                ),
                drop_puppeteer_base::msg::ResponseAnswer::TransferResponse(
                    drop_puppeteer_base::proto::MsgSendResponse {}
                )
            ]
        );
    }
}

mod register_delegations_and_balance_query {
    use cosmwasm_std::{testing::MockApi, MemoryStorage, OwnedDeps, StdResult};
    use drop_helpers::testing::WasmMockQuerier;
    use drop_puppeteer_base::error::ContractError;

    use super::*;

    fn setup(
        owner: Option<&str>,
    ) -> (
        OwnedDeps<MemoryStorage, MockApi, WasmMockQuerier, NeutronQuery>,
        PuppeteerBase<'static, drop_staking_base::state::puppeteer::Config, KVQueryType>,
    ) {
        let mut deps = mock_dependencies(&[]);
        let puppeteer_base = base_init(&mut deps.as_mut());
        let deps_mut = deps.as_mut();
        cw_ownable::initialize_owner(
            deps_mut.storage,
            deps_mut.api,
            Some(Addr::unchecked(owner.unwrap_or("owner")).as_ref()),
        )
        .unwrap();
        (deps, puppeteer_base)
    }

    #[test]
    fn non_owner() {
        let (mut deps, _puppeteer_base) = setup(None);
        let env = mock_env();
        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery { validators: vec![] } ;
        let res = crate::contract::execute(deps.as_mut(), env, mock_info("not_owner", &[]), msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            ContractError::OwnershipError(cw_ownable::OwnershipError::NotOwner)
        );
    }

    #[test]
    fn too_many_validators() {
        let (mut deps, _puppeteer_base) = setup(None);
        let env = mock_env();
        let mut validators = vec![];
        for i in 0..=65536u32 {
            validators.push(format!("valoper{}", i));
        }

        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(deps.as_mut(), env, mock_info("owner", &[]), msg);
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            ContractError::Std(StdError::generic_err("Too many validators provided"))
        );
    }

    #[test]
    fn happy_path_validators_count_less_than_chunk_size() {
        let (mut deps, puppeteer_base) =
            setup(Some("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"));
        let env = mock_env();
        let validators = vec![
            "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
            "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
        ];
        puppeteer_base
            .ica
            .set_address(
                deps.as_mut().storage,
                "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud",
                "port",
                "channel",
            )
            .unwrap();

        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(
            deps.as_mut(),
            env,
            mock_info("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2", &[]),
            msg,
        )
        .unwrap();
        assert_eq!(
            res,
            Response::new().add_submessages(vec![SubMsg::reply_on_success(
                drop_helpers::icq::new_delegations_and_balance_query_msg(
                    "connection_id".to_string(),
                    "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                    "remote_denom".to_string(),
                    vec![
                        "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
                        "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
                    ],
                    60,
                    "0.47.0",
                )
                .unwrap(),
                ReplyMsg::KvDelegationsAndBalance { i: 0 }.to_reply_id(),
            )])
        );
    }

    #[test]
    fn happy_path_validators_count_more_than_chunk_size() {
        let (mut deps, puppeteer_base) =
            setup(Some("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2"));
        let env = mock_env();
        let validators = vec![
            "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
            "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
            "cosmos15tuf2ewxle6jj6eqd4jm579vpahydzwdsvkrhn".to_string(),
        ];
        puppeteer_base
            .ica
            .set_address(
                deps.as_mut().storage,
                "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud",
                "port",
                "channel",
            )
            .unwrap();
        puppeteer_base
            .delegations_and_balances_query_id_chunk
            .save(deps.as_mut().storage, 1, &2)
            .unwrap();
        puppeteer_base
            .delegations_and_balances_query_id_chunk
            .save(deps.as_mut().storage, 2, &3)
            .unwrap();
        let msg = drop_staking_base::msg::puppeteer::ExecuteMsg::RegisterBalanceAndDelegatorDelegationsQuery {
            validators
        };
        let res = crate::contract::execute(
            deps.as_mut(),
            env,
            mock_info("neutron1m9l358xunhhwds0568za49mzhvuxx9ux8xafx2", &[]),
            msg,
        )
        .unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_messages(vec![
                    NeutronMsg::remove_interchain_query(1),
                    NeutronMsg::remove_interchain_query(2)
                ])
                .add_submessages(vec![
                    SubMsg::reply_on_success(
                        drop_helpers::icq::new_delegations_and_balance_query_msg(
                            "connection_id".to_string(),
                            "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                            "remote_denom".to_string(),
                            vec![
                                "cosmos1jy7lsk5pk38zjfnn6nt6qlaphy9uejn4hu65xa".to_string(),
                                "cosmos14xcrdjwwxtf9zr7dvaa97wy056se6r5e8q68mw".to_string(),
                            ],
                            60,
                            "0.47.0",
                        )
                        .unwrap(),
                        ReplyMsg::KvDelegationsAndBalance { i: 0 }.to_reply_id(),
                    ),
                    SubMsg::reply_on_success(
                        drop_helpers::icq::new_delegations_and_balance_query_msg(
                            "connection_id".to_string(),
                            "cosmos1m9l358xunhhwds0568za49mzhvuxx9uxre5tud".to_string(),
                            "remote_denom".to_string(),
                            vec!["cosmos15tuf2ewxle6jj6eqd4jm579vpahydzwdsvkrhn".to_string(),],
                            60,
                            "0.47.0",
                        )
                        .unwrap(),
                        ReplyMsg::KvDelegationsAndBalance { i: 1 }.to_reply_id(),
                    )
                ])
        );
        assert_eq!(
            puppeteer_base
                .delegations_and_balances_query_id_chunk
                .keys(
                    deps.as_ref().storage,
                    None,
                    None,
                    cosmwasm_std::Order::Ascending
                )
                .collect::<StdResult<Vec<u64>>>()
                .unwrap()
                .len(),
            0
        )
    }
}

fn get_base_config() -> Config {
    Config {
        delegations_queries_chunk_size: 2u32,
        port_id: "port_id".to_string(),
        connection_id: "connection_id".to_string(),
        update_period: 60u64,
        remote_denom: "remote_denom".to_string(),
        allowed_senders: vec![Addr::unchecked("allowed_sender")],
        transfer_channel_id: "transfer_channel_id".to_string(),
        sdk_version: "0.47.10".to_string(),
        timeout: 100u64,
    }
}

fn base_init(deps_mut: &mut DepsMut<NeutronQuery>) -> PuppeteerBase<'static, Config, KVQueryType> {
    let puppeteer_base = Puppeteer::default();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    puppeteer_base
        .config
        .save(deps_mut.storage, &get_base_config())
        .unwrap();
    puppeteer_base
        .ica
        .set_address(deps_mut.storage, "ica_address", "port", "channel")
        .unwrap();
    puppeteer_base
}

fn get_standard_fees() -> IbcFee {
    IbcFee {
        recv_fee: vec![],
        ack_fee: coins(100, "untrn"),
        timeout_fee: coins(200, "untrn"),
    }
}

#[test]
fn test_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    let deps_mut = deps.as_mut();
    cw_ownable::initialize_owner(deps_mut.storage, deps_mut.api, Some("owner")).unwrap();
    let _ = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: Some(cw_ownable::Expiration::Never {}),
            },
        ),
    )
    .unwrap();
    let _ = crate::contract::execute(
        deps.as_mut(),
        mock_env(),
        mock_info("new_owner", &[]),
        drop_staking_base::msg::puppeteer::ExecuteMsg::UpdateOwnership(
            cw_ownable::Action::AcceptOwnership {},
        ),
    )
    .unwrap();
    let query_res: cw_ownable::Ownership<cosmwasm_std::Addr> = from_json(
        crate::contract::query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Ownership {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        cw_ownable::Ownership {
            owner: Some(cosmwasm_std::Addr::unchecked("new_owner".to_string())),
            pending_expiry: None,
            pending_owner: None
        }
    );
}

#[test]
fn test_query_kv_query_ids() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .kv_queries
        .save(
            deps.as_mut().storage,
            0u64,
            &KVQueryType::NonNativeRewardsBalances,
        )
        .unwrap();
    let query_res: Vec<(u64, KVQueryType)> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::KVQueryIds {},
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        vec![(0u64, KVQueryType::NonNativeRewardsBalances)]
    );
}

#[test]
fn test_query_extension_delegations_none() {
    let deps = mock_dependencies(&[]);
    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations {
                delegations: vec![],
            },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_delegations_some() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .last_complete_delegations_and_balances_key
        .save(deps.as_mut().storage, &0u64)
        .unwrap();
    let delegations = vec![
        Delegation {
            delegator: Addr::unchecked("delegator1"),
            validator: "validator1".to_string(),
            amount: cosmwasm_std::Coin::new(100, "denom1"),
        },
        Delegation {
            delegator: Addr::unchecked("delegator2"),
            validator: "validator2".to_string(),
            amount: cosmwasm_std::Coin::new(100, "denom2"),
        },
    ];
    puppeteer_base
        .delegations_and_balances
        .save(
            deps.as_mut().storage,
            &0u64,
            &BalancesAndDelegationsState {
                data: BalancesAndDelegations {
                    balances: Balances { coins: vec![] },
                    delegations: Delegations {
                        delegations: delegations.clone(),
                    },
                },
                remote_height: 123u64,
                local_height: 123u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::DelegationsResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Delegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::DelegationsResponse {
            delegations: Delegations {
                delegations: delegations,
            },
            remote_height: 123u64,
            local_height: 123u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_balances_none() {
    let deps = mock_dependencies(&[]);
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins: vec![] },
            remote_height: 0,
            local_height: 0,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_extension_balances_some() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    puppeteer_base
        .last_complete_delegations_and_balances_key
        .save(deps.as_mut().storage, &0u64)
        .unwrap();
    let coins = vec![
        cosmwasm_std::Coin::new(123u128, "denom1".to_string()),
        cosmwasm_std::Coin::new(123u128, "denom2".to_string()),
    ];
    puppeteer_base
        .delegations_and_balances
        .save(
            deps.as_mut().storage,
            &0u64,
            &BalancesAndDelegationsState {
                data: BalancesAndDelegations {
                    balances: Balances {
                        coins: coins.clone(),
                    },
                    delegations: Delegations {
                        delegations: vec![],
                    },
                },
                remote_height: 123u64,
                local_height: 123u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::Balances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins: coins },
            remote_height: 123u64,
            local_height: 123u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_query_non_native_rewards_balances() {
    let mut deps = mock_dependencies(&[]);
    let coins = vec![
        cosmwasm_std::Coin::new(123u128, "denom1".to_string()),
        cosmwasm_std::Coin::new(123u128, "denom2".to_string()),
    ];
    NON_NATIVE_REWARD_BALANCES
        .save(
            deps.as_mut().storage,
            &BalancesAndDelegationsState {
                data: drop_staking_base::msg::puppeteer::MultiBalances {
                    coins: coins.clone(),
                },
                remote_height: 1u64,
                local_height: 2u64,
                timestamp: Timestamp::default(),
                collected_chunks: vec![],
            },
        )
        .unwrap();
    let query_res: drop_staking_base::msg::puppeteer::BalancesResponse = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::NonNativeRewardsBalances {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        query_res,
        drop_staking_base::msg::puppeteer::BalancesResponse {
            balances: Balances { coins: coins },
            remote_height: 1u64,
            local_height: 2u64,
            timestamp: Timestamp::default(),
        }
    );
}

#[test]
fn test_unbonding_delegations() {
    let mut deps = mock_dependencies(&[]);
    let puppeteer_base = base_init(&mut deps.as_mut());
    let unbonding_delegations = vec![
        drop_puppeteer_base::state::UnbondingDelegation {
            validator_address: "validator_address1".to_string(),
            query_id: 1u64,
            unbonding_delegations: vec![
                neutron_sdk::interchain_queries::v047::types::UnbondingEntry {
                    balance: Uint128::from(0u64),
                    completion_time: None,
                    creation_height: 0u64,
                    initial_balance: Uint128::from(0u64),
                },
            ],
            last_updated_height: 0u64,
        },
        drop_puppeteer_base::state::UnbondingDelegation {
            validator_address: "validator_address2".to_string(),
            query_id: 2u64,
            unbonding_delegations: vec![],
            last_updated_height: 0u64,
        },
    ];
    puppeteer_base
        .unbonding_delegations
        .save(deps.as_mut().storage, "key1", &unbonding_delegations[0])
        .unwrap();
    puppeteer_base
        .unbonding_delegations
        .save(deps.as_mut().storage, "key2", &unbonding_delegations[1])
        .unwrap();
    let query_res: Vec<drop_puppeteer_base::state::UnbondingDelegation> = from_json(
        query(
            deps.as_ref(),
            mock_env(),
            drop_puppeteer_base::msg::QueryMsg::Extension {
                msg: drop_staking_base::msg::puppeteer::QueryExtMsg::UnbondingDelegations {},
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(query_res, unbonding_delegations);
}
