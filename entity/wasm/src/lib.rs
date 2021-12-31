////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    entity
    (
        init
        cancel
        changeLockTimeAfterVotingEndsInBlocks
        changeMaxActionsPerProposal
        changeMinTokenBalanceForProposing
        changeQuorum
        changeVotingDelayInBlocks
        changeVotingPeriodInBlocks
        depositTokensForAction
        downvote
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
        initGovernanceModule
        propose
        queue
        setFeatureFlag
        upvote
        withdrawGovernanceTokens
    )
}

elrond_wasm_node::wasm_empty_callback! {}
