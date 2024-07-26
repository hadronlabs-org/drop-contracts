module runs {

    // ************************************************************************************************
    // The runs module showcases interactions with the module by executing sequences of actons
    // and inspecting the state at interesting points.
    // ************************************************************************************************

    import drop.* from "./drop"
    import constants.* from "./constants"
    import dropEvolution.* from "./dropEvolution"
    import simulationValues.* from "./simulationValues"
    import Dec.* from "./dec"
    
    run offchain_component = 
        executeICAAction(true, 20)
            .then(receiveICAResponseAction(20))
            .then(remoteQueryAction)

    run sample_run = 
        init.then(userStakeAction(200))
            .then(TransferFundsToDelegateAction)
            .then(tickAction)            
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })

    // The run that encodes two user's stakes and corresponding successful unstakes.
    run two_stakes_two_unstakes_run = 
        sample_run
            .then(userStakeAction(200))
            .then(TransferFundsToDelegateAction)
            .then(all{
                assert(state.dropState.delegatorContractICABalance == 200),
                state'=state
            })
            .then(tickAction)
            .then(offchain_component)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(userUnstakeAction(100))
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_UNDELEGATE),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(userUnstakeAction(150))
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(advanceTimeAction(UNBONDING_PERIOD))
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(receiveFromPumpAction)
            .then(userWithdrawAction({ batch: 0, expected_amount: 100, id: 0, received_amount: -1 }))
            .then(
                all{
                    assert(
                        state.envState.consumedNfts == 
                        Set({ batch: 0, expected_amount: 100, id: 0, received_amount: 100 })
                        ),
                    state' = state
            })
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_UNDELEGATE),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(advanceTimeAction(UNBONDING_PERIOD))            
            .then(tickAction)    
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })        
            .then(offchain_component)
            .then(tickAction)          
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })    
            .then(receiveFromPumpAction)
            .then(userWithdrawAction({ batch: 1, expected_amount: 150, id: 1, received_amount: -1 }))
            .then(
                all{
                    assert(
                        state.envState.consumedNfts == 
                        Set({ batch: 0, expected_amount: 100, id: 0, received_amount: 100 },{ batch: 1, expected_amount: 150, id: 1, received_amount: 150 })
                        ),
                    state' = state
            })

    
    // This run encodes a user's stake, a successful unstake, and a successful withdraw.
    run user_withdraw_run = 
        init.then(userStakeAction(500))
            .then(userStakeAction(100))
            .then(TransferFundsToDelegateAction)
            .then(tickAction)  
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(userUnstakeAction(200))
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_UNDELEGATE),
                state' = state
            })
            .then(offchain_component) //here, the batch is added to unbondings
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(advanceTimeAction(UNBONDING_PERIOD)) //here, the undelegated amount is added to puppeteer   
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction) // here, the witdraw pump gets the balance
            .then(receiveFromPumpAction)
            .then(userWithdrawAction({ batch: 0, expected_amount: 200, id: 0, received_amount: -1 }))
            .then(
                all{
                    assert(
                        state.envState.consumedNfts == 
                        Set({ batch: 0, expected_amount: 200, id: 0, received_amount: 200 })
                        ),
                    state' = state
            })

// This run encodes a user's stake, a successful unstake, and a successful withdraw.
    run slashing_run = 
        init.then(userStakeAction(500))
            .then(userStakeAction(100))
            .then(TransferFundsToDelegateAction)
            .then(tickAction)  
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(userUnstakeAction(200))
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_UNDELEGATE),
                state' = state
            })
            .then(offchain_component) //here, the batch is added to unbondings
            .then(slashingAction)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })
            .then(advanceTimeAction(UNBONDING_PERIOD)) //here, the undelegated amount is added to puppeteer   
            .then(tickAction) // here, drop recognizes there was slashing and marks it
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })
            .then(offchain_component)// here, the witdraw pump gets the balance
            .then(tickAction) 
            .then(receiveFromPumpAction)
            .then(userWithdrawAction({ batch: 0, expected_amount: 200, id: 0, received_amount: -1 }))
            .then(
                all{
                    assert(
                        state.envState.consumedNfts == 
                        Set({ batch: 0, expected_amount: 200, id: 0, received_amount: 196})
                        ),
                    state' = state
            }) 

    // the run shows how to reach a case in which the exchange rate turns negative
    // It consists of the following steps:
    // 1) a user stakes 400 tokens
    // 2) the funds are successfully delegated
    // 3) the slashing happens
    // 3) the user unstakes 399 tokens 
    // 4) the undelegation request fails
    //  Now, upon the next tick, the exchange rate will be calculated taking into account the the reduced value of 
    // delegated, but subtracting from it the (unreduced) amount of the failed undelegation request.
    run negative_exchange_rate_run = 
        init.then(userStakeAction(400))
            .then(TransferFundsToDelegateAction)
            .then(all{
                assert(state.dropState.delegatorContractICABalance == 400),
                state'=state
            })
            .then(tickAction)            
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS),
                state' = state
            })
            .then(offchain_component)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            })                        
            .then(tickAction)
            .then(offchain_component)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW),
                state' = state
            })                        
            .then(tickAction) 
            
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS),
                state' = state
            })
            .then(offchain_component)
            .then(userUnstakeAction(399))
            .then(tickAction) // now we are in the waiting undelegate state
            .then(executeICAAction(false, 20))
            .then(receiveICAResponseAction(20))
            .then(remoteQueryAction)
            .then(slashingAction)
            .then(slashingAction)
            .then(tickAction)
            .then(all{
                assert(state.dropState.fsmState == FSM_STATES.IDLE),
                state' = state
            
            })
            .then(tickAction)
            .then(all{
                assert(le(state.dropState.cachedTokenToLdExchangeRate, dec(0))),
                state' = state
            })

    }

    