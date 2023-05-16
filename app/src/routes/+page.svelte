<script>
	import { onMount } from "svelte";
	import { fade, fly } from "svelte/transition";

	let ready = false
	onMount(() => (ready = true))
</script>

{#if ready}
<div in:fly="{{ x: 100, duration: 1000 }}" class="container h-full mx-auto flex justify-center space-x-24 items-center">
	<div class="inline-block space-y-2 w-2/3">
		<h1 class="font-semibold">Introduction</h1>
		<p>This project presents an efficient Automated Market Maker (AMM) protocol, modeled after the Liquidity Book protocol used by Trader Joe (<a href="https://docs.traderjoexyz.com/concepts/concentrated-liquidity" 
			target="_blank"
			rel="noopener noreferrer">
			  Liquidity Book docs
			</a>). The protocol retains the key features of its predecessor, such as: </p>
		<ul class="list-disc">
			<li class="list-item">
				<strong> No Slippage: </strong> 
				<span> Enabling token swaps with zero slippage within designated bins </span>
			</li>
			<li class="list-item">
				<strong> Adaptive Pricing: </strong> 
				<span> Offering Liquidity Providers extra dynamic fees during periods of increased market volatility </span>
			</li>
			<li class="list-item">
				<strong> Enhanced Capital Efficiency: </strong>
				<span> Facilitating high-volume trading with minimal liquidity requirements </span>
			</li>
			<li class="list-item">
				<strong> Customizable Liquidity: </strong> 
				<span> Liquidity providers can customize their liquidity distributions based on their strategy </span>
			</li>
		</ul>
		<h2 class="font-semibold">Liquidity Book vs Uniswap V3</h2>
		<p>Both Liquidity Book and Uniswap V3 are concentrated liquidity AMMs with some subtle differences:</p>
		<ul class="list-disc">
			<li>Price ranges are divided into bins instead of ticks</li>
			<li>Bins employ a constant sum invariant instead of constant product</li>
			<li>Bin steps can exceed 1 basis point</li>
			<li>Liquidity is aggregated vertically instead of horizontally</li>
			<li>Liquidity positions are fungible</li>
			<li>Liquidity positions can be distributed in any desired shape, without restrictions to uniform distribution across the price range</li>
			<li>Swap fees consist of fixed and variable pricing, enabling the AMM to impose higher fees during periods of heightened market volatility</li></ul>
	</div>
	<div class="text-center">
		<h2 class="font-bold"></h2>
		<!-- Animated Logo -->
		<figure>
			<section class="img-bg" />
			<svg class="fill-token -scale-x-[100%]" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"><!--! Font Awesome Pro 6.4.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license (Commercial License) Copyright 2023 Fonticons, Inc. --><path d="M456 0c-48.6 0-88 39.4-88 88v29.2L12.5 390.6c-14 10.8-16.6 30.9-5.9 44.9s30.9 16.6 44.9 5.9L126.1 384H259.2l46.6 113.1c5 12.3 19.1 18.1 31.3 13.1s18.1-19.1 13.1-31.3L311.1 384H352c1.1 0 2.1 0 3.2 0l46.6 113.2c5 12.3 19.1 18.1 31.3 13.1s18.1-19.1 13.1-31.3l-42-102C484.9 354.1 544 280 544 192V128v-8l80.5-20.1c8.6-2.1 13.8-10.8 11.6-19.4C629 52 603.4 32 574 32H523.9C507.7 12.5 483.3 0 456 0zm0 64a24 24 0 1 1 0 48 24 24 0 1 1 0-48z"/></svg>
		</figure>
		<!-- / -->
	</div>
</div>
{/if}

<style lang="postcss">
	figure {
		@apply flex relative flex-col;
	}
	figure svg,
	.img-bg {
		@apply  w-36 h-36 md:w-48 md:h-48;
	}
	.img-bg {
		@apply absolute z-[-1] rounded-full blur-[50px] transition-all;
		animation: pulse 5s cubic-bezier(0, 0, 0, 0.5) infinite,
			glow 5s linear infinite;
	}
	@keyframes glow {
		0% {
			@apply bg-primary-400/50;
		}
		33% {
			@apply bg-secondary-400/50;
		}
		66% {
			@apply bg-success-400/50;
		}
		100% {
			@apply bg-primary-400/50;
		}
	}
	@keyframes pulse {
		50% {
			transform: scale(1.5);
		}
	}
</style>
