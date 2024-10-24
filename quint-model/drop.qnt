// -*- mode: Bluespec; -*-
// ****************************************************************************
//  This encodes Neutron's Drop protocol
//
//  Simplifications:
//    - Single remote chain
//    - No rewards in non-native
//    - No LSM shares. Therefore, we also do not update the exchange rate each time there is an
//      idle tick, but only if there is an idle tick after sufficient time has passed.
//    - Interchain Queries (ICQs) for different values are treated the same (as if a single query
//      returns the data for both the balance and delegations).
//
//
// ****************************************************************************

module drop {

  import basicSpells.* from "./basicSpells"
  import extraSpells.* from "./extraSpells"
  import Dec.* from "./dec"
  import constants.* from "./constants"
  import simulationValues.* from "./simulationValues"
  // ****************************************************************************
  // Data Types
  // *************************************************************************
  import types.* from "./types"

  // **************************************************************************
  // Auxiliary functions
  // ************************************************************************* 
  import helpers.* from "./helpers"
  



  // **************************************************************************
  // Main functions
  // ************************************************************************* 

  // User operations

  pure def stake(state: State, amount: int): Result = {
    val dropState = state.dropState
    val updatedDropState = dropState.with("delegatorContractBalance", dropState.delegatorContractBalance + amount)
                            .with("totalLdInCirculation", dropState.totalLdInCirculation + token_to_ld(state, amount))
    {state: {dropState: updatedDropState, ...state}, error: ERRORS.NIL, description: "staked"}
  }


  pure def unstake(state: State, amount: int): Result = { 
    if(state.dropState.totalLdInCirculation >= amount){
      val dropState = state.dropState
      val nft = {id: dropState.nextNftId, batch: dropState.lastBatch.id, expected_amount: ld_to_token(state, amount), received_amount: -1}
      val updatedDropState = dropState.with("lastBatch", dropState.lastBatch.with("amount", dropState.lastBatch.amount + ld_to_token(state, amount)))
                              .with("totalLdInCirculation", dropState.totalLdInCirculation - amount)
                              .with("nftBalances", dropState.nftBalances.setAdd(nft))
                              .with("nextNftId", dropState.nextNftId + 1)
      {state: {dropState: updatedDropState, ...state}, error: ERRORS.NIL, description: "unstaked"}
    }else { state: state, error: ERRORS.REVERT, description: "not enough ld tokens"}
     
  }


  pure def withdraw(state: State, nft: Nft): Result = {
    val dropState = state.dropState
    val envState = state.envState
    if(not(dropState.withdrawnBatches.keys().contains(nft.batch))){
      {state: state, description: "batch not withdrawn yet", error: ERRORS.REVERT}
    } else {
      val batch = dropState.withdrawnBatches.get(nft.batch) 
      val amountToWithdraw = batch.amount

      if (dropState.withdraw_manager_balance >= amountToWithdraw) {
        val updatedNftBalances = dropState.nftBalances.filter(n => n.id != nft.id)
          
        val updatedConsumedNftBalances =
          envState.consumedNfts.setAdd({received_amount: slashedValue(nft.expected_amount, SLASHING_RATIO), ...nft})
        val updatedDropState = dropState
                                .with("withdraw_manager_balance", dropState.withdraw_manager_balance - amountToWithdraw)
                                .with("nftBalances", updatedNftBalances)
        val updatedEnvState = envState.with("consumedNfts", updatedConsumedNftBalances)
              {state: {dropState: updatedDropState, envState: updatedEnvState}, error: ERRORS.NIL, description: "withdrawn"} 
      } else {state: state, description: "not enough funds", error: ERRORS.REVERT}
    }
  }


  // FSM modeling

  

  // TICKSs
  // Each tick is a state transformation upon receiving a tick. 
  // They are prefixed with the state they are in 
  // (and deciding which tick to take is done at the level of actions)

  pure def idleTick(state: State): Result =
    val dropState = state.dropState
    val envState = state.envState

