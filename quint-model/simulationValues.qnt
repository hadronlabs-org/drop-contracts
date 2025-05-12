module simulationValues{
    import constants.* from "./constants"
    import types.* from "./types"
    import Dec.* from "./dec"

    // no clue about the correct value for this at the moment
    val UNBONDING_SAFE_BUFFER = 10

    // no clue about the correct value for this at the moment
    val DELEGATION_THRESHOLD = 0

    // no clue about the correct value for this at the moment
    val IDLE_CALL_INTERVAL = 10

    pure val TIME_PER_BLOCK_REMOTE = 6
    pure val TIME_PER_BLOCK_LOCAL = 6

    pure val UNBONDING_THRESHOLD = 3 * 24 * 60 * 60

    pure val TIME_PASSED_ICA_RESPONSE_OPTIONS = Set(3*TIME_PER_BLOCK_LOCAL, 5 * TIME_PER_BLOCK_LOCAL, 50*TIME_PER_BLOCK_LOCAL) 
    pure val TIME_PASSED_OPTINS = (1*TIME_PER_BLOCK_LOCAL).to(100*TIME_PER_BLOCK_LOCAL)

    pure val UNBONDING_PERIOD = 21 * 24 * 60 * 60

    pure val USER_BALANCES = Map("Alice" -> 10000, "Bob"->100, "Charlie"->20000)

    pure val AMOUNTS_TO_STAKE = 90.to(10000)
    pure val AMOUNTS_TO_UNSTAKE = 90.to(10000)

    pure val EMPTY_BATCH: UnbondingBatch =  {id: -1, amount: 0, unbonding_time: 0, slashing_happened: false}
    pure val SLASHING_RATIO: Dec = (1, 50)
    pure val FIRST_BATCH: UnbondingBatch =  {id: 0, amount: 0, unbonding_time: -1, slashing_happened: false}

    pure val EMPTY_ICA_RESPONSE: ICAResponse = {success: false, req: ICA_None}

    pure val NO_REVERTS: bool = true

    pure val REWARDS_FIXED_AMOUNT = 1
    
    pure val INIT_STATE = {
      dropState: DROP_STATE_INIT_STATE,
      envState: ENV_STATE_INIT_STATE
    }
    
    pure val DROP_STATE_INIT_STATE = {
      // users state
      nftBalances: Set(),

      // drop state      
      puppeteerContractICABalance: 0,
      withdrawPumpBalance: 0,
      pendingToUnstake: 0,      
      withdraw_manager_balance: 0,
      fsmState: FSM_STATES.IDLE,

      totalLdInCirculation: 0,
      lastIdleCall: -IDLE_CALL_INTERVAL,
      delegatorContractBalance: 0,
      delegatorContractICABalance: 0,      

      // drop state: unbonding flow
      lastTimeUnbonding: -UNBONDING_THRESHOLD,
      lastBatch: FIRST_BATCH,
      failedBatch: EMPTY_BATCH,
      withdrawnBatches: Map(),

      // counters
      nextNftId: 0,
      nextUnbondingBatchId: 1,
      nextOffChainRequestId: 0,

      // exchange rate: how many TOKENs per one LD
      cachedTokenToLdExchangeRate: dec(1),

      // batches in the process of unbonding
      unbondingBatches: Set(),
      
    }
    pure val ENV_STATE_INIT_STATE = {
      // the set of requests that we are waiting the response to
      ICAResponses: List(),
      ICARequests: List(),
      ICAExecuted: List(),
      lastQueryRemoteHeight: -1,
      lastICARemoteHeight: -1,
      offChainRequests: Set(),

      // environment state
      // TODO: a question for us is whether it makes sense to 
      // model time in terms of (local?) blocks
      blockInfo: {height: 0, block_time: 0},

      // remote chain state
      delegated: 0,    
      remoteChainHeight: 0, 
      unbondings: Set(),

      consumedNfts: Set(),

    }

}