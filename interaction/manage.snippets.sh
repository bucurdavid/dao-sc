##### - configuration - #####
NETWORK_NAME="devnet" # devnet, testnet, mainnet
DEPLOYER="./deployer.pem" # main actor pem file
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

##### - configuration end - #####

ENTITY_ADDRESS=$(erdpy data load --partition ${NETWORK_NAME} --key=entity--address)
ENTITY_DEPLOY_TRANSACTION=$(erdpy data load --partition ${NETWORK_NAME} --key=entity--deploy-transaction)
MANAGER_ADDRESS=$(erdpy data load --partition ${NETWORK_NAME} --key=manager--address)
MANAGER_DEPLOY_TRANSACTION=$(erdpy data load --partition ${NETWORK_NAME} --key=manager--deploy-transaction)

COST_TOKEN=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-token-id)
COST_TOKEN_HEX="0x$(echo -n ${COST_TOKEN} | xxd -p -u | tr -d '\n')"
COST_ENTITY_CREATION_AMOUNT=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-entity-creation-amount)

deploy() {
    echo ">> building ENTITY contract for deployment ..."
    erdpy --verbose contract build entity || return

    echo ">> building MANAGER contract for deployment ..."
    erdpy --verbose contract build manager || return

    echo ">> deploying ENTITY to ${NETWORK_NAME} ..."
    erdpy --verbose contract deploy --project entity \
        --recall-nonce --gas-limit=80000000 \
        --pem=${DEPLOYER} --outfile="deploy-${NETWORK_NAME}-entity.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    ENTITY_ADDRESS=$(erdpy data parse --file="deploy-${NETWORK_NAME}-entity.interaction.json" --expression="data['emitted_tx']['address']")
    ENTITY_TRANSACTION=$(erdpy data parse --file="deploy-${NETWORK_NAME}-entity.interaction.json" --expression="data['emitted_tx']['hash']")

    erdpy data store --partition ${NETWORK_NAME} --key=entity--address --value=${ENTITY_ADDRESS}
    erdpy data store --partition ${NETWORK_NAME} --key=entity--deploy-transaction --value=${ENTITY_TRANSACTION}

    ENTITY_ADDRESS_HEX="0x$(erdpy wallet bech32 --decode ${ENTITY_ADDRESS})"

    sleep 10

    echo ">> deploying MANAGER to ${NETWORK_NAME} ..."
    erdpy --verbose contract deploy --project manager \
        --arguments $entity_template_address_hex ${COST_TOKEN_HEX} ${COST_ENTITY_CREATION_AMOUNT} \
        --recall-nonce --gas-limit=80000000 \
        --pem=${DEPLOYER} --outfile="deploy-${NETWORK_NAME}-manager.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    MANAGER_ADDRESS=$(erdpy data parse --file="deploy-${NETWORK_NAME}-manager.interaction.json" --expression="data['emitted_tx']['address']")
    MANAGER_TRANSACTION=$(erdpy data parse --file="deploy-${NETWORK_NAME}-manager.interaction.json" --expression="data['emitted_tx']['hash']")

    erdpy data store --partition ${NETWORK_NAME} --key=manager--address --value=${MANAGER_ADDRESS}
    erdpy data store --partition ${NETWORK_NAME} --key=manager--deploy-transaction --value=${MANAGER_TRANSACTION}

    echo ""
    echo "deployed ENTITY smart contract address: ${ENTITY_ADDRESS}"
    echo "deployed MANAGER smart contract address: ${MANAGER_ADDRESS}"
}

upgradeManager() {
    entity_template_address_hex="0x$(erdpy wallet bech32 --decode ${ENTITY_ADDRESS})"

    echo "building contract for upgrade ..."
    erdpy --verbose contract build manager || return

    echo "upgrading MANAGER contract ${MANAGER_ADDRESS} on ${NETWORK_NAME} ..."
    erdpy --verbose contract upgrade ${MANAGER_ADDRESS} --project manager \
        --arguments $entity_template_address_hex ${COST_TOKEN_HEX} ${COST_ENTITY_CREATION_AMOUNT} \
        --recall-nonce --gas-limit=80000000 \
        --pem=${DEPLOYER} --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    echo ""
    echo "upgraded MANAGER smart contract"
}

# params:
#   $1 = token amount
createEntity() {
    token_name_hex="0x$(echo -n 'test' | xxd -p -u | tr -d '\n')"
    token_ticker_hex="0x$(echo -n 'TEST' | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${MANAGER_ADDRESS} \
        --recall-nonce \
        --pem=${DEPLOYER} \
        --gas-limit=100000000 \
        --function="createEntity" \
        --arguments $token_name_hex $token_ticker_hex 0 \
        --value=50000000000000000 \
        --proxy=${PROXY} \
        --chain=${CHAIN_ID} \
        --send || return
}

# params:
#   $1 = token id
getEntityAddress() {
    token_id_hex="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy contract query ${MANAGER_ADDRESS} \
        --function="getEntityAddress" \
        --arguments $token_id_hex \
        --proxy=${PROXY} || return
}

# params:
#   $1 = token id
setup() {
    token_id_hex="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy contract call ${MANAGER_ADDRESS} \
        --function="setup" \
        --arguments $token_id_hex \
        --pem=${DEPLOYER} \
        --recall-nonce --gas-limit=50000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return
}
