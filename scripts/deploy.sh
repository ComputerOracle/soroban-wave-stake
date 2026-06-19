#!/bin/bash

set -e

# Configuration settings
NETWORK="testnet"
SOURCE_ACCOUNT="default"

echo "===================================================="
echo "⚡ Starting production build compilation of contract"
echo "===================================================="

cd contracts/soroban-wave-stake
cargo build --target wasm32-unknown-unknown --release

echo "===================================================="
echo "📦 Running Soroban WASM binary code optimizer"
echo "===================================================="

stellar contract optimize --wasm target/wasm32-unknown-unknown/release/soroban_wave_stake.wasm

echo "===================================================="
echo "🚀 Deploying optimized bytecode to Stellar $NETWORK"
echo "===================================================="

CONTRACT_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/soroban_wave_stake.optimized.wasm \
    --source $SOURCE_ACCOUNT \
    --network $NETWORK)

echo "===================================================="
echo "✅ Operational Deployment Complete!"
echo "Target Contract Address: $CONTRACT_ID"
echo "===================================================="

# Sync to local environments for quick frontend consumption
echo "NEXT_PUBLIC_WAVESTAKE_CONTRACT_ADDRESS=$CONTRACT_ID" > ../../frontend/.env.local
