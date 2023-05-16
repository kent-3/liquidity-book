<script lang="ts">
	import { base } from '$app/paths'
	import { onMount } from 'svelte';
	import { fade, fly } from "svelte/transition";

	// TODO: get this info from the factory contract
	const tableArr = [
		{ pool_name: 'example', volume: '0', liquidity: 0, fees: '0', apr: 0 },
		{ pool_name: 'tbd', volume: '0', liquidity: 0, fees: '0', apr: 0 },
		{ pool_name: 'tbd', volume: '0', liquidity: 0, fees: '0', apr: 0 },
	];
</script>

<div in:fly="{{ x: 100, duration: 500 }}" class="flex flex-col p-6 h-full mx-auto justify-start items-left space-y-6 first:pt-8">
	<div class="flex flex-row items-center gap-12">
		<div class="space-y-0">
			<h2 class="font-semibold !text-4xl">Pool</h2>
			<p class="text-sm text-secondary-500-400-token">Provide liquidity and earn fees.</p>
		</div>
		<a href="{base}/pool/create" class="btn variant-ghost-secondary p-4 items-center h-3/4">
			Create New Pool
		</a>
	</div>

	<div class="table-container !w-3/4">
		<table class="table table-hover">
			<thead>
				<tr>
					<th>Pool Name</th>
					<th class="w-[5%] whitespace-nowrap">Volume (24H)</th>
					<th class="table-cell-fit">Liquidity</th>
					<th class="table-cell-fit">Fees (24H)</th>
					<th class="table-cell-fit">APR (24H)</th>
				</tr>
			</thead>
			<tbody>
				{#each tableArr as row, i}
					<tr>
						{#if i == 0}
						<a class="!text-primary-500 dark:!text-success-500 font-bold uppercase" href="{base}/pool/{row.pool_name}">
							<td>{row.pool_name} Pool (click me!)</td>
						</a>
						{:else}
						<td>{row.pool_name}</td>
						{/if}
						<td class="text-center">{row.volume}</td>
						<td class="text-center">{row.liquidity}</td>
						<td class="text-center">{row.fees}</td>
						<td class="text-center">{row.apr}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>

</div>