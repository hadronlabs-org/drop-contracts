module dropEvolution {
    // ************************************************************************************************
    // This module defines the evolution of the drop module.
    // The evolution is defined by: 
    //     1) variables, 
    //     2) initial state, and 
    //     3) step action, which defines the possible transitions of the system.
    // ************************************************************************************************

    
    import drop.* from "./drop"
    import basicSpells.* from "./basicSpells"
    import extraSpells.* from "./extraSpells"
    import Dec.* from "./dec"    
    import constants.* from "./constants"
    import simulationValues.* from "./simulationValues"
    import types.* from "./types"

    
    // VARIABLES
    var state: State    
    
    
    // INITIAL STATE
    action init = 
        state' = INIT_STATE

    // STEP ACTION
    // step: any of the following can happen
    action step = any {        
        
        // ENVIRONMENT ACTIONS
        
        //  an ICA request is received
        nondet timePassed = oneOf(TIME_PASSED_ICA_RESPONSE_OPTIONS)
        receiveICAResponseAction(timePassed),

        // an ICA request is executed
        nondet success = oneOf(Set(true, false))
        nondet timePassed = oneOf(TIME_PASSED_ICA_RESPONSE_OPTIONS)
        executeICAAction(success, timePassed),

        // time passing
        nondet timeToPass = oneOf(TIME_PASSED_OPTINS)
        advanceTimeAction(timeToPass),

        // slashing happens
        slashingAction,

        // PERMISSIONLESS OFFCHAIN ACTIONS        
        receiveFromPumpAction,
        TransferFundsToDelegateAction,
        tickAction,
        remoteQueryAction,

        // USER ACTIONS

        // staking
        nondet amountToStake = oneOf(AMOUNTS_TO_STAKE)
        userStakeAction(amountToStake),

        // unstaking
        all{
            require(state.dropState.totalLdInCirculation > 0),
            nondet amountToUnstake = oneOf(0.to(state.dropState.totalLdInCirculation))
            userUnstakeAction(amountToUnstake)
        },
        
        // withdrawing
        all{
            require(size(state.dropState.nftBalances) > 0),
            nondet nftToWithdraw = oneOf(state.dropState.nftBalances)
            userWithdrawAction(nftToWithdraw)
        },

    }


    action TransferFundsToDelegateAction = 
        val res = transferFundsToDelegate(state)
        all {
        require(NO_REVERTS implies res.error != ERRORS.REVERT),
        state' = res.state
        }



    action receiveFromPumpAction = all {
        require(size(state.envState.offChainRequests) > 0),
        nondet req = oneOf(state.envState.offChainRequests)
        // in this action, there is no error possible --> thus no error handling
        state' = receiveWithdrawnFunds(state, req).state
    }

    
    action slashingAction = all{
        require(state.envState.delegated > 0),
        state' = slashing(state)
    }
    


    action remoteQueryAction = 
        state' = {
        envState: {
            lastQueryRemoteHeight: state.envState.remoteChainHeight,
            ...state.envState
        },
        ... state
        }

    action advanceTimeAction(timePassed: int): bool = 
        state' = timePassing(state, timePassed)

    action tickAction = 
        val res = if (state.dropState.fsmState == FSM_STATES.IDLE) idleTick(state)
                else if (state.dropState.fsmState == FSM_STATES.WAITING_CLAIM_WITHDRAW) waitingClaimWithdrawTick(state)
                else if (state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_USER_FUNDS) waitingDelegateStakedFundsTick(state)
                else if (state.dropState.fsmState == FSM_STATES.WAITING_DELEGATE_REWARDS) waitingDelegateRewardsTick(state)
                else if (state.dropState.fsmState == FSM_STATES.WAITING_UNDELEGATE) waitingUndelegateTick(state)
                else {state: state, error: ERRORS.UNEXPECTED, description: "no tick function for this state"}

        // state update
        all {
        require(NO_REVERTS implies res.error != ERRORS.REVERT),
        state' = res.state
        }


    action executeICAAction(success: bool, timePassed: int) : bool = all {
        // precondition: there is something to be executed
        require(length(state.envState.ICARequests) > 0),
        state' = executeICA(state, success, timePassed).state
    }



    action receiveICAResponseAction(timePassed: int) : bool = all {
        // precondition: there is something we are expecting an answer to
        require(length(state.envState.ICAExecuted) > 0),
        state' = receiveICAResponse(state, timePassed).state

    }

    action userStakeAction(amountToStake: int) : bool = 
        val res = stake(state, amountToStake)
        all {
        require(NO_REVERTS implies res.error != ERRORS.REVERT),
        state' = timePassing(res.state, TIME_PER_BLOCK_LOCAL)
        }


    action userUnstakeAction(amountToUnstake: int) : bool = all {
        val res = unstake(state, amountToUnstake) 
        all {
        require(NO_REVERTS implies res.error != ERRORS.REVERT),
        state' = timePassing(res.state, TIME_PER_BLOCK_LOCAL)
        
        }
    }
        


    action userWithdrawAction(nftToWithdraw: Nft) : bool =
        val res = withdraw(state, nftToWithdraw)
        all{
        require(NO_REVERTS implies res.error != ERRORS.REVERT),
        state' = timePassing(res.state, TIME_PER_BLOCK_LOCAL)
        }
        

    // BASIC INVARIANTS

    val no_two_ica_messages = all {
        length(state.envState.ICARequests) <= 1,
        length(state.envState.ICAExecuted) <= 1,
        length(state.envState.ICAResponses) <= 1
    }

    
    val total_ld_nonnegative = 
        state.dropState.totalLdInCirculation >= 0


    val delegations_nonnegative = 
        state.envState.delegated >= 0

    val batch_expected_value_nonnegative = 
        state.dropState.nftBalances.forall(
        nft => nft.expected_amount >= 0
        )


    // this invariant gets violated as an effect of slashing
    val exchange_rate_positive =
        state.dropState.cachedTokenToLdExchangeRate._1 > 0
    
    }