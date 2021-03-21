# Timekeeper

Timekeeper is a secret contract that uses the Bitcoin network to act as a time reference on the Secret Network. It's meant to serve as a workaround until there's a way to verify the block height within secret enclaves.


The contract stores a start block height, along with a current offset and a block hash.

This offset is updated by doing [verification](https://en.bitcoin.it/wiki/Hashcash) on block header values. Anyone who updates it must provide valid consecutive block headers, where the first header provided references the current block hash.

In order to make the process more secure, we require a minimum number of headers to be provided in a single call. We also require block header difficulty values to be harder than a threshold difficulty, and check the difficulty against the hash computed from the header values.

## Status
The contract is currently in development, the proof of concept (passing unit tests) is complete so far.


## Notes

[Publishing](./Publishing.md) contains useful information on how to publish your contract
to the world, once you are ready to deploy it on a running blockchain. And
[Importing](./Importing.md) contains information about pulling in other contracts or crates
that have been published.
