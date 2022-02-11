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

COST_TOKEN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-token-id)
COST_TOKEN_ID_HEX="0x$(echo -n ${COST_TOKEN_ID} | xxd -p -u | tr -d '\n')"
COST_ENTITY_CREATION_AMOUNT=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-entity-creation-amount)

deploy() {
    echo "accidental deploy protection is activated."
    exit 1;

    echo "building ENTITY contract for deployment ..."
    erdpy --verbose contract build entity || return

    echo "building MANAGER contract for deployment ..."
    erdpy --verbose contract build manager || return

    echo "running ENTITY tests ..."
    erdpy --verbose contract test entity || return

    echo "running MANAGER tests ..."
    erdpy --verbose contract test manager || return

    echo "deploying ENTITY to ${NETWORK_NAME} ..."
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

    echo "deploying MANAGER to ${NETWORK_NAME} ..."
    erdpy --verbose contract deploy --project manager \
        --arguments $entity_template_address_hex ${COST_TOKEN_ID_HEX} ${COST_ENTITY_CREATION_AMOUNT} \
        --recall-nonce --gas-limit=80000000 \
        --pem=${DEPLOYER} --outfile="deploy-${NETWORK_NAME}-manager.interaction.json" \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    MANAGER_ADDRESS=$(erdpy data parse --file="deploy-${NETWORK_NAME}-manager.interaction.json" --expression="data['emitted_tx']['address']")
    MANAGER_TRANSACTION=$(erdpy data parse --file="deploy-${NETWORK_NAME}-manager.interaction.json" --expression="data['emitted_tx']['hash']")

    erdpy data store --partition ${NETWORK_NAME} --key=manager--address --value=${MANAGER_ADDRESS}
    erdpy data store --partition ${NETWORK_NAME} --key=manager--deploy-transaction --value=${MANAGER_TRANSACTION}

    sleep 10

    setCostTokenBurnRole()

    echo ""
    echo "deployed ENTITY smart contract address: ${ENTITY_ADDRESS}"
    echo "deployed MANAGER smart contract address: ${MANAGER_ADDRESS}"
}

upgrade() {
    entity_template_address_hex="0x$(erdpy wallet bech32 --decode ${ENTITY_ADDRESS})"

    echo "building contract for upgrade ..."
    erdpy --verbose contract build manager || return

    echo "running MANAGER tests ..."
    erdpy --verbose contract test manager || return

    echo "upgrading MANAGER contract ${MANAGER_ADDRESS} on ${NETWORK_NAME} ..."
    erdpy --verbose contract upgrade ${MANAGER_ADDRESS} --project manager \
        --arguments $entity_template_address_hex ${COST_TOKEN_ID_HEX} ${COST_ENTITY_CREATION_AMOUNT} \
        --recall-nonce --gas-limit=80000000 \
        --pem=${DEPLOYER} --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    echo ""
    echo "upgraded MANAGER smart contract"
}

upgradeEntityTemplate() {
    echo "building contract for upgrade ..."
    erdpy --verbose contract build entity || return

    echo "running ENTITY tests ..."
    erdpy --verbose contract test entity || return

    echo "upgrading ENTITY template contract ${ENTITY_ADDRESS} on ${NETWORK_NAME} ..."
    erdpy --verbose contract upgrade ${ENTITY_ADDRESS} --project entity \
        --recall-nonce --gas-limit=150000000 \
        --pem=${DEPLOYER} --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return

    echo ""
    echo "upgraded ENTITY template smart contract"
}

setCostTokenBurnRole() {
    echo "adding ESDTLocalBurn role for ${MANAGER_ADDRESS} ..."

    manager_template_address_hex="0x$(erdpy wallet bech32 --decode ${MANAGER_ADDRESS})"
    burn_role_hex="0x$(echo -n 'ESDTRoleLocalBurn' | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call erd1qqqqqqqqqqqqqqqpqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzllls8a5w6u \
        --function=setSpecialRole \
        --arguments ${COST_TOKEN_ID_HEX} $manager_template_address_hex $burn_role_hex  \
        --recall-nonce \
        --pem=${DEPLOYER} \
        --gas-limit=60000000 \
        --proxy=${PROXY} \
        --chain=${CHAIN_ID} \
        --send || return

    echo ""
    echo "local burn role added!"
}

createEntityToken() {
    token_name="0x$(echo -n 'test' | xxd -p -u | tr -d '\n')"
    token_ticker="0x$(echo -n 'TEST' | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${MANAGER_ADDRESS} \
        --recall-nonce \
        --pem=${DEPLOYER} \
        --gas-limit=100000000 \
        --function="createEntityToken" \
        --arguments $token_name $token_ticker 0 \
        --value=50000000000000000 \
        --proxy=${PROXY} \
        --chain=${CHAIN_ID} \
        --send || return
}

# params:
#   $1 = token id
#   $2 = feature1
#   $3 = feature2
createEntity() {
    function="0x$(echo -n 'createEntity' | xxd -p -u | tr -d '\n')"
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    feature1="0x$(echo -n $2 | xxd -p -u | tr -d '\n')"
    feature2="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"

    erdpy contract call ${MANAGER_ADDRESS} \
        --function="ESDTTransfer" \
        --arguments ${COST_TOKEN_ID_HEX} ${COST_ENTITY_CREATION_AMOUNT} $function $token_id $feature1 $feature2 \
        --pem=${DEPLOYER} \
        --recall-nonce --gas-limit=80000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return
}

# params:
#   $1 = token id
finalizeEntity() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy contract call ${MANAGER_ADDRESS} \
        --function="finalizeEntity" \
        --arguments $token_id \
        --pem=${DEPLOYER} \
        --recall-nonce --gas-limit=80000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --send || return
}

# params:
#   $1 = token id
getEntityAddress() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy contract query ${MANAGER_ADDRESS} \
        --function="getEntityAddress" \
        --arguments $token_id \
        --proxy=${PROXY} || return
}
