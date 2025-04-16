# Compressed Program Template

This template initializes a counter program with instructions to create a compressed account, increment the accounts counter field and delete the account.

## Build

``
$ cargo build-sbf
``

## Test

Requirements:
- light cli

``
$ light start-prover --run-mode rpc && cargo test-sbf && kill $(lsof -t -i:3001)
``

The test spawns a prover server in the background.
In case of a connection refused error on port 3001 try to kill the prover server with `lsof -i:3001` and `kill <pid>`.


## Disclaimer

Programs are audited and deployed on Solana devnet and mainnet.
The light rust macros are experimental and api will change.