    if (
      not(batchUnbondedSoon(state)) 
      and freshICQ(state)
      // this part is modeled here because LSM shares are not modeled. When adding them,
      // it should be another try... block 
      and envState.blockInfo.block_time - dropState.lastIdleCall > IDLE_CALL_INTERVAL
      ) {
      if (not(withdrawableBatches(state)) and not(somethingToClaim(state))) {
        tryDelegateStakedFunds(state)
      } else {

        val unbondedBatches = dropState.unbondingBatches.filter(b => isWithdrawable(envState.blockInfo.block_time, b))
        
        val unbondedBatchesKeys = unbondedBatches.map(b => b.id)
        
        val expected_amount = unbondedBatches.fold(0, (acc, b) => acc + b.amount)
        val available_amount = dropState.puppeteerContractICABalance
        val updatedBatches = 
          // have to mark batches as slashed
          if (expected_amount > available_amount) 
            dropState.unbondingBatches.map(
            b => 
              if (unbondedBatchesKeys.contains(b.id)) 
                {
                  slashing_happened: true,
                  amount: slashedValue(b.amount, SLASHING_RATIO),
                  ... b
                }
              else 
                b
            )
          else
            dropState.unbondingBatches  

        val msg = ICA_ClaimAndWithdraw({
          unbondedBatches: updatedBatches, 
          withdraw: withdrawableBatches(state),
          claim: somethingToClaim(state)          
          })
    
        
          
          
        val updatedState = sendICAMessage(state, msg)

        val updatedDropState = 
          updatedState.dropState
            .with("fsmState", FSM_STATES.WAITING_CLAIM_WITHDRAW)
            .with("cachedTokenToLdExchangeRate", recalculateExchangeRate(state))
            .with("lastIdleCall", envState.blockInfo.block_time)
            .with("unbondingBatches", updatedBatches) 
                              
        
        {state: {dropState: updatedDropState, ...updatedState}, error: ERRORS.NIL, description: "requested claim/withdraw"}
      }
    } else {state: state, error: ERRORS.REVERT, description: "idle tick conditions not met"}   

  pure def waitingClaimWithdrawTick(state: State): Result = {
    val dropState = state.dropState
    val envState = state.envState
    if (length(envState.ICAResponses) > 0){
      val ica_response = head(envState.ICAResponses)
      
      match ica_response.req {
        | ICA_ClaimAndWithdraw(args) =>
          {
          val expected_unbonded_amount = args.unbondedBatches.fold(0, (acc, b) => acc + b.amount)
          val stateReadResponse = {
            envState: envState.with("ICAResponses", tail(envState.ICAResponses)),
            ...state
          }
          // in case there were withdrawals
          if (ica_response.success and args.withdraw) {
            val numUnbondedBatches = size(args.unbondedBatches)
            if (numUnbondedBatches == 0) {state: stateReadResponse, error: ERRORS.UNEXPECTED, description: "tried withdraw but no unbonded batches"} //this should never happen
            else if (numUnbondedBatches > 1 and expected_unbonded_amount > dropState.withdrawPumpBalance) 
              {state: stateReadResponse, description: "emergency withdrawn", error: ERRORS.UNEXPECTED}
            else {
              val amountToRequest = min(expected_unbonded_amount, dropState.withdrawPumpBalance)              
              val req = {id: dropState.nextOffChainRequestId, name: "receive_withdrawn_funds", amount: amountToRequest}
              val updatedWithdrawnBatches = args.unbondedBatches.fold(
                                                  dropState.withdrawnBatches, 
                                                  (acc, b) => acc.put(b.id, b)
                                                  )
              val updatedState = 
                stateReadResponse
                  .with(
                    "dropState",
                    {
                      withdrawnBatches: updatedWithdrawnBatches,
                      unbondingBatches: dropState.unbondingBatches.exclude(args.unbondedBatches),
                      nextOffChainRequestId: dropState.nextOffChainRequestId + 1,                      
                      ...stateReadResponse.dropState
                    })
                  .with(
                    "envState",
                    {
                      offChainRequests: envState.offChainRequests.setAdd(req),
                      ...stateReadResponse.envState
                      
                    })
                                             
              tryDelegateStakedFunds(updatedState)
            }
          } 
          else if (ica_response.success and not(args.withdraw)) 
            tryDelegateStakedFunds(stateReadResponse)        
        
          // in case of failure
          else tryDelegateStakedFunds(stateReadResponse)  


        }
        | _ => {state: state, error: ERRORS.UNEXPECTED, description: "unexpected ICA response"} 
      }   
      
    } else {state: state, error: ERRORS.REVERT, description: "waiting claim/withdraw tick: no ICAs"}

  }


