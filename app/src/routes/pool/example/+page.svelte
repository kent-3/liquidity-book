<script lang="ts">
    import { fade, fly } from "svelte/transition";
    import { Tab, TabGroup } from '@skeletonlabs/skeleton';
    import DoubleRangeSlider from '$lib/DoubleRangeSlider.svelte';
	import { setupKeplr } from '$lib/keplr'
    import { secretClient, isAccountAvailable } from '$lib/stores';
    import { tokenList } from '$lib/tokens';
    import { spotUniform, curve, bidAsk } from '$lib/joe-sdk/constants'
    import { getIdFromPrice } from '$lib/utils';
    import type { CustomToken, TokenType } from '$lib/contracts/misc_types';
    import { queryLBPairInformation } from '$lib/contracts/lb_factory';
    import { executeAddLiquidity, executeRemoveLiquidity } from '$lib/contracts/lb_pair';

    let start = 0.45;
    let end = 0.56;

    function reset() {
        start = 0.45;
        end = 0.56;
    }

    const nice = (d: number) => {
        if (!d && d !== 0) return '';
        return d.toFixed(2);
    }

    let _tokenX = tokenList.find((token) => token.symbol === "sSCRT");
    let _tokenY = tokenList.find((token) => token.symbol === "SILK");

    let inputX: number;
    let inputY: number;

    $: amountX = inputX * (10 ** _tokenX!.decimals)
    $: amountY = inputY * (10 ** _tokenY!.decimals)

    // TODO: match shape number to liquidity configuration
    let shape = 0;
    let binRadius = 5;
    let price = 1;
    let binStep = 100;

    let tabSet: number = 0;
    let tabSetBy: number = 0;
	

	// $: activeId = getIdFromPrice(parseFloat(activePrice), parseInt(binStep))
	// $: logActiveId = console.log("active bin id: ", activeId);

    async function addLiquidity() {
        const tokenX: CustomToken = {
          custom_token: {
            contract_addr: _tokenX!.address,
            token_code_hash: _tokenX!.codeHash,
          }
        };

        const tokenY: CustomToken = {
          custom_token: {
            contract_addr: _tokenY!.address,
            token_code_hash: _tokenY!.codeHash,
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
        await executeAddLiquidity(
            $secretClient,
            contractHashPair,
            contractAddressPair,
            binStep,
            tokenX,
            tokenY,
            amountX,
            amountY,
        )
    }

    async function removeLiquidity() {
        const tokenX: CustomToken = {
          custom_token: {
            contract_addr: _tokenX!.address,
            token_code_hash: _tokenX!.codeHash,
          }
        };

        const tokenY: CustomToken = {
          custom_token: {
            contract_addr: _tokenY!.address,
            token_code_hash: _tokenY!.codeHash,
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
        await executeRemoveLiquidity(
            $secretClient,
            contractHashPair,
            contractAddressPair,
            binStep,
            tokenX,
            tokenY,
        )
    }
    
</script>

<div in:fly="{{ x: 100, duration: 500 }}" class="flex xl:flex-row flex-col p-4 h-full mx-auto xl:justify-even justify-center xl:items-start items-center gap-6">
	<!-- <h2 class="font-semibold !text-3xl self-center">Manage Liquidity</h2> -->
    <div class="card variant-ghost-surface xl:w-[49%] w-full h-full px-8 py-6 items-center space-y-6">
        <h2 class="font-semibold !text-3xl text-center">Manage Liquidity</h2> 
        <p> Nothing to see here yet 😓 </p>
    </div>
	<div class="card xl:w-[49%] w-full h-auto px-8 py-6 items-center space-y-6">
        <TabGroup 
        justify="justify-evenly"
        active="variant-ghost-secondary"
        hover=""
        flex="flex"
        rounded=""
        border="border-[1px] border-surface-500 p-1"
        padding="py-1 px-8 w-full"
        spacing="space-x-1"
        class="font-heading-token font-bold"
        >
        <Tab bind:group={tabSet} name="add" value={0} class="w-full">
            Add Liquidity
        </Tab>
        <Tab bind:group={tabSet} name="remove" value={1}>
            Remove Liquidity
        </Tab>
        <!-- ... -->
        <svelte:fragment slot="panel">
            {#if tabSet === 0}
            <div in:fade={{duration: 200}} class="space-y-8">
                <div class="space-y-2">
                    <h2 class="!text-xl !font-semibold">Deposit Liquidity</h2>
                    <div class="input-group input-group-divider grid-cols-[1fr_80px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                        <input bind:value={inputX} type="number" name="" id="" placeholder="Enter Amount" />
                        <div class="input-group-shim font-normal text-sm !justify-center">sSCRT</div>
                    </div>
                    <div class="input-group input-group-divider grid-cols-[1fr_80px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                        <input bind:value={inputY} type="number" name="" id="" placeholder="Enter Amount"/>
                        <div class="input-group-shim font-normal text-sm !justify-center">SILK</div>
                    </div>
                </div>
                <div class="space-y-2">
                    <h2 class="!text-xl !font-semibold">Choose Liquidity Shape</h2>
                    <div class="flex flex-wrap justify-evenly gap-y-2">
                        <button class="btn variant-ringed-surface card-hover p-2" class:selected="{shape === 0}" on:click={()=>shape=0}>
                            <div class="flex flex-col items-center">
                                <svg width="142" height="56" viewBox="0 0 142 56" fill="none" xmlns="http://www.w3.org/2000/svg">
                                    <defs>
                                        <linearGradient id="spot" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="0%" stop-color="#997db3" stop-opacity="{0}"></stop>
                                            <stop offset="100%" stop-color="#997db3" stop-opacity="{1}"></stop>
                                        </linearGradient>
                                    </defs>
                                    <rect opacity="0.4" x="26.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                    <rect opacity="0.4" x="36.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                    <rect opacity="0.4" x="46.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                    <rect x="56.5" y="8" width="9" height="40" rx="2" fill="url(#spot)"></rect>
                                    <rect x="66.5" y="8" width="9" height="40" rx="2" fill="url(#spot)"></rect>
                                    <rect x="76.5" y="8" width="9" height="40" rx="2" fill="url(#spot)"></rect>
                                    <rect opacity="0.4" x="86.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                    <rect opacity="0.4" x="96.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                    <rect opacity="0.4" x="106.5" y="40" width="9" height="8" rx="2" fill="url(#spot)"></rect>
                                </svg>
                                <p class="!text-xs font-heading-token font-bold">Spot</p>
                            </div>
                        </button>
                        <button class="btn variant-ringed-surface card-hover p-2" class:selected="{shape === 1}" on:click={()=>shape=1}>
                            <div class="flex flex-col items-center">
                                <svg width="142" height="56" viewBox="0 0 142 56" fill="none" xmlns="http://www.w3.org/2000/svg">
                                    <defs>
                                        <linearGradient id="normal" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="0%" stop-color="#997db3" stop-opacity="{0}"></stop>
                                            <stop offset="100%" stop-color="#997db3" stop-opacity="{1}"></stop>
                                        </linearGradient>
                                    </defs>
                                    <rect opacity="0.2" x="26.5" y="47" width="9" height="1" rx="0.5" fill="url(#normal)"></rect>
                                    <rect opacity="0.4" x="36.5" y="40" width="9" height="8" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.4" x="46.5" y="32" width="9" height="16" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.8" x="56.5" y="24" width="9" height="24" rx="2" fill="url(#normal)"></rect>
                                    <rect x="66.5" y="8" width="9" height="40" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.8" x="76.5" y="24" width="9" height="24" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.4" x="86.5" y="32" width="9" height="16" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.4" x="96.5" y="40" width="9" height="8" rx="2" fill="url(#normal)"></rect>
                                    <rect opacity="0.2" x="106.5" y="47" width="9" height="1" rx="0.5" fill="url(#normal)"></rect>
                                </svg>
                                <p class="!text-xs font-heading-token font-bold">Curve</p></div>
                        </button>
                        <button class="btn variant-ringed-surface card-hover p-2" class:selected="{shape === 2}" on:click={()=>shape=2}>
                            <div class="flex flex-col items-center">
                                <svg width="142" height="56" viewBox="0 0 142 56" fill="none" xmlns="http://www.w3.org/2000/svg">
                                    <defs>
                                        <linearGradient id="bidAsk" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="0%" stop-color="#997db3" stop-opacity="{0}"></stop>
                                            <stop offset="100%" stop-color="#997db3" stop-opacity="{1}"></stop>
                                        </linearGradient>
                                    </defs>
                                    <rect x="26.5" y="8" width="9" height="40" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.8" x="36.5" y="16" width="9" height="32" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.6" x="46.5" y="24" width="9" height="24" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.4" x="56.5" y="32" width="9" height="16" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.2" x="66.5" y="40" width="9" height="8" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.4" x="76.5" y="32" width="9" height="16" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.6" x="86.5" y="24" width="9" height="24" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect opacity="0.8" x="96.5" y="16" width="9" height="32" rx="2" fill="url(#bidAsk)"></rect>
                                    <rect x="106.5" y="8" width="9" height="40" rx="2" fill="url(#bidAsk)"></rect>
                                </svg>
                                <p class="!text-xs font-heading-token font-bold">Bid-Ask</p></div>
                        </button>
                    </div>
                </div>
                <div class="flex flex-nowrap flex-row justify-between items-center">
                    <h2 class="!text-xl !font-semibold">Price</h2>
                    <TabGroup 
                    justify="justify-center"
                    active="variant-ghost-secondary"
                    hover=""
                    flex="flex"
                    rounded=""
                    border="border-[1px] border-surface-500 p-1"
                    padding="py-1 px-4"
                    spacing="space-x-1"
                    class="font-heading-token font-bold"
                    >
                    <Tab bind:group={tabSetBy} name="add" value={0}>
                        By Range
                    </Tab>
                    <Tab bind:group={tabSetBy} name="remove" value={1}>
                        By Radius
                    </Tab>
                    </TabGroup>
                </div>
                {#if tabSetBy == 0}
                <div class="flex flex-col space-y-4">
                    <DoubleRangeSlider bind:start bind:end/>
                    <div class="grid grid-cols-[minmax(0,_1fr)_minmax(0,_1fr)_90px] gap-2">
                        <div class="flex flex-col justify-start space-y-2">
                            <p>Min Price:</p>
                            <div class="input-group input-group-divider grid-cols-[1fr_105px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                                <input type="number" name="" id="" placeholder="0.0" value="{nice( (1 - 100*(0.50 - start) * binStep / 10_000 ) * price )}" />
                                <div class="input-group-shim !px-2 !bg-surface-50-900-token font-normal text-xs !justify-center">SILK per sSCRT</div>
                            </div>
                        </div>
                        <div class="flex flex-col justify-start space-y-2">
                            <p>Max Price:</p>
                            <div class="input-group input-group-divider grid-cols-[1fr_105px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                                <input type="number" name="" id="" placeholder="0.0" value="{nice( (1 + 100*(end - 0.50) * binStep / 10_000 ) * price )}" />
                                <div class="input-group-shim !px-2 !bg-surface-50-900-token font-normal text-xs !justify-center">SILK per sSCRT</div>
                            </div>
                        </div>
                        <div class="flex flex-col justify-start space-y-2">
                            <p>Num Bins:</p>
                            <input type="number" name="" id="" placeholder="11" min=0 max=100 value="{(100 * (end - start)).toFixed(0)}" class="input !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold" />
                        </div>
                    </div>
                    <button on:click={()=>reset()} class="btn hover:variant-soft-primary dark:text-primary-400 text-primary-500 inline-flex items-center content-center self-end">
                        <svg width="16" height="15" viewBox="0 0 16 15" fill="none" xmlns="http://www.w3.org/2000/svg" focusable="false" class="dark:fill-primary-400 fill-primary-500 mr-2" aria-hidden="true">
                            <path fill-rule="evenodd" clip-rule="evenodd" d="M4.58818 2.62272C5.53752 2.09927 6.63128 1.89854 7.70463 2.05078C8.77798 2.20303 9.77277 2.7 10.5391 3.46681C10.5462 3.47397 10.5535 3.48101 10.5609 3.48794L11.8125 4.66663H10.3331C9.78085 4.66663 9.33313 5.11434 9.33313 5.66663C9.33313 6.21891 9.78085 6.66663 10.3331 6.66663H14.3331C14.4902 6.66663 14.6389 6.63039 14.7712 6.56583C14.8523 6.52634 14.929 6.4753 14.9989 6.41278C15.0585 6.35961 15.1116 6.29943 15.157 6.23352C15.2681 6.07241 15.3331 5.87711 15.3331 5.66663V1.66663C15.3331 1.11434 14.8854 0.666627 14.3331 0.666627C13.7808 0.666627 13.3331 1.11434 13.3331 1.66663V3.35135L11.9426 2.04191C10.8713 0.974696 9.48299 0.283008 7.9855 0.0706017C6.48281 -0.142543 4.95155 0.138479 3.62247 0.871321C2.29339 1.60416 1.2385 2.74912 0.616758 4.13365C-0.00498575 5.51819 -0.159896 7.0673 0.175372 8.54753C0.510639 10.0278 1.31792 11.3589 2.47556 12.3405C3.63321 13.322 5.0785 13.9006 6.59364 13.9893C8.10878 14.0779 9.61168 13.6717 10.8759 12.8319C11.7808 12.2307 12.5283 11.4307 13.0655 10.4997C13.3415 10.0214 13.1775 9.40982 12.6991 9.13379C12.2208 8.85777 11.6092 9.02179 11.3332 9.50015C10.9495 10.1652 10.4155 10.7366 9.76918 11.166C8.86619 11.7659 7.79268 12.056 6.71044 11.9927C5.6282 11.9294 4.59585 11.516 3.76896 10.815C2.94207 10.1139 2.36544 9.16305 2.12596 8.10573C1.88649 7.04842 1.99714 5.94192 2.44124 4.95296C2.88534 3.96401 3.63884 3.14618 4.58818 2.62272Z" fill="current"></path>
                        </svg>
                        Reset price
                    </button>
                </div>
                {:else if tabSetBy == 1}
                <div class="grid grid-cols-2 gap-2">
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Target Price:</p>
                        <div class="input-group input-group-divider grid-cols-[minmax(0,_1fr)_105px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                            <input type="number" name="" id="" placeholder="0.0" value="{nice(price)}" />
                            <div class="input-group-shim !px-2 !bg-surface-50-900-token font-normal text-xs !justify-center">SILK per sSCRT</div>
                        </div>
                    </div>
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Radius (number of bins):</p>
                        <input bind:value={binRadius} class="input !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold" type="number" name="" id="" />
                    </div>
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Range Min:</p>
                        <div class="input-group opacity-60 input-group-divider grid-cols-[minmax(0,_1fr)_105px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                            <input disabled type="number" name="" id="" placeholder="0.0" value={ nice(price * (1 - binRadius * binStep / 10_000)) } class="hover:cursor-not-allowed"/>
                            <div class="input-group-shim !px-2 !bg-surface-50-900-token font-normal text-xs !justify-center">SILK per sSCRT</div>
                        </div>
                    </div>
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Range Max:</p>
                        <div class="input-group opacity-60 input-group-divider grid-cols-[minmax(0,_1fr)_105px] !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold">
                            <input disabled type="number" name="" id="" placeholder="0.0" value={ nice(price * (1 + binRadius * binStep / 10_000)) } class="hover:cursor-not-allowed"/>
                            <div class="input-group-shim !px-2 !bg-surface-50-900-token font-normal text-xs !justify-center">SILK per sSCRT</div>
                        </div>
                    </div>
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Num Bins:</p>
                        <input value={binRadius * 2} disabled class="input !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold" type="number" name="" id="" />
                    </div>
                    <div class="flex flex-col justify-start space-y-2">
                        <p>Pct Range:</p>
                        <input value="{(binRadius * 2 * binStep / 100)}" disabled class="input !bg-surface-50-900-token font-heading-token text-secondary-600-300-token font-bold" type="number" name="" id="" />
                    </div>
                </div>
                {/if}

                {#if !$isAccountAvailable}
                    <button on:click={()=>setupKeplr()} class="btn w-full variant-ghost-primary mt-4 font-heading-token font-bold">
                        Connect Wallet
                    </button>
                {:else}
                    <button on:click={()=>addLiquidity()} class="btn w-full variant-ghost-secondary mt-4 font-heading-token font-bold">
                        Add Liquidity
                    </button>
                {/if}
            </div>
            {:else if tabSet === 1}
            <div in:fade={{duration: 200}}>
                <div class="space-y-2">
                    <h2 class="!text-xl !font-semibold">Not finished yet 😓</h2>
                    <p class="font-normal">
                        But you can still click the button to do an example transaction! <br/>
                        This will remove a very small amount of liquidity.
                    </p>
                </div>
                {#if !$isAccountAvailable}
                    <button on:click={()=>setupKeplr()} class="btn w-full variant-ghost-primary mt-4 font-heading-token font-bold">
                        Connect Wallet
                    </button>
                {:else}
                    <button on:click={()=>removeLiquidity()} class="btn w-full variant-ghost-secondary mt-4 font-heading-token font-bold">
                        Remove Liquidity
                    </button>
                {/if}
            </div>
            {/if}
        </svelte:fragment>
        </TabGroup>
    </div>
</div>

<style lang="postcss">
	.selected {
		@apply variant-ghost-primary;
	}
</style>