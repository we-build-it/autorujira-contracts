# Autoclaimer Contract

## Overview

The `Autoclaimer` contract is designed to automatically claim tokens from different protocols and then do something with those tokens. The current version only supports auto claiming & staking of `$AUTO` tokens.

## Features

- `DAO DAO` staking and `cw_rewards` support.
- Configurable fee percentages.
- Supports parallel claims, with a configurable maximum number of concurrent operations.

## Usage

### Building the Contract

To optimize the contract for deployment, run the following command:

```bash
cargo run-script optimize
```

This will use the Docker optimizer for CosmWasm contracts.

### Instantiation

Here’s an example instantiation message for the `Autoclaimer` contract:

```json
{
  "owner": "kujira1653fy3f609tnmm52r7f42rxqtlsaxn9v5g06fm",
  "max_parallel_claims": 5,
  "protocol_configs": [
    {
      "protocol": "AUTO",
      "provider": "CW_REWARDS",
      "fee_percentage": "0.01",
      "fee_address": "kujira1qj6p8m66zz5ru54xv9jzzlhff98l4nyy08lhzy",
      "claim_contract_address": "kujira19yyjw8ymr39lnvggacyxd37vmqmgwj05ur989f39gxzvj6nxeg3qkr394x",
      "stake_contract_address": "kujira15edk56qz43syg3hz0nv4ywrn7a6saw7p3ue0gdlzez4xsrf7gvkqrzkag7",
      "reward_denom": "factory/kujira1q2h7q5ynjfxl5xgz0zkw8xmnsrr9ssvp0zyrscy5tftkm58sn84sfukrwu/auto"
    }
  ]
}
```

### Configuration Parameters

- **owner**: The owner of the contract who has administrative privileges.
- **max_parallel_claims**: The maximum number of claims that can be processed simultaneously.
- **protocol_configs**: An array of configurations for each supported protocol. Each config includes:
  - `protocol`: The name of the protocol (e.g., `"AUTO"`).
  - `provider`: The staking provider (e.g., `"CW_REWARDS"`).
  - `fee_percentage`: The percentage of claimed rewards sent to the fee address.
  - `fee_address`: The address where fees are sent.
  - `claim_contract_address`: The contract address where claims are made.
  - `stake_contract_address`: The contract address where staking occurs.
  - `reward_denom`: The denomination of the reward tokens.

## Testing

To run the contract tests, simply run:

```bash
cargo test
```

This will execute all unit tests and integration tests defined in the project.

## Planned Updates

In future versions, the `Autoclaimer` will include:
- Support for claiming filled orders from FIN.
- Additional protocols and post claiming actions.