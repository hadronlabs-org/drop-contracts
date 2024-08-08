module helpers{
    import constants.* from "./constants"
    import simulationValues.* from "./simulationValues"
    import Dec.* from "./dec"
    import types.* from "./types"

    pure def token_to_ld(state: State, token_amount: int) : int = 
    truncate(div(
      dec(token_amount),
      exchangeRate(state)
    ))

  pure def ld_to_token(state: State, ld_amount: int) : int =
    truncate(mul(
      exchangeRate(state), 
      dec(ld_amount)
      ))
    
  pure def exchangeRate(state: State) : Dec =
    state.dropState.cachedTokenToLdExchangeRate

  pure def recalculateExchangeRate(state: State) : Dec =
    val dropState = state.dropState
    val envState = state.envState
    // this is just in case: it should only ever be called from within the IDLE state
    if (dropState.fsmState != FSM_STATES.IDLE)
      dropState.cachedTokenToLdExchangeRate
    else      
      val numerator = 
        envState.delegated 
        + dropState.delegatorContractBalance 
        + dropState.delegatorContractICABalance 
        - dropState.lastBatch.amount
        - dropState.failedBatch.amount 
        
      val denominator = dropState.totalLdInCirculation

      if (denominator == 0)
        dec(1)
      else        
        (numerator, denominator)

  pure def batchUnbondedSoon(state: State): bool = 
    val dropState = state.dropState
    val envState = state.envState
    dropState.unbondingBatches.exists(
      b => and(
        // didn't happen yet
        envState.blockInfo.block_time < b.unbonding_time ,
        // and it will happen within the safe buffer
        b.unbonding_time < envState.blockInfo.block_time + UNBONDING_SAFE_BUFFER
      )
    )

  pure def freshICQ(state: State): bool = 
    state.envState.lastQueryRemoteHeight >= state.envState.lastICARemoteHeight

  // NOTE: for now we model all three "fresh...ICQ" functions as if they are the same
  // A candidate to be changed.
  pure def freshUndelegationsICQ(state: State): bool = freshICQ(state)    
  pure def freshDelegationsICQ(state: State): bool = freshICQ(state)    
  pure def freshBalanceICQ(state: State): bool = freshICQ(state)

  pure def isWithdrawable(time: int, batch: UnbondingBatch): bool =
    batch.unbonding_time <= time and batch.unbonding_time > 0

  pure def withdrawableBatches(state: State): bool = 
    state.dropState.unbondingBatches.exists(
      b => isWithdrawable(state.envState.blockInfo.block_time, b)
    )

  pure def somethingToClaim(state: State): bool = 
  // whenever there is something delegated, there is something to claim
    state.envState.delegated > 0
    

  pure def pendingOnMainICA(state: State): bool = 
    state.dropState.puppeteerContractICABalance > DELEGATION_THRESHOLD

  pure def pendingOnDelegatorICA(state: State): bool = 
    state.dropState.delegatorContractICABalance > DELEGATION_THRESHOLD


  pure def isThereFailedUnbondingBatch(state: State): bool = state.dropState.failedBatch.id != -1

  pure def pendingToUndelegate(state: State): bool =
    val dropState = state.dropState
    val envState = state.envState
    dropState.lastBatch.amount > 0 and
    (envState.blockInfo.block_time - dropState.lastTimeUnbonding) >= UNBONDING_THRESHOLD



  pure def sendICAMessage(state: State, msg: ICARequest): State = 
    {
      envState: {
        ICARequests: state.envState.ICARequests.append(msg),
        ...state.envState},
      ... state
    }

  pure def slashedValue(amount: int, ratio: Dec): int =
    amount - truncate(mul(dec(amount), ratio))
    
    
    
  }