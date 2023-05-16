<script lang='ts'>
	import '../theme.postcss';
	import '@skeletonlabs/skeleton/styles/all.css';
	import '../app.postcss';

	import { base } from '$app/paths'
	import Wallet from '$lib/Wallet.svelte';
	import { AppShell, AppBar, Modal } from '@skeletonlabs/skeleton';
	import { LightSwitch } from '@skeletonlabs/skeleton';
	import { computePosition, autoUpdate, flip, shift, offset, arrow } from '@floating-ui/dom';
	
	import { chains } from '$lib/config';
	import { popup, storePopup, type PopupSettings } from '@skeletonlabs/skeleton';
	import { Drawer, drawerStore, type DrawerSettings } from '@skeletonlabs/skeleton';
	import { modalStore, type ModalSettings, type ModalComponent } from '@skeletonlabs/skeleton';
	import { Toast, toastStore } from '@skeletonlabs/skeleton';
	import type { ToastSettings } from '@skeletonlabs/skeleton';

	import { testModal, testToasts } from '$lib/tests-ui';

	storePopup.set({ computePosition, autoUpdate, flip, shift, offset, arrow });

	function debug() {
		testToasts()
		testModal()
	}
</script>

<Toast duration=400 width=" sm:max-w-[32rem] sm:min-w-[24rem]" position="br" background="variant-glass-secondary" buttonDismiss="btn-icon btn-icon-sm variant-glass" buttonAction="btn btm-sm variant-filled" max=6 />
<Modal width="" regionBody="max-h-[440px]" regionBackdrop="bg-surface-backdrop-token" />
<!-- App Shell -->
<AppShell>
	<svelte:fragment slot="header">
		<!-- App Bar -->
		<AppBar background="bg-surface-50-900-token" shadow="shadow-md">
			<svelte:fragment slot="lead">
				<a
					class="btn pl-0 pr-2 py-0"
					href="https://github.com/kent-3/trader-joe"
					target="_blank"
					rel="noopener noreferrer"
				>
					<svg class="fill-primary-400 h-6 pr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"><!--! Font Awesome Pro 6.4.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license (Commercial License) Copyright 2023 Fonticons, Inc. --><path d="M456 0c-48.6 0-88 39.4-88 88v29.2L12.5 390.6c-14 10.8-16.6 30.9-5.9 44.9s30.9 16.6 44.9 5.9L126.1 384H259.2l46.6 113.1c5 12.3 19.1 18.1 31.3 13.1s18.1-19.1 13.1-31.3L311.1 384H352c1.1 0 2.1 0 3.2 0l46.6 113.2c5 12.3 19.1 18.1 31.3 13.1s18.1-19.1 13.1-31.3l-42-102C484.9 354.1 544 280 544 192V128v-8l80.5-20.1c8.6-2.1 13.8-10.8 11.6-19.4C629 52 603.4 32 574 32H523.9C507.7 12.5 483.3 0 456 0zm0 64a24 24 0 1 1 0 48 24 24 0 1 1 0-48z"/></svg>
				</a>
				<button on:click={debug}>
					<strong class="text-xl uppercase font-heading-token">Trader Crow</strong>
				</button>
			</svelte:fragment>
			<svelte:fragment>
				<div class="pl-4 flex space-x-4">
					<a href="{base}/" class="">
						<button class="btn font-heading-token font-bold variant-ghost-secondary">Home</button>
					</a>
					<!-- TODO: include a page with explanations of the bin math, tree index, and encoding -->
					<a href="{base}/demo" class="">
						<button class="btn font-heading-token font-bold variant-ghost-secondary">Demo</button>
					</a>
					<a href="{base}/trade" class="">
						<button class="btn font-heading-token font-bold variant-ghost-secondary">Trade</button>
					</a>
					<a href="{base}/pool" class="">
						<button class="btn font-heading-token font-bold variant-ghost-secondary">Pool</button>
					</a>
				</div>
			</svelte:fragment>

			<svelte:fragment slot="trail">
				<LightSwitch height="h-6" />
				<Wallet/>
			</svelte:fragment>
		</AppBar>
	</svelte:fragment>
	<!-- Page Route Content -->
	<slot />
	<svelte:fragment slot="pageFooter">
		<div class="container ml-auto p-1 flex justify-end items-center">
			<code class="unstyled py-0.5 px-1 rounded text-xs font-mono whitespace-nowrap bg-secondary-500/30 text-secondary-800 dark:bg-primary-500/30 dark:text-primary-400"> connected to 
				<a 
					href="https://secret.express"
					target="_blank"
					rel="noopener noreferrer"
					class="unstyled text-secondary-700 dark:text-secondary-500 underline"
				>
					{chains['Secret Network'].lcd}
				</a>
			</code>
		</div>
	</svelte:fragment>
</AppShell>
