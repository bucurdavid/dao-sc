// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           23
// Async Callback (empty):               1
// Total number of exported functions:  25

#![no_std]
#![feature(alloc_error_handler, lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    manager
    (
        executeTicket
        createEntity
        upgradeEntity
        setFeatures
        setEntityCreationCost
        setDailyBaseCost
        setDailyFeatureCost
        getEntities
        getEntityTemplateAddress
        getTrustedHostAddress
        getCostTokenId
        getEntityCreationCost
        getBaseDailyCost
        getFeatureDailyCost
        getFeatures
        initCreditsModule
        boost
        boostWithSwap
        registerExternalBoost
        getCredits
        initDexModule
        initOrgModule
        forwardCostTokensToOrg
    )
}

multiversx_sc_wasm_adapter::empty_callback! {}
