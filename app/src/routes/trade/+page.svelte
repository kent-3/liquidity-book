<script lang="ts">
	import { fade, fly } from "svelte/transition";
	import { Accordion, AccordionItem } from '@skeletonlabs/skeleton';
	import { isAccountAvailable, secretClient, secretAddress, viewingKeys } from '$lib/stores';
	import { setupKeplr } from '$lib/keplr'
	import { tokenList, type Token } from '$lib/tokens';
	import type { CustomToken } from '$lib/contracts/misc_types';
	import { executeSwap } from "$lib/contracts/lb_pair";
	import { queryLBPairInformation } from "$lib/contracts/lb_factory";

	let routeFound = true;

	let tokenX: string = "TOKENX";
	let tokenY: string = "TOKENY";

	let swapForY: boolean = true;

    $: selectedtokenX = tokenList.find((token) => token.symbol === tokenX);
	$: selectedtokenY = tokenList.find((token) => token.symbol === tokenY);

	let amountX: number;
	let amountY: number;

	$: amount = swapForY
		? (amountX * 10 ** selectedtokenX!.decimals).toString()
		: (amountY * 10 ** selectedtokenY!.decimals).toString()
		
	$: testing = console.log(amount);

	// $: balanceX = getBalance(selectedtokenX);
	// $: balanceY = getBalance(selectedtokenY);

	// TODO still working on this
	// async function getSnip20Balance(token: Token) {
		// try {
		// 	const snip20Response = await $secretClient.query.snip20.getBalance({
		// 		contract: {
		// 			address: token.address,
		// 			code_hash: token.codeHash
		// 		},
		// 		address: $secretAddress,
		// 		auth: {
		// 			key: $viewingKeys.get(token.address)
		// 		}
		// 	})
		// 	let balance = Number((snip20Response.balance.amount as any) / 1e6).toString()
		// } catch (error) {
		// 	console.log(`No viewing key for SNIP20`)
		// }
	// }

    async function swap() {
        const tokenX: CustomToken = {
          custom_token: {
            contract_addr: selectedtokenX!.address,
            token_code_hash: selectedtokenX!.codeHash,
          }
        };

        const tokenY: CustomToken = {
          custom_token: {
            contract_addr: selectedtokenY!.address,
            token_code_hash: selectedtokenY!.codeHash,
          }
        };

        // TODO have a better way of knowing the LBPair contract info
        const { 
            lb_pair_information: { 
                lb_pair: { 
                    contract: { 
                        address: contractAddressPair, 
                        code_hash: contractHashPair 
                    } 
                } 
            } 
        } =  await queryLBPairInformation($secretClient, tokenX, tokenY, 100);

        // TODO allow inputs for amounts, liquidity config, etc.
        await executeSwap(
            $secretClient,
            contractHashPair,
            contractAddressPair,
            amount,
			swapForY,
        )
    }
</script>