  pure def waitingDelegateStakedFundsTick(state: State): Result = {
    val dropState = state.dropState
    val envState = state.envState
    if (freshICQ(state))
      if (length(envState.ICAResponses) > 0){
        val ica_response = head(envState.ICAResponses)
        match ica_response.req {
          | ICA_Delegate => 
            val icaResponseReadState = {envState: envState.with("ICAResponses", tail(envState.ICAResponses)), ... state}      
            tryDelegateRewards(icaResponseReadState)    
          
          | _ => {state: state, error: ERRORS.UNEXPECTED, description: "unexpected ICA response"}
        }      
      } else {state: state, error: ERRORS.REVERT, description: "waiting delegate staked funds tick: no ICAs"}
    else {state: state, error: ERRORS.REVERT, description: "waiting delegate staked funds tick: no fresh ICQ"}
  }

  pure def waitingDelegateRewardsTick(state: State): Result = {

    if (freshICQ(state))
      val dropState = state.dropState
      val envState = state.envState

      if (length(envState.ICAResponses) > 0){
        val ica_response = head(envState.ICAResponses)
        match ica_response.req {
          | ICA_DelegateRewardsSendFees =>
            // there used to be a difference based on the msg = success | failure, but not anymore
            tryUndelegate(
              {envState: envState.with("ICAResponses", tail(envState.ICAResponses)), ... state}      
              )

          | _ => {state: state, error: ERRORS.UNEXPECTED, description: "unexpected ICA response"}
          }
      } else {state: state, error: ERRORS.REVERT, description: "waiting delegate rewards tick: no ICAs"}
    else {state: state, error: ERRORS.REVERT, description: "waiting delegate rewards tick: no fresh ICQ"}
  }

  pure def waitingUndelegateTick(state: State): Result = {
    val envState = state.envState
    val dropState = state.dropState

    if (length(envState.ICAResponses) > 0){
      val ica_response = head(envState.ICAResponses)
      val stateReadResponse = {envState: envState.with("ICAResponses", tail(envState.ICAResponses)), ... state}      
      match ica_response.req {
        | ICA_Undelegate(args) => {
          if (ica_response.success) 
            val updatedBatch = args.requestedBatch.with(
              "unbonding_time", 
              state.envState.blockInfo.block_time + UNBONDING_PERIOD
              )
            val updatedState = stateReadResponse
              .with("dropState", 
                {
                  fsmState: FSM_STATES.IDLE,
                  failedBatch: {id: -1, amount: 0, unbonding_time: -1, slashing_happened: false},
                  unbondingBatches: dropState.unbondingBatches.setAdd(updatedBatch),
                  lastTimeUnbonding: stateReadResponse.envState.blockInfo.block_time,
                  ...dropState
                })

            {
              state: updatedState,
              error: ERRORS.NIL,
              description: "successful unbonding"
            }
          else 
            val updatedState = stateReadResponse
              .with("dropState", 
                {
                  fsmState: FSM_STATES.IDLE,
                  failedBatch: args.requestedBatch,                  
                  ...dropState
                })

            {
              state: updatedState,
              error: ERRORS.IBC,
              description: "failed unbonding"
            }
                       
        }
      | _ => {state: stateReadResponse, error: ERRORS.UNEXPECTED, description: "unexpected ICA response"}
      }
    } else {state: state, error: ERRORS.REVERT, description: "waiting undelegate tick: no ICAs"}
  }


  // PATHS
  // Paths are called by different tick functions.
  // Each try... block represents one path in a diagram, leading from one FSM state to another.
  // Depending on the conditions, the FSM can branch to different paths.

  pure def tryDelegateStakedFunds(state: State): Result = {
    if (pendingOnDelegatorICA(state)) {
      val msg = ICA_Delegate
      val updatedState = sendICAMessage(state, msg)
      {state: { dropState: updatedState.dropState.with("fsmState", FSM_STATES.WAITING_DELEGATE_USER_FUNDS), ...updatedState}, error: ERRORS.NIL, description: "requested delegation of staked funds"}
    } else tryDelegateRewards(state)
  }

  pure def tryDelegateRewards(state: State): Result = {    
    if (pendingOnMainICA(state)) {
      val msg = ICA_DelegateRewardsSendFees
      val updatedState = sendICAMessage(state, msg)
      {state: { dropState: updatedState.dropState.with("fsmState", FSM_STATES.WAITING_DELEGATE_REWARDS), ...updatedState}, error: ERRORS.NIL, description: "requested delegation of rewards"}
    } else tryUndelegate(state)
    
  }

