# Trader Crow: Liquidity Book

Liquidity Book is an innovative, highly-capital efficient Automated Market Maker (AMM) protocol designed to support high volume trading with lower liquidity requirements. It introduces unique features that enhance the trading experience and optimize earnings for Liquidity Providers.

## Key Features

- **Zero Slippage**: Liquidity Book allows traders to swap tokens with zero slippage within designated bins, ensuring optimal trading conditions.
- **Surge Pricing**: During periods of high market volatility, Liquidity Providers earn additional dynamic fees, increasing their potential earnings.
- **High Capital Efficiency**: Unlike many existing AMMs, Liquidity Book supports high volume trading with significantly lower liquidity requirements.
- **Flexible Liquidity**: Liquidity Providers can strategically build flexible liquidity distributions according to their specific trading strategies.

## Comparing Liquidity Book and Uniswap V3

While both Liquidity Book and Uniswap V3 operate as concentrated liquidity AMMs, there are a few key differences:

| Feature                            | Liquidity Book                                                 | Uniswap V3                           |
| ---------------------------------- | -------------------------------------------------------------- | ------------------------------------ |
| Price Ranges                       | Discretized into bins                                          | Utilizes ticks                       |
| Invariant used                     | Constant sum                                                   | Constant product                     |
| Bin Steps/Tick Sizes               | Can be more than 1 basis point                                 | Generally 1 basis point              |
| Liquidity Aggregation              | Vertically aggregated                                          | Horizontally aggregated              |
| Fungibility of Liquidity Positions | Yes                                                            | No                                   |
| Liquidity Distribution             | Not restricted to uniform, can take any desired shape          | Typically uniform across price range |
| Swap Fees                          | Fixed + variable pricing, allows higher fees during volatility | Fixed pricing                        |
| Zero Slippage                      | Yes                                                            | No                                   |
| Surge Pricing                      | Yes                                                            | No                                   |
| High Capital Efficiency            | Yes                                                            | Variable                             |
| Flexible Liquidity Distributions   | Yes                                                            | Limited                              |

## Project Structure

This repository contains the Liquidity Book contracts, as well as tests and deploy scripts.

- The [LBPair](./contracts/LbPair/src/contract.rs) is the contract that contains all the logic of the actual pair for swaps, adds, removals of liquidity and fee claiming. This contract should never be deployed directly, and the factory should always be used for that matter.

- The [LBToken](./contracts/LbToken/src/contract.rs) is the contract that is used to calculate the shares of a user. The LBToken is a new token standard that is similar to SNIP-1155.

- The [LBFactory](./contracts/LbFactory/src/contract.rs) is the contract used to deploy the different pairs and acts as a registry for all the pairs already created. There are also privileged functions such as setting the parameters of the fees, the flashloan fee, setting the pair implementation, set if a pair should be ignored by the quoter and add new presets. Only the owner of the factory can create pairs.

- The [LBRouter](./contracts/LbRouter/src/contract.rs) is the main contract that user will interact LbPair when swapping

- The [LBQuoter](./contracts/LbQuoter/src/contract.rs) is a contract that is used to return the best route of all those given. This should be used before a swap to get the best return on a swap.

## Notice!
You'll need to grant allowances for testnet sSCRT and SILK to the LBPair contract:

`address`: secret1vdp8lm27h7d906fg0d7g59jnn32sdl7x2m8sea

`code_hash`: aff1a59f3886b7f0a2d20e8ac9ed3628fd11d4b7df2e6a69ebd7cb481b03c70f

## Investors Pitch
[Link](https://drive.google.com/file/d/1o9ItsmxIAFk6MZ9eBJYM_J0UMz-9FfrM/view?usp=sharing)

## Development Description 
[Link](https://docs.google.com/document/d/1li_wg_TLZPVSEPnrJ33Ig3AxIodgJCG5UK7Fv6NaGYY)


## Misc Notes
- after each testnet contract deployment, the following variables need to be updated:
  - /app/src/lib/contracts.ts - update the Liquidity Book Contracts section
  - /app/src/lib/tokens.ts - update token X and Y

## Developing

Once you've created a project and installed dependencies with `npm install` (or `pnpm install` or `yarn`), start a development server:

```bash
npm run dev

# or start the server and open the app in a new browser tab
npm run dev -- --open
```

## Building

To create a production version of your app:

```bash
npm run build
```

You can preview the production build with `npm run preview`.
