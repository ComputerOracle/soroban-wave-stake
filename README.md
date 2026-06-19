# soroban-wave-stake 🌊🥩

A production-grade, anti-ghosting commit-to-work protocol built natively for Stellar Soroban smart contract engines. Designed to integrate directly into continuous integration workflows and agile developer grant pipelines such as **Drips Wave**.

## Problem Statement

Continuous distributed grant systems face structural task-hogging bottlenecks. A developer signals intent by claiming an issue, locking out alternative community capacity, and then goes missing (ghosts). This stalls product timelines and burns considerable maintainer coordination resources.

## Solution Architecture

`soroban-wave-stake` establishes an explicit, trustless micro-staking economic penalty to align developer incentives:

- **Exclusivity via Staking:** Contributors allocate a tiny financial commitment (e.g., 20 USDC) to temporarily lock assignment rights.
- **Milestone Distribution:** Merged Pull Requests release the original escrow balance, any accumulated rollover incentives, and the base bounty payout.
- **Automated Slashed Rollovers:** Missing a deadline authorizes core maintainers to slash the stake. The forfeited funds pool rolls into the specific target issue, multiplying the payout value for subsequent developers.

## Quickstart

### Smart Contract Execution Engine

```bash
# Verify unit/integration testing suite locally
make test

# Compile production optimized bytecode artifacts
make build
```

### Deployment Protocol (Stellar Testnet)

Ensure you have an account profile configured within your Stellar CLI:

```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh
```