  pure def tryUndelegate(state: State): Result = {
    val dropState = state.dropState
    val envState = state.envState
    
    
    if (not(isThereFailedUnbondingBatch(state)) and not(pendingToUndelegate(state))) {
            {state: {dropState: dropState.with("fsmState", FSM_STATES.IDLE), ...state}, error: ERRORS.NIL, description: "nothing to unbond, back to idle"}
    } else {

      val batch =
        if (isThereFailedUnbondingBatch(state))
          dropState.failedBatch
        else
          dropState.lastBatch
      
      val msg = ICA_Undelegate({requestedBatch: batch})

      val updatedDropState = 
        if (isThereFailedUnbondingBatch(state)) 
          dropState
        else 
          {
            lastBatch: {id: dropState.nextUnbondingBatchId, amount: 0, unbonding_time: -1, slashing_happened: false},
            nextUnbondingBatchId: dropState.nextUnbondingBatchId + 1,
            ... dropState
          }
      val updatedStatePrime = sendICAMessage({dropState: updatedDropState, ...state}, msg)
      {
        state: {dropState: updatedStatePrime.dropState.with("fsmState", FSM_STATES.WAITING_UNDELEGATE), ...updatedStatePrime}, 
        error: ERRORS.NIL, 
        description: "requested unbonding"
      }                

    }      
  }

  // OFFCHAIN FUNCTIONS

  pure def receiveWithdrawnFunds(state: State, req: OffChainRequest): Result = {
    val dropState = state.dropState
    val envState = state.envState
    {
      state: {
        dropState: {
          withdraw_manager_balance: dropState.withdraw_manager_balance + dropState.withdrawPumpBalance,
          withdrawPumpBalance: 0,
          ...dropState
        },                
        envState: envState.with("offChainRequests", envState.offChainRequests.setRemove(req))
      },
      error: ERRORS.NIL,
      description: "received withdrawn funds"
    }
  }

  pure def transferFundsToDelegate(state: State): Result = {
    val dropState = state.dropState
    if (dropState.delegatorContractBalance > 0) {            

      {
        state: {
          dropState: {
            delegatorContractICABalance: dropState.delegatorContractICABalance + dropState.delegatorContractBalance,
            delegatorContractBalance: 0,
            ...dropState
          },
          ... state        
        }, 
        error: ERRORS.NIL, 
        description: "funds moved to the ICA delegator contract"}
    } else {state: state, error: ERRORS.REVERT, description: "no funds to transfer"}
  }

  // ENVIRONMENT FUNCTIONS

  // the time is passing in the discrete mode - in block progression. 
  // However, we keep blockInfo.block_time as a reference to time values for readability.
  // This function will also transfer any funds that successfully unbonded to the puppeteer contract.
  pure def timePassing(state: State, timePassed: int): State = {    
    val envState = state.envState 
    val dropState = state.dropState 
    val newTime = envState.blockInfo.block_time + timePassed  
    val unbonded = envState.unbondings.filter(b => isWithdrawable(newTime, b))
    val unbondedAmount = unbonded.fold(0, (acc, b) => acc + b.amount)
    { 
      envState: {
        blockInfo: {
          block_time: newTime,
          height: envState.blockInfo.height + timePassed / TIME_PER_BLOCK_LOCAL,
        },
          remoteChainHeight: envState.remoteChainHeight + timePassed / TIME_PER_BLOCK_REMOTE,
          unbondings: envState.unbondings.exclude(unbonded),
          ... envState
        },
       dropState: {
        puppeteerContractICABalance: dropState.puppeteerContractICABalance + unbondedAmount,

        ... dropState
       }
    }

  }

  pure def slashing(state: State): State =
    val activeUnbondings = state.envState.unbondings.filter(
      b => b.unbonding_time > state.envState.blockInfo.block_time
      )
    val activeUnbondingsKeys = activeUnbondings.map(b => b.id)
    val unbondingsWithSlashes = state.envState.unbondings.map(
      b => 
        if (activeUnbondingsKeys.contains(b.id))
          {
            slashing_happened: true,
            amount: slashedValue(b.amount, SLASHING_RATIO),
            ... b
          }
        else
          b
      )
    {
      envState: {
        unbondings: unbondingsWithSlashes,
        delegated: slashedValue(state.envState.delegated, SLASHING_RATIO),
        ... state.envState
      },
      ... state
    }
    

