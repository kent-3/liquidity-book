# Concentrated Liquidity

## Introduction

In a traditional AMM like Uniswap V2, liquidity is spread evenly across the entire price range from 0 to ∞. This means a lot of liquidity remains unused, especially for pairs that consistently trade within a narrow range.

For example, USDC/USDT mostly trades between $0.99 and $1.01. Liquidity outside this range is rarely utilized and could be put to better use elsewhere.

With Liquidity Book (LB), liquidity providers (LPs) can allocate liquidity within a specific price range—this is called concentrated liquidity. Using USDC/USDT as an example, an LP providing liquidity between $0.99 and $1.01 earns trading fees as long as the price stays within that range.

## Bin Pricing

Liquidity Book structures liquidity into discrete bins with a fixed price width. Within each bin, trades execute at a fixed price. The difference between two consecutive bins is called the bin step.

For instance, if the current USDC/USDT price is $1 and the bin step is 1 basis point (0.01%), the next bin up is:
\\[ 1×1.0001=1.0001 \\]

and the next after that:
\\[ 1.0001×1.0001=1.00020001 \\]

This forms a geometric sequence of price levels: \\( 1.0001^n \\)

Bin steps are set by the pool creator, allowing multiple markets for the same pair with different bin steps. A Liquidity Book pool is uniquely identified by the tuple (X, Y, s), where X and Y are the pooled assets, and s is the bin step.

## Liquidity Book vs Uniswap V3

- Uniswap V3 concentrates liquidity within a fixed range, following a continuous X \* Y = k curve.
- Liquidity Book breaks liquidity into fixed-width bins, where price follows a X + Y = k rule within each bin.
- TL;DR: Uniswap V3 uses a constant product formula between ticks, while Liquidity Book applies a constant sum formula within bins.
