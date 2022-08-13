# Sei Transfer

This is a Sei smart contract built using CosmWasm for the Cosmos ecosystem. This contract allows a 1-to-2 transfer for a CW20 token.

## Features

- Instantiate the contract and set the owner, fee basis point, CW20 contract address and decimals
- Read query to get the owner of the contract
- Read query to get the withdrawable tokens for an address
- Read query to get the fee basis point
- Execute message to send tokens and specifying 2 recipients
- Execute message to withdraw tokens
- Execute message for the owner to set the fee basis point
- Storage to keep track of eligigle recipients for withdrawals
- Unit tests for the features listed above

## Instructions

- Run `aptos move test` in the project directory to run the tests
