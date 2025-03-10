#!/bin/bash

# Test the faucet service
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"requestFromFaucet","params":["0x1234567890abcdef1234567890abcdef12345678", 50],"id":1}' http://127.0.0.1:9933

# Test the Ethereum-compatible faucet service
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"ubi_requestFromFaucet","params":["0x1234567890abcdef1234567890abcdef12345678", 50],"id":1}' http://127.0.0.1:8545 