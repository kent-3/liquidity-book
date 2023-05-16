<script lang="ts" type="module">
	import { onMount } from 'svelte'
	import { setupKeplr, getKeplrViewingKey, disconnectKeplr } from '$lib/keplr'
	import { chains } from '$lib/config'
	import type { Token } from '$lib/tokens'
	import { compactAddress } from '$lib/utils'
	import {
		resetStores,
		amberBalance,
		scrtBalance,
		isAccountAvailable,
		keplrKey,
		secretAddress,
		secretClient,
		viewingKeys,
	} from '$lib/stores'
	import { modalStore, type ModalSettings } from '@skeletonlabs/skeleton';
	import { popup, type PopupSettings } from '@skeletonlabs/skeleton';

	// TODO implement a modal that displays details of the message before requesting signature
	// TODO implement wallet choice modal on click connect button

	let popupSettings: PopupSettings = {
		// Set the event as: click | hover | hover-click
		event: 'click',
		placement: 'bottom-end',
		// Provide a matching 'data-popup' value.
		target: 'walletMenu',
		closeQuery: '.btn',
	};

	const alert: ModalSettings = {
		type: 'alert',
		title: 'Attention',
		body: 'This is an example alert',
		// image: '/image.webp',
		buttonTextCancel: 'OK',
		modalClasses: 'w-modal-slim',
		backdropClasses: '',
	};

	// TODO combine these two triggers
	function triggerConfirm(): void {
		const confirm: ModalSettings = {
			type: 'confirm',
			title: 'Please Confirm',
			body: 'Are you sure you wish to proceed?',
			modalClasses: 'ring-secondary-500 ring-1',
			// TRUE if confirm pressed, FALSE if cancel pressed
			response: (r: boolean) => {if (r) {disconnectKeplr()}},
			// Optionally override the button text
			buttonTextCancel: 'Cancel',
			buttonTextConfirm: 'Disconnect',
		};
		modalStore.trigger(confirm);
	}

	// TODO make some kind of 'user' object that can be saved to localstorage?

	let user = { loggedIn: false };

	function toggle() {
		user.loggedIn = !user.loggedIn;
	}

	const SECRET_CHAIN_ID = chains['Secret Network'].chain_id

	onMount(() => {
		window.addEventListener('keplr_keystorechange', async () => {
			console.log('Key store in Keplr is changed. You may need to refetch the account info.')
			await resetStores()
			await connect()
		})

		// TODO check if account is already available to update the button state
	})

	export async function connect() {
		// resetStores()
		await setupKeplr()
		// await getViewingKeys(tokenList)
		await getBalances()
	}

	async function getViewingKeys(tokens:Token[]) {
		for (const token of tokens) {
			const key = await getKeplrViewingKey(token.address)
			if (key != null) {
				viewingKeys.update(map => map.set(token.address, key))
			}
		}
	}
	
	async function getBalances() {
		try {
			const response = await $secretClient.query.bank.balance({
				address: $secretAddress,
				denom: 'uscrt'
			})
			$scrtBalance = Number((response.balance?.amount as any) / 1e6).toString()
		} catch (error) {
			console.log(error)
		}
	}
</script>

<div
	class="card !ring-secondary-500 p-4 w-64 shadow-2xl transition-opacity -translate-x-[11px]"
	data-popup="walletMenu"
>
	<div class="text-center space-y-2">
		<p class="text-center font-heading-token font-bold drop-shadow-md">
			[ {$keplrKey.name} ]
		</p>
		<hr class="!border-t-2 !border-secondary-500/25" />
		<p class="font-bold font-mono text-primary-600 dark:text-primary-400">{$scrtBalance} SCRT</p>
		<button
			on:click={()=>triggerConfirm()}
			on:keypress={()=>triggerConfirm()}
			class="btn btn-sm px-8 font-medium variant-ghost-secondary"
		>
			Disconnect
		</button>
	</div>
	<!-- Append the arrow element -->
	<!-- <div class="arrow bg-surface-800" /> -->
