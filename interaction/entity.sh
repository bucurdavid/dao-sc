NETWORK_NAME="devnet" # devnet, testnet, mainnet
DEPLOYER="./deployer.pem"
ADDRESS="erd1qqqqqqqqqqqqqpgq90nl5ta7nwcqv5uqf33xsdh67et544xeyt2s0u8sp3"
TOKEN_ID="ONE-8602e1"

PROXY=$(erdpy data load --partition ${NETWORK_NAME} --key=proxy)
CHAIN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=chain-id)
COST_TOKEN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-token-id)

# params:
#   $1 = title
#   $2 = description
#   $3 = token amount
propose() {
    erdpy contract call ${ADDRESS} \
        --function="ESDTTransfer" \
        --arguments "str:$TOKEN_ID" $3 "str:propose" "str:$1" "str:$2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --pem=${DEPLOYER} \
        --send || return
}

getGovQuorum() {
    erdpy contract query ${ADDRESS} \
        --function="getQuorum" \
        --proxy=${PROXY} || return
}

# params:
#   $1 = proposal id
getProposalTitle() {
    erdpy contract query ${ADDRESS} \
        --function="getProposalTitle" \
        --arguments $1 \
        --proxy=${PROXY} || return
}

# params:
#   $1 = proposal id
getProposalDescription() {
    erdpy contract query ${ADDRESS} \
        --function="getProposalDescription" \
        --arguments $1 \
        --proxy=${PROXY} || return
}

# params:
#   $1 = proposal id
getProposalUpvotes() {
    erdpy contract query ${ADDRESS} \
        --function="getTotalUpvotes" \
        --arguments $1 \
        --proxy=${PROXY} || return
}
