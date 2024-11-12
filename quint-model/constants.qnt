module constants{
    // This module contains values for enum-like records.

    val FSM_STATES = {
        IDLE: "idle",
        WAITING_DELEGATE_USER_FUNDS: "delegate_user_funds",
        WAITING_DELEGATE_REWARDS: "delegate_rewards",
        WAITING_CLAIM_WITHDRAW: "waiting_claim_withdraw",
        WAITING_UNDELEGATE: "waiting_undelegate"
    }

    val ERRORS = {
        NIL: "nil",
        REVERT: "revert",
        IBC: "ibc",
        UNEXPECTED: "unexpected",
    }

    val BATCH_STATUS = {
        WITHDRAWING: "withdrawing"

    }

    val ICA_REQUEST_NAMES = {
        UNDELEGATE: "undelegate_request",
        DELEGATE: "delegator_delegate_request",
        DELEGATE_REWARDS_SEND_FEES: "delegate_rewards_send_fees_request",
        CLAIM_WITHDRAW: "claim_withdraw_request",
        CLAIM: "claim_request",
        WITHDRAW: "withdraw_request",
    }    

    }