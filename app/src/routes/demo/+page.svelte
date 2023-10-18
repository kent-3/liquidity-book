<script lang="ts">
  import { base } from "$app/paths";
  import { fly } from "svelte/transition";


</script>

<div in:fly="{{ x: 100, duration: 500 }}" class="container h-full mx-auto p-6 flex justify-center items-start first:pt-8">
    <div class="block xl:w-2/3 w-auto space-y-10">
      <h1 class="font-semibold">HackSecret Demo Instructions</h1>
      <div class="space-y-2">
        <h2>Create a Pool</h2>
        <p>
          Go to the <a href="{base}/pool" class="btn btn-sm px-1 py-0 variant-ghost-primary">Pool</a> page and click 
          <a href="{base}/pool/create" class="btn btn-sm px-1 py-0 variant-ghost-primary">Create New Pool</a>. 
          Check the console as you choose different options, 
          especially "Active Price" which is converted to the Active Bin ID.
          A pool has already been created using the default settings.
          Try changing the inputs to create a new pool.
          Currently, only Bin Step 1% has presets configured.
        </p>
      </div>
      <div class="space-y-2">
        <h2>Add/Remove Liquidity</h2>
        <p>
          Go to the
          <a href="{base}/pool" class="btn btn-sm px-1 py-0 variant-ghost-primary">
            Pool
          </a>
          page and click on the 
          <a href="{base}/pool/example" class="btn btn-sm px-1 py-0 variant-ghost-primary">
            Example Pool
          </a>
          link. Experiment with the price settings, but the button is preset to add 1,000,000,000 
          of TOKENX and TOKENY with a fixed liquidity configuration.
          See <code>/app/src/lib/contracts/lb_pair/execute.ts</code> for the full inputs.
        </p>
      </div>
      <div class="space-y-2">
        <h2>Make a Trade</h2>
        <p>
          Go to the
          <a href="{base}/trade" class="btn btn-sm px-1 py-0 variant-ghost-primary">
            Trade
          </a> page. The UI isn't fully ready yet, but the Swap button is preset to swap 1000 TOKENX for TOKENY.
        </p>
      </div>
    </div>
</div>