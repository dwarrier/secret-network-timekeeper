# Timekeeper

Timekeeper is a contract that uses the Bitcoin network to act as a time reference on the Secret Network. It's meant to serve as a workaround until there's a way to verify the block height within secret enclaves.


## How it works

The contract stores a start block height, along with a current offset and a block hash.

This offset and block hash are updated after doing [verification](https://en.bitcoin.it/wiki/Hashcash) on block header values. The user performing the update must provide valid consecutive block headers, where the first header provided references the current block hash.

In order to make the process more secure, we require a minimum number of headers to be provided in a single call. We also require block header difficulty values to be harder than a threshold difficulty, and check the declared block difficulty against the hash computed from the header values.

## Status
The contract is currently in development and can be tested in a local dev environment.

## Testing locally
See the local testnet [deploy instructions](https://build.scrt.network/dev/quickstart.html#deploy-smart-contract-to-our-local-testnet). Here are some commands you can use to interact with the contract:

Assume these commands have already been run:
```shell
docker exec -it secretdev /bin/bash

cd code

secretcli tx compute store contract.wasm.gz --from a --gas 1000000 -y --keyring-backend test
```

Instantiate the contract with a short minimum update length of 3 blocks:
```shell
INIT='{"start_height": 125551, "min_difficulty_bits": 453248203, "start_hash": "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000", "min_update_length": 3}'
CODE_ID=1
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "time keeper 1" -y --keyring-backend test
```
Set the contract address:
```shell
CONTRACT=$(secretcli query compute list-contract-by-code 1 | jq -r '.[0].address')
```

Get the contract info to see the current offset and start hash:
```shell
secretcli query compute query $CONTRACT '{"get_contract_info": {}}'
```
Update the offset and add 4 blocks:
```shell
secretcli tx compute execute $CONTRACT '{"update_block_offset": {"block_headers": ["0100000081cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000e320b6c2fffc8d750423db8b1eb942ae710e951ed797f7affc8892b0f1fc122bc7f5d74df2b9441a42a14695", "010000001dbd981fe6985776b644b173a4d0385ddc1aa2a829688d1e0000000000000000b371c14921b20c2895ed76545c116e0ad70167c5c4952ca201f5d544a26efb53b4f6d74df2b9441a071a0c81", "0100000085afcb448a3fcde31dc78babd352d9dbde6fcb566777ea33051c000000000000ca5b6b96fe65e1a7d50e7c3025a176472ba26d44512de86a6f3e39649330cd2f16f7d74df2b9441a8574adaf", "010000001e60224709df1feb2e2849b7b10570abf7d4355ba8e2f6df121100000000000028cc65b7be2f8a1edc2af86ef369472443a1b70479cee205e8db5440cfbe943f57fad74df2b9441acc24ce5b"]}}' --from a --keyring-backend test
```
Get the contract info again. The offset and current hash should be updated:
```shell
secretcli query compute query $CONTRACT '{"get_contract_info": {}}'
```
Reset the contract state to the initial state:
```shell
RESET='{"reset_state" : {"new_state" : {"start_height": 0, "min_difficulty_bits": 453248203, "start_hash": "81cd02ab7e569e8bcd9317e2fe99f2de44d49ab2b8851ba4a308000000000000", "min_update_length": 3}}}'
secretcli tx compute execute $CONTRACT "$RESET" --from a --keyring-backend test
```
Check the contract state to make sure it succeeded:
```shell
secretcli query compute query $CONTRACT '{"get_contract_info": {}}'
```


## Misc. Notes

[Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain. And
[Importing](./Importing.md) contains information about pulling in other contracts or crates
that have been published.
