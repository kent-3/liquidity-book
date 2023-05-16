# Trader Joe implementation on Secret Network

## Goals
Implement Trader Joe’s Liquidity Book with Secret Network contracts.

Milestone 1: Build a POC that allows you to launch a pool that implements the Liquidity Book spec, complete with a rudimentary front end that allows you to manage your LP position.

Milestone 2: Implement a pool factory and router that allows you to manage the deployment of pools permissionlessly, as well as route trades through multiple pools.

## Steps to Build and Run

### Build contracts:
```
make build-mainnet
```

### Integration test / deploy contracts:
```
make start-server
npx ts-node tests/integration.ts
```

To deploy to Pulsar, uncomment the three environment variables near the beginning of `tests/integration.ts`. Add your mnemonic to `.env_example` and rename the file to `.env`. The script is not very robust, so sometimes node connection issues will cause it to fail. Just try it again.

Manually copy (sorry) information from `contracts.log` to:
- `/app/src/lib/contracts.ts` - update the Liquidity Book Contracts section
- `/app/src/lib/tokens.ts` - update token X and Y addresses

A pre-configured front-end will be deployed to https://kent-3.github.io/trader-crow for demonstration.

### Run the front-end:
```
cd app
npm i
npm run dev -- --open
```

## Some Notes
During contract deployment, the LBFactory is instantiated first, then the LBRouter. The LBPair and LBToken contracts are only uploaded. Whenever a new pool is created, the LBFactory will instantiate an LBPair contract, and that LBPair contract will instantiate an LBToken contract.

## Other things
To generate documentation:
```
cargo doc --workspace --no-deps --open
```

To generate JSON schema, `cd` into the contract folder and run:
```
cargo run --bin schema
```