<div in:fly="{{ x: 100, duration: 500 }}" class="flex flex-col p-4 h-full mx-auto justify-start items-center space-y-6 first:pt-8">
	<h2 class="font-semibold !text-3xl">Trade</h2>
	<!-- <div class="block text-center">
		<h2 class="font-semibold !text-3xl">Interface incomplete 😓</h2>
		<p class="">But you can still click the button to do an example transaction!</p>
	</div> -->
	<!-- TODO: Add a settings button with pop-up -->
	<div class="card xl:w-5/12 md:w-7/12 sm:w-9/12 w-full h-auto p-8 space-y-8 items-center">
		<div class="space-y-2">
			<div class="flex justify-between">
				<h2 class=" !text-base">Token X</h2>
				<button class="btn btn-sm py-0 px-2 hover:bg-secondary-500/20">Balance: 👀</button>
			</div>
			<div class="flex justify-between space-x-2">
				<input bind:value={amountX} disabled={!swapForY} class="input !bg-surface-50-900-token font-heading-token text-primary-900-50-token placeholder:text-primary-900-50-token/50 font-bold" type="number" name="" id="" placeholder="0.0"/>
				<select bind:value={tokenX} class="select w-36 !bg-surface-50-900-token font-heading-token text-primary-900-50-token placeholder:text-primary-900-50-token/50 font-bold" title="Select Token">
					<option value="TOKENX">TOKEN X</option>
					<option value="TOKENY">TOKEN Y</option>
					<option value="sSCRT">sSCRT</option>
					<option value="stkd-SCRT">stkd-SCRT</option>
					<option value="SHD">SHD</option>
					<option value="SILK">SILK</option>
					<option value="AMBER">AMBER</option>
				</select>
			</div>
		</div>
		<div class="flex space-x-2 justify-center items-center">
			<p>swapForY?</p>
			<input class=" checkbox" bind:checked={swapForY} type="checkbox"  />
		</div>
		<!-- <hr> -->
		<div class="space-y-2">
			<div class="flex justify-between">
				<h2 class="!text-base">Token Y</h2>
				<button class="btn btn-sm py-0 px-2 hover:bg-secondary-500/20">Balance: 👀</button>
			</div>
			<div class="flex justify-between space-x-2">
				<input bind:value={amountY} disabled={swapForY} class="input !bg-surface-50-900-token font-heading-token text-primary-900-50-token placeholder:text-primary-900-50-token/50 font-bold" type="text" name="" id="" placeholder="0.0"/>
				<select bind:value={tokenY} class="select w-36 !bg-surface-50-900-token font-heading-token text-primary-900-50-token placeholder:text-primary-900-50-token/50 font-bold" title="Select Token" id="to-token">
					<option value="TOKENX">TOKEN X</option>
					<option value="TOKENY">TOKEN Y</option>
					<option value="sSCRT">sSCRT</option>
					<option value="stkd-SCRT">stkd-SCRT</option>
					<option value="SHD">SHD</option>
					<option value="SILK">SILK</option>
					<option value="AMBER">AMBER</option>
				</select>
			</div>
		</div>
		{#if routeFound}
		<div class="border-2 border-primary-500-400-token !bg-surface-50-900-token">
			<Accordion regionPanel="space-y-1 items-center" regionControl="font-bold" regionCaret="fill-primary-400">
				<AccordionItem >
					<!-- <svelte:fragment slot="lead">(icon)</svelte:fragment> -->
					<svelte:fragment slot="summary">
						<span class="font-bold">
							{1} TOKENX = {1} TOKENY &ensp; (example only)
						</span>
					</svelte:fragment>
					<svelte:fragment slot="content">
						<div class="flex justify-between">
							<div class="text-sm text-primary-500-400-token ">
								Expected Output:
							</div>
							<div class="font-bold text-token text-sm">
								1 TOKENY
							</div>
						</div>
						<div class="flex justify-between">
							<div class="text-sm text-primary-500-400-token ">
								Minimum Received:
							</div>
							<div class="font-bold text-token text-sm">
								0.99 TOKENY
							</div>
						</div>
						<div class="flex justify-between">
							<div class="text-sm text-primary-500-400-token ">
								Price Impact:
							</div>
							<div class="font-bold text-token text-sm">
								0.1%
							</div>
						</div>
						<div class="flex justify-between">
							<div class="text-sm text-primary-500-400-token ">
								Fees:
							</div>
							<div class="font-bold text-token text-sm">
								0.01 TOKENX
							</div>
						</div>
					</svelte:fragment>
				</AccordionItem>
			</Accordion>
		</div>
		{/if}
		{#if $isAccountAvailable}
			<button on:click={()=>swap()} class="btn w-full variant-ghost-secondary mt-4 font-heading-token font-bold">
				Swap
			</button>
		{:else}
			<button on:click={()=>setupKeplr()} class="btn w-full variant-ghost-primary mt-4 font-heading-token font-bold">
				Connect Wallet
			</button>
		{/if}
	</div>
</div>
