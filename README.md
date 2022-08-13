# Sei Transfer

This is a Sei smart contract built using CosmWasm for the Cosmos ecosystem. This contract allows a 1-to-2 transfer for a CW20 token.

## Features

- Instantiate the contract and set the owner, fee basis point, CW20 contract address and decimals
- Read query to get the owner of the contract
- Read query to get the withdrawable tokens for an address
- Read query to get the CW20 contract address
- Read query to get the CW20 decimals
- Read query to get the fee basis point
- Execute message to send tokens and specifying 2 recipients
- Execute message to withdraw tokens
- Execute message for the owner to set the fee basis point
- Storage to keep track of eligigle recipients for withdrawals
- Unit tests for the features listed above

## Prerequisites

Before starting, make sure you have [rustup](https://rustup.rs/) along with a
recent `rustc` and `cargo` version installed. Currently, this is running on 1.58.1+.

And you need to have the `wasm32-unknown-unknown` target installed as well.

You can check that via:

```sh
rustc --version
cargo --version
rustup target list --installed
# if wasm32 is not listed above, run this
rustup target add wasm32-unknown-unknown
```

## Compiling and running tests

Now that you created your custom contract, make sure you can compile and run it before
making any changes. Go into the repository and do:

```sh
# this will produce a wasm build in ./target/wasm32-unknown-unknown/release/YOUR_NAME_HERE.wasm
cargo wasm

# this runs unit tests with helpful backtraces
RUST_BACKTRACE=1 cargo unit-test

# auto-generate json schema
cargo schema
```
