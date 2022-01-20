////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    entity
    (
        cancel
        changeLockTimeAfterVotingEndsInBlocks
        changeMaxActionsPerProposal
        changeMinTokenBalanceForProposing
        changeQuorum
        changeVotingDelayInBlocks
        changeVotingPeriodInBlocks
        depositTokensForAction
        enableFeatures
        execute
        getGovernanceTokenId
        getLockTimeAfterVotingEndsInBlocks
        getMaxActionsPerProposal
        getMinTokenBalanceForProposing
        getProposalActions
        getProposalDescription
        getProposalStatus
        getProposer
        getQuorum
        getTotalDownvotes
        getTotalVotes
        getVotingDelayInBlocks
        getVotingPeriodInBlocks
        propose
        queue
        setFeatureFlag
        voteAgainst
        voteFor
        withdrawGovernanceTokens
    )
}

elrond_wasm_node::wasm_empty_callback! {}