  // Execution of the ICA requests. 
  // It checks for conditions of specific actions to see if the action should fail.
  // If it does not fail by specific conditions, the function uses `default_success` parameter
  // (which will be provided by the simulator).
  pure def executeICA(state: State, default_success: bool, timePassed: int): Result = 
    val envState = state.envState    
    val dropState = state.dropState

    val messageToExecute = head(envState.ICARequests)
    val timePassedState = timePassing(state, timePassed)
    val specificSuccess = match messageToExecute {
      // if the user attempts to undelegate more than there is delegated, the transaction will fail
      | ICA_Undelegate(args) => args.requestedBatch.amount <= envState.delegated
      | _ => default_success
    }
    // and: we do not want to succeed if the specific success is false
    val success = default_success and specificSuccess
    val stateDeliveredMessage = timePassedState.with(
      "envState",
      {
        ICAExecuted: envState.ICAResponses.append({req: messageToExecute, success: success}),
        ICARequests: tail(envState.ICARequests),                
        ... timePassedState.envState
      }
    )            
      
    val stateRemoteUpdate =
      if (not(success))
        stateDeliveredMessage
      else
        match messageToExecute {
          // Undelegations: we update `delegated` and `unbondings` in the environment state
          | ICA_Undelegate(args) =>
            val updatedBatch = args.requestedBatch.with(
              "unbonding_time", 
              stateDeliveredMessage.envState.blockInfo.block_time + UNBONDING_PERIOD
            )
            stateDeliveredMessage.with(
              "envState",
              {
                delegated: stateDeliveredMessage.envState.delegated - args.requestedBatch.amount,
                unbondings: stateDeliveredMessage.envState.unbondings.setAdd(updatedBatch),
                
                ... stateDeliveredMessage.envState
              }
            )

            // Claim and Withdraw: upon execution, the funds will be moved from the puppeteerICA contract 
            // to the withdraw pump. If there are rewards, puppeteerICA gets the rewards
          | ICA_ClaimAndWithdraw(args) =>
            val unbonded = args.unbondedBatches
            val withdraw = args.withdraw
            val claim = args.claim
            val amountToWithdraw = 
              if (withdraw)
                min(
                  unbonded.fold(0, (acc, b) => acc + b.amount),
                  stateDeliveredMessage.dropState.puppeteerContractICABalance)
              else
                0
            val amountToClaim =
              if (claim)
                REWARDS_FIXED_AMOUNT
              else
                0
              
            stateDeliveredMessage.with(
              "dropState",
              {
                // get rewards and transfer the requested amount to the withdraw pump
                puppeteerContractICABalance: stateDeliveredMessage.dropState.puppeteerContractICABalance + amountToClaim - amountToWithdraw,                                
                withdrawPumpBalance: 
                  stateDeliveredMessage.dropState.withdrawPumpBalance + amountToWithdraw,
                
                ... stateDeliveredMessage.dropState
              }
            )
            
          // Delegations: we update the `delegated` in the environment state
          | ICA_Delegate =>
            
            {
              envState: {
                delegated: stateDeliveredMessage.envState.delegated + stateDeliveredMessage.dropState.delegatorContractICABalance,
                ... stateDeliveredMessage.envState
              },
              dropState: {
                delegatorContractICABalance: 0,   
                ... stateDeliveredMessage.dropState
              },              
            }
          
          | ICA_DelegateRewardsSendFees =>

            {
              envState: {
                delegated: stateDeliveredMessage.envState.delegated + stateDeliveredMessage.dropState.puppeteerContractICABalance,
                ... stateDeliveredMessage.envState
              },
              dropState: {
                puppeteerContractICABalance: 0,
                ... stateDeliveredMessage.dropState
              },              
            }
            
          | ICA_None => stateDeliveredMessage
          }
          
      
      {state: stateRemoteUpdate, error: ERRORS.NIL, description: "executed ICA request"}


  
  
  // Receiving the ICA response is updating the ICAResponses queue.
  pure def receiveICAResponse(state: State, timePassed: int): Result =  
    val envState = state.envState
    val receivedResponse = head(envState.ICAExecuted)
    val timePassedState = timePassing(state, timePassed)
    val updatedEnvState = 
       {
          ICAResponses: envState.ICAResponses.append(receivedResponse),
          ICAExecuted: tail(envState.ICAExecuted),        
          lastICARemoteHeight: timePassedState.envState.remoteChainHeight,
          ... timePassedState.envState
        } 

    {state: {envState: updatedEnvState, ...state}, error: ERRORS.NIL, description: "received ICA response"}



  
  
  



}

