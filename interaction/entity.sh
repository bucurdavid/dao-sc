NETWORK_NAME="devnet" # devnet, testnet, mainnet
DEPLOYER="./deployer.pem"
ADDRESS="erd1qqqqqqqqqqqqqpgqq2usalruzalugenl0pcngtlwf3pxk24t27rs8j66tv"
TOKEN_ID="ONE-8602e1"

PROXY=$(erdpy data load --partition $NETWORK_NAME --key=proxy)
CHAIN_ID=$(erdpy data load --partition $NETWORK_NAME --key=chain-id)
COST_TOKEN_ID=$(erdpy data load --partition $NETWORK_NAME --key=cost-token-id)

# params:
#   $1 = title
#   $2 = description
#   $3 = token amount
propose() {
    erdpy contract call $ADDRESS \
        --function="ESDTTransfer" \
        --arguments "str:$TOKEN_ID" $3 "str:propose" "str:$1" "str:$2" \
        --recall-nonce --gas-limit=80000000 \
        --proxy=$PROXY --chain=$CHAIN_ID \
        --pem=$DEPLOYER \
        --send || return
}

getGovQuorum() {
    erdpy contract query $ADDRESS \
        --function="getQuorum" \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposal() {
    erdpy contract query $ADDRESS \
        --function="getProposal" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
getProposalVotes() {
    erdpy contract query $ADDRESS \
        --function="getProposalVotes" \
        --arguments $1 \
        --proxy=$PROXY || return
}

# params:
#   $1 = proposal id
#   $2 = address
getProposalAddressVotes() {
    erdpy contract query $ADDRESS \
        --function="getProposalAddressVotes" \
        --arguments $1 $2 \
        --proxy=$PROXY || return
}
