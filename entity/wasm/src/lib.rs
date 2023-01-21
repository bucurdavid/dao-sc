// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           55
// Async Callback:                       1
// Total number of exported functions:  57

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    entity
    (
        changeVoteTokenLock
        registerDns
        getVersion
        getTrustedHostAddress
        getGovTokenId
        getGuardedVoteTokens
        isLockingVoteTokens
        getProposalIdCounter
        getProposalNftVotes
        getWithdrawableProposalIds
        getWithdrawableVotes
        getProposalAddressVotes
        getQuorum
        getMinVoteWeight
        getMinProposeWeight
        getVotingPeriodMinutes
        createRole
        removeRole
        assignRole
        unassignRole
        createPermission
        createPolicyWeighted
        createPolicyForOne
        createPolicyForAll
        createPolicyQuorum
        getUserRoles
        getPermissions
        getPolicies
        getRoles
        getRoleMemberAmount
        hasUserPlugVoted
        getPlugScAddress
        initGovToken
        changeGovToken
        changeQuorum
        changeMinVoteWeight
        changeMinProposeWeight
        changeVotingPeriodMinutes
        setPlug
        propose
        voteFor
        voteAgainst
        sign
        execute
        withdraw
        issueGovToken
        setGovTokenLocalRoles
        mint
        burn
        getProposal
        getProposalStatus
        getProposalVotes
        getProposalSigners
        getProposalSignatureRoleCounts
        getProposalPollResults
        callBack
    )
}
