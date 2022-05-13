NETWORK_NAME="devnet" # devnet, testnet, mainnet

ENTITY_ADDRESS=$(erdpy data load --partition $NETWORK_NAME --key=entity--address)
ENTITY_DEPLOY_TRANSACTION=$(erdpy data load --partition $NETWORK_NAME --key=entity--deploy-transaction)
MANAGER_ADDRESS=$(erdpy data load --partition $NETWORK_NAME --key=manager--address)
MANAGER_DEPLOY_TRANSACTION=$(erdpy data load --partition $NETWORK_NAME --key=manager--deploy-transaction)
PROXY=$(erdpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(erdpy data load --partition $NETWORK_NAME --key=chain-id)
COST_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=cost-token-id)
COST_ENTITY_CREATION_AMOUNT=$(erdpy data load --partition $NETWORK_NAME --key=cost-entity-creation-amount)

deploy() {
    echo "accidental deploy protection is activated."
    exit 1;

    erdpy --verbose contract build entity || return
    erdpy --verbose contract build manager || return
    erdpy --verbose contract test entity || return
    erdpy --verbose contract test manager || return

    erdpy --verbose contract deploy --project entity \
        --recall-nonce --gas-limit=200000000 \
        --outfile="deploy-$NETWORK_NAME-entity.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return

    ENTITY_ADDRESS=$(erdpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['contractAddress']")
    ENTITY_TRANSACTION=$(erdpy data parse --file="deploy-$NETWORK_NAME-entity.interaction.json" --expression="data['emittedTransactionHash']")

    erdpy data store --partition $NETWORK_NAME --key=entity--address --value=$ENTITY_ADDRESS
    erdpy data store --partition $NETWORK_NAME --key=entity--deploy-transaction --value=$ENTITY_TRANSACTION

    sleep 6

    erdpy --verbose contract deploy --project manager \
        --arguments $ENTITY_ADDRESS "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --outfile="deploy-$NETWORK_NAME-manager.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return

    MANAGER_ADDRESS=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['contractAddress']")
    MANAGER_TRANSACTION=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['emittedTransactionHash']")

    erdpy data store --partition $NETWORK_NAME --key=manager--address --value=$MANAGER_ADDRESS
    erdpy data store --partition $NETWORK_NAME --key=manager--deploy-transaction --value=$MANAGER_TRANSACTION

    sleep 6
    setCostTokenBurnRole

    echo ""
    echo "deployed ENTITY TEMPLATE: $ENTITY_ADDRESS"
    echo "deployed MANAGER: $MANAGER_ADDRESS"
}

upgrade() {
    erdpy --verbose contract clean manager || return
    erdpy --verbose contract build manager || return
    erdpy --verbose contract test manager || return

    erdpy --verbose contract upgrade $MANAGER_ADDRESS --project manager \
        --arguments $ENTITY_ADDRESS "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

upgradeEntityTemplate() {
    erdpy --verbose contract clean entity || return
    erdpy --verbose contract build entity || return
    erdpy --verbose contract test entity || return

    erdpy --verbose contract upgrade $ENTITY_ADDRESS --project entity \
        --recall-nonce --gas-limit=500000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token id
upgradeEntity() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="upgradeEntity" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

setCostTokenBurnRole() {
    erdpy --verbose contract call erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u \
        --function="setSpecialRole" \
        --arguments "str:$COST_TOKEN_ID" $MANAGER_ADDRESS "str:ESDTRoleLocalBurn"  \
        --recall-nonce --gas-limit=60000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = amount
setDailyBaseCost() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="setDailyBaseCost" \
        --arguments $1 \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = feature name
#   $2 = amount
setDailyFeatureCost() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="setDailyFeatureCost" \
        --arguments "str:$1" $2 \
        --recall-nonce --gas-limit=5000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token name
#   $2 = token ticker
#   $3 = initial supply
createEntityToken() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="createEntityToken" \
        --arguments "str:$1" "str:$2" $3 \
        --value=50000000000000000 \
        --recall-nonce --gas-limit=100000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token id
#   $2 = feature1
#   $3 = feature2
createEntity() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT "str:createEntity" "str:$1" "str:$1" "str:$2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = token id
finalizeEntity() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="finalizeEntity" \
        --arguments "str:$1" \
        --recall-nonce --gas-limit=500000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = address
clearSetup() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="clearSetup" \
        --arguments $1 \
        --recall-nonce --gas-limit=500000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

# params:
#   $1 = address
#   $2 = amount
boost() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$COST_TOKEN_ID" $2 "str:boost" $1 \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --ledger \
        --send || return
}

getEntityTemplateAddress() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getEntityTemplateAddress" \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
getEntityAddress() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getEntityAddress" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}

# params:
#   $1 = address
getSetupToken() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getSetupToken" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
getBaseDailyCost() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getBaseDailyCost" \
        --proxy=$PROXY || return
}

# params:
#   $1 = token id
getCredits() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getCredits" \
        --arguments "str:$1" \
        --proxy=$PROXY || return
}
