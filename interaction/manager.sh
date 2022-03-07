NETWORK_NAME="devnet" # devnet, testnet, mainnet
DEPLOYER="./deployer.pem"

ENTITY_ADDRESS=$(erdpy data load --partition ${NETWORK_NAME} --key=entity--address)
ENTITY_DEPLOY_TRANSACTION=$(erdpy data load --partition ${NETWORK_NAME} --key=entity--deploy-transaction)
MANAGER_ADDRESS=$(erdpy data load --partition ${NETWORK_NAME} --key=manager--address)
MANAGER_DEPLOY_TRANSACTION=$(erdpy data load --partition ${NETWORK_NAME} --key=manager--deploy-transaction)
PROXY=$(erdpy data load --partition ${NETWORK_NAME} --key=proxy)
CHAIN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=chain-id)
COST_TOKEN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-token-id)
COST_ENTITY_CREATION_AMOUNT=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-entity-creation-amount)

deploy() {
    echo "accidental deploy protection is activated."
    exit 1;

    erdpy --verbose contract build entity || return
    erdpy --verbose contract build manager || return
    erdpy --verbose contract test entity || return
    erdpy --verbose contract test manager || return

    erdpy --verbose contract deploy --project entity \
        --recall-nonce --gas-limit=200000000 \
        --outfile="deploy-${NETWORK_NAME}-entity.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --pem=${DEPLOYER} \
        --send || return

    ENTITY_ADDRESS=$(erdpy data parse --file="deploy-${NETWORK_NAME}-entity.interaction.json" --expression="data['contractAddress']")
    ENTITY_TRANSACTION=$(erdpy data parse --file="deploy-${NETWORK_NAME}-entity.interaction.json" --expression="data['emittedTransaction']['hash']")

    erdpy data store --partition ${NETWORK_NAME} --key=entity--address --value=${ENTITY_ADDRESS}
    erdpy data store --partition ${NETWORK_NAME} --key=entity--deploy-transaction --value=${ENTITY_TRANSACTION}

    sleep 6

    erdpy --verbose contract deploy --project manager \
        --arguments $ENTITY_ADDRESS "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --outfile="deploy-$NETWORK_NAME-manager.interaction.json" \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
        --send || return

    MANAGER_ADDRESS=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['contractAddress']")
    MANAGER_TRANSACTION=$(erdpy data parse --file="deploy-$NETWORK_NAME-manager.interaction.json" --expression="data['emittedTransaction']['hash']")

    erdpy data store --partition $NETWORK_NAME --key=manager--address --value=$MANAGER_ADDRESS
    erdpy data store --partition $NETWORK_NAME --key=manager--deploy-transaction --value=$MANAGER_TRANSACTION

    sleep 6
    setCostTokenBurnRole

    echo ""
    echo "deployed ENTITY TEMPLATE: $ENTITY_ADDRESS"
    echo "deployed MANAGER: $MANAGER_ADDRESS"
}

upgrade() {
    erdpy --verbose contract build manager || return
    erdpy --verbose contract test manager || return

    erdpy --verbose contract upgrade $MANAGER_ADDRESS --project manager \
        --arguments $ENTITY_ADDRESS $COST_TOKEN_ID_HEX $COST_ENTITY_CREATION_AMOUNT \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --keyfile="key.json" \
        --passfile="pass.txt" \
        --send || return
}

upgradeEntityTemplate() {
    erdpy --verbose contract build entity || return
    erdpy --verbose contract test entity || return

    erdpy --verbose contract upgrade $ENTITY_ADDRESS --project entity \
        --recall-nonce --gas-limit=250000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
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
        --pem=$DEPLOYER \
        --send || return
}

setCostTokenBurnRole() {
    erdpy --verbose contract call erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u \
        --function="setSpecialRole" \
        --arguments "str:$COST_TOKEN_ID" $MANAGER_ADDRESS "str:ESDTRoleLocalBurn"  \
        --recall-nonce --gas-limit=60000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
        --send || return
}

# params:
#   $1 = token name
#   $2 = token ticker
createEntityToken() {
    erdpy --verbose contract call $MANAGER_ADDRESS \
        --function="createEntityToken" \
        --arguments "str:$1" "str:$2" 18 \
        --value=50000000000000000 \
        --recall-nonce --gas-limit=10000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
        --send || return
}

# params:
#   $1 = token id
#   $2 = feature1
#   $3 = feature2
createEntity() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$COST_TOKEN_ID" $COST_ENTITY_CREATION_AMOUNT "str:createEntity" "str:$1" "str:$feature1" "str:$feature2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
        --send || return
}

# params:
#   $1 = token id
finalizeEntity() {
    erdpy contract call $MANAGER_ADDRESS \
        --function="finalizeEntity" \
        --arguments "str:$1" \
        --pem=$DEPLOYER \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
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
getSetupOwnerToken() {
    erdpy contract query $MANAGER_ADDRESS \
        --function="getSetupOwnerToken" \
        --arguments $1 \
        --proxy=$PROXY || return
}
