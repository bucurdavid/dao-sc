# Fellowship DAOs Smart Contracts

Smart contract used for managing and running DAOs through [Superciety](https://superciety.com) on Elrond Blockchain.

Specifically, two smart contracts:

- The Entity Template: Is the actual DAO smart contract that users interact with
- The Manager: Deploys and manages instances of the Entity Template smart contract & contains other utilities

Mainnet (Manager): [erd1qqqqqqqqqqqqqpgq4kns8he9r84c58ed3jjuam3tp7u9zl4n27rsy2kv6u](https://explorer.elrond.com/accounts/erd1qqqqqqqqqqqqqpgq4kns8he9r84c58ed3jjuam3tp7u9zl4n27rsy2kv6u)

Mainnet (Entity Template): [erd1qqqqqqqqqqqqqpgqces4kdydtp9ea29pymjjyg7vcfqfllly27rsv3qats](https://explorer.elrond.com/accounts/erd1qqqqqqqqqqqqqpgqces4kdydtp9ea29pymjjyg7vcfqfllly27rsv3qats)

## Requirements

- Contract must possess the `ESDTRoleLocalMint` role for the configured token of `boost-reward-token-id` – [SUPERPOWER-6f4cee](https://explorer.elrond.com/tokens/SUPERPOWER-6f4cee) in our case

## Deploy

Before deploying the smart contract to the blockchain, be sure to:

1. Remove the `exit` part within the `deploy` function in `interaction/manager.sh` to disable deploy protection.
2. Configure all variables within `erdpy.data-storage.json` for the corresponding network.
3. Connect & unlock your Ledger device with the Elrond app open, ready to sign the deploy transaction.

```bash
. ./interaction/manager.sh && deploy
```

## Upgrade

To upgrade the Manager smart contract:

```bash
. ./interaction/manager.sh && upgrade
```

To upgrade the Entity Template smart contract:

```bash
. ./interaction/manager.sh && upgradeEntityTemplate
```

## Security Vulnerabilities

Please review [our security policy](../../security/policy) on how to report security vulnerabilities.

## Credits

- [Micha Vie](https://github.com/michavie)
- [All Contributors](../../contributors)

## License

The GNU GENERAL PUBLIC LICENSE v3.0. Please see [License File](LICENSE) for more information.
