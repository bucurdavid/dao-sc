##### - configuration - #####
NETWORK_NAME="devnet" # devnet, testnet, mainnet
DEPLOYER="./deployer.pem" # main actor pem file
PROXY=https://devnet-gateway.elrond.com
CHAIN_ID="D"

ADDRESS="erd1qqqqqqqqqqqqqpgqut7243tkk5lhqshld28ssn2w9e8vkgmyyt2spf0sdj"

##### - configuration end - #####

COST_TOKEN_ID=$(erdpy data load --partition ${NETWORK_NAME} --key=cost-token-id)
COST_TOKEN_ID_HEX="0x$(echo -n ${COST_TOKEN_ID} | xxd -p -u | tr -d '\n')"

getGovQuorum() {
    erdpy contract query ${ADDRESS} \
        --function="getQuorum" \
        --proxy=${PROXY} || return
}
