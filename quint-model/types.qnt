module types {
    import Dec.* from "./dec"
    
    type User = str

    type Nft = {
      id: int,
      batch: int,
      expected_amount: int,
      received_amount: int
    }

    type UnbondingBatch = {
      id: int,
      amount: int,
      unbonding_time: int,
      slashing_happened: bool
    }

    type ICARequest = 
      | ICA_Undelegate({requestedBatch: UnbondingBatch})
      | ICA_ClaimAndWithdraw({withdraw: bool, claim: bool, unbondedBatches: Set[UnbondingBatch]})      
      | ICA_Delegate
      | ICA_DelegateRewardsSendFees  
      | ICA_None    
      
    type ICAResponse = {success: bool, req: ICARequest}

     type OffChainRequest = {
      id: int, //just for uniqueness
      name: str,
      amount: int
    }

    

    type BlockHeight = int

    type BlockInfo = {
      height: BlockHeight,
      block_time: int
    }

    type Validator = str
    
    type DropState = {

      // users state
      nftBalances: Set[Nft],

      // drop state      
      puppeteerContractICABalance: int,
      withdrawPumpBalance: int,
      pendingToUnstake: int,      
      withdraw_manager_balance: int,
      fsmState: str,

      totalLdInCirculation: int,
      lastIdleCall: int,
      delegatorContractBalance: int,
      delegatorContractICABalance: int,                  

      // drop state: unbonding flow
      lastTimeUnbonding: int,
      lastBatch: UnbondingBatch,
      failedBatch: UnbondingBatch,
      withdrawnBatches: int -> UnbondingBatch,

      // counters
      nextNftId: int,
      nextUnbondingBatchId: int,
      nextOffChainRequestId: int,

      // exchange rate: how many TOKENs per LDs
      cachedTokenToLdExchangeRate: Dec,

      // batches in the process of unbonding
      unbondingBatches: Set[UnbondingBatch],
      
    }

    type EnvironmentState = {
      // IBC      
      // the queue of incoming ICA responses
      ICAResponses: List[ICAResponse],
      // the queue of outgoing ICA requests
      ICARequests: List[ICARequest],
      ICAExecuted: List[ICAResponse],
      lastQueryRemoteHeight: BlockHeight,
      lastICARemoteHeight: BlockHeight,
      offChainRequests: Set[OffChainRequest],

      // TODO: a question for us is whether it makes sense to 
      // model time in terms of (local?) blocks
      blockInfo: BlockInfo,
      
      // remote chain state
      delegated: int,
      remoteChainHeight: BlockHeight,
      unbondings: Set[UnbondingBatch],


      //consumed nft balances
      consumedNfts: Set[Nft]
    }

    type State = {
      dropState: DropState,
      envState: EnvironmentState
    }

    type Result = {
      state: State,
      error: str,
      description: str,
    }
  }