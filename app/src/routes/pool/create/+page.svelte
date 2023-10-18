<script lang="ts">
	import type { PageData } from './$types';
	import { onMount } from 'svelte';
	import { fade, fly } from 'svelte/transition';
	import { setupKeplr } from '$lib/keplr'
	import { secretClient, isAccountAvailable } from '$lib/stores';
	import { tokenList, type Token } from '$lib/tokens';
	import { getIdFromPrice } from '$lib/utils';
	import { queryLBPairInformation } from '$lib/contracts/lb_factory'
	import { executeCreateLBPair } from '$lib/contracts/lb_router'

	export let tokenX: string = "TOKENX";
	export let tokenY: string = "TOKENY";

	export let activePrice: string = "1";
	export let binStep: string = "100";

	$: selectedtokenX = tokenList.find((token) => token.symbol === tokenX);
	$: logtokenX = console.log("token X:", selectedtokenX);

	$: selectedtokenY = tokenList.find((token) => token.symbol === tokenY);
	$: logtokenY = console.log("token Y:", selectedtokenY);

	$: selectedBinStep = binStep;
	$: logBinStep = console.log("bin step", selectedBinStep);
	
	$: activeId = getIdFromPrice(parseFloat(activePrice), parseInt(binStep))
	$: logActiveId = console.log("active bin id: ", activeId);

	async function createPool() {
		await executeCreateLBPair(
			$secretClient,
			selectedtokenX!.codeHash,
			selectedtokenX!.address,
			selectedtokenY!.codeHash,
			selectedtokenY!.address,
			activeId,
			parseInt(selectedBinStep),
		)
	}
</script>

<div in:fly="{{ x: 100, duration: 500 }}" class="flex flex-col p-4 h-full mx-auto justify-start items-center space-y-6 first:pt-8">
	<h2 class="font-semibold !text-3xl">Create New Pool</h2>
	<div class="card xl:w-1/2 md:w-2/3 w-full h-full p-8 items-center space-y-6">
		<label class="label">
			<span class="font-heading-token">Select Token</span>
			<select bind:value={tokenX} class="select !bg-surface-50-900-token font-heading-token text-secondary-600-300-token placeholder:text-secondary-600-300-token/50 font-bold" title="Select Token">
				<option value="TOKENX">TOKEN X</option>
				<option value="sSCRT">sSCRT</option>
				<option value="SHD">SHD</option>
				<option value="AMBER">AMBER</option>
				<option value="SILK">SILK</option>
			</select>
		</label>
		<label class="label">
			<span class="font-heading-token">Select Quote Asset</span>
			<select bind:value={tokenY} class="select !bg-surface-50-900-token font-heading-token text-secondary-600-300-token placeholder:text-secondary-600-300-token/50 font-bold" title="Select Quote Asset">
				<option value="TOKENY">TOKEN Y</option>
				<option value="sSCRT">sSCRT</option>
				<option value="stkd-SCRT">stkd-SCRT</option>
				<option value="SILK">SILK</option>
			</select>
		</label>
		<label class="label">
			<span class="font-heading-token">Select Bin Step</span>
			<div class="flex flex-row gap-4">
				<label class="flex items-center space-x-2">
					<input class="radio" type="radio" bind:group={binStep} name="radio-direct" value="25" />
					<p>0.25%</p>
				</label>
				<label class="flex items-center space-x-2">
					<input class="radio" type="radio" bind:group={binStep} name="radio-direct" value="50" />
					<p>0.5%</p>
				</label>
				<label class="flex items-center space-x-2">
					<input class="radio" type="radio" bind:group={binStep} name="radio-direct" value="100" />
					<p>1%</p>
				</label>
			</div>
		</label>
		<label class="label">
			<span class="font-heading-token">Enter Active Price</span>
			<input bind:value={activePrice} class="input !bg-surface-50-900-token font-heading-token text-secondary-600-300-token placeholder:text-secondary-600-300-token/50 font-bold" type="number" inputmode="decimal" title="Enter Active Price" placeholder="0.0"/>
		</label>
		{#if !$isAccountAvailable}
			<button on:click={()=>setupKeplr()} class="btn w-full variant-ghost-primary mt-4 font-heading-token font-bold">
				Connect Wallet
			</button>
		{:else}
			<button on:click={() => createPool()} class="btn w-full variant-ghost-secondary mt-4 font-heading-token font-bold">
				Create Pool
			</button>
		{/if}
	</div>
</div>