</div>

<div class="flex flex-row-reverse flex-nowrap md:gap-4 items-center">
	<!-- Alternate method for different button states -->
	<!-- <button 
		class="{$isAccountAvailable == true ? 'btn variant-ghost-secondary' : 'btn variant-ghost-primary'}"
		on:click={() => connect()}
	>
		{$isAccountAvailable == false ? 'Connect Wallet' : compactAddress($keplrKey.bech32Address)}
	</button> -->
	{#if $isAccountAvailable}
		<button 
			class="btn font-bold font-heading-token variant-ghost-secondary pr-6"
			use:popup={popupSettings}
		>
			<svg viewBox="0 0 21 18" fill="none" xmlns="http://www.w3.org/2000/svg" focusable="false" class=" fill-token mr-3 w-[21px] h-[18px]" aria-hidden="true"><path fill-rule="evenodd" clip-rule="evenodd" d="M0 3C0 1.34315 1.34315 0 3 0H16C17.6569 0 19 1.34315 19 3V5.17071C20.1652 5.58254 21 6.69378 21 8V10C21 11.3062 20.1652 12.4175 19 12.8293V15C19 16.6569 17.6569 18 16 18H3C1.34315 18 0 16.6569 0 15V3ZM17 3V5H14C12.3431 5 11 6.34315 11 8V10C11 11.6569 12.3431 13 14 13H17V15C17 15.5523 16.5523 16 16 16H3C2.44772 16 2 15.5523 2 15V3C2 2.44772 2.44772 2 3 2H16C16.5523 2 17 2.44772 17 3ZM14 7C13.4477 7 13 7.44772 13 8V10C13 10.5523 13.4477 11 14 11H18C18.5523 11 19 10.5523 19 10V8C19 7.44772 18.5523 7 18 7H14ZM17 9C17 9.55228 16.5523 10 16 10C15.4477 10 15 9.55228 15 9C15 8.44771 15.4477 8 16 8C16.5523 8 17 8.44771 17 9Z" fill="current"></path></svg>
			{compactAddress($secretAddress)}
		</button>
	{:else}
		<button 
			class="btn font-bold font-heading-token variant-ghost-primary pr-6"
			on:click={() => connect()}
			>
			<!-- on:click={() => modalStore.trigger(alert)} -->
			<svg viewBox="0 0 21 18" fill="none" xmlns="http://www.w3.org/2000/svg" focusable="false" class=" fill-token mr-3 w-[21px] h-[18px]" aria-hidden="true"><path fill-rule="evenodd" clip-rule="evenodd" d="M0 3C0 1.34315 1.34315 0 3 0H16C17.6569 0 19 1.34315 19 3V5.17071C20.1652 5.58254 21 6.69378 21 8V10C21 11.3062 20.1652 12.4175 19 12.8293V15C19 16.6569 17.6569 18 16 18H3C1.34315 18 0 16.6569 0 15V3ZM17 3V5H14C12.3431 5 11 6.34315 11 8V10C11 11.6569 12.3431 13 14 13H17V15C17 15.5523 16.5523 16 16 16H3C2.44772 16 2 15.5523 2 15V3C2 2.44772 2.44772 2 3 2H16C16.5523 2 17 2.44772 17 3ZM14 7C13.4477 7 13 7.44772 13 8V10C13 10.5523 13.4477 11 14 11H18C18.5523 11 19 10.5523 19 10V8C19 7.44772 18.5523 7 18 7H14ZM17 9C17 9.55228 16.5523 10 16 10C15.4477 10 15 9.55228 15 9C15 8.44771 15.4477 8 16 8C16.5523 8 17 8.44771 17 9Z" fill="current"></path></svg>
			Connect Wallet
		</button>
	{/if}
	<!-- {#if $keplrKey.name != ""}
		<div in:fade class="hidden lg:block dark:text-secondary-400 font-bold transition-all ease-standard duration-300">
			[ {$keplrKey.name} ]
		</div>
	{/if} -->
</div>