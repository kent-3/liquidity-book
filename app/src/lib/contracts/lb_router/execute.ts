import type { SecretNetworkClient } from "secretjs";
import type * as LBFactory from "../lb_factory/types"
import { LB_ROUTER } from "$lib/contracts"
import { toastStore, type ToastSettings } from '@skeletonlabs/skeleton';

export async function executeCreateLBPair(
  client: SecretNetworkClient,
  contractHashTokenA: string,
  contractAddressTokenA: string,
  contractHashTokenB: string,
  contractAddressTokenB: string,
  active_id: number,  // 8388607 is the middle bin
  bin_step: number,   // 100 represents a 1% bin step
) {
  const msg: LBFactory.CreateLBPairMsg = {
    create_lb_pair: {
      token_x: {
        custom_token: {
          contract_addr: contractAddressTokenA,
          token_code_hash: contractHashTokenA,
        }
      },
      token_y: {
        custom_token: {
          contract_addr: contractAddressTokenB,
          token_code_hash: contractHashTokenB,
        }
      },
      active_id: active_id,
      bin_step: bin_step,
    }
  }

  try {
    const tx = await client.tx.compute.executeContract(
      {
        sender: client.address,
        contract_address: LB_ROUTER.address,
        code_hash: LB_ROUTER.code_hash,
        msg: msg,
        sent_funds: [],
      },
      {
        gasLimit: 500_000,
      }
    );

    // TODO move all this toast logic somewhere else and make it reusable
    if (tx.code === 0) {
      const t: ToastSettings = {
        message: `
        <h4>Transaction Success 🥳</h4>
        <details class="text-sm">
          <summary>Details</summary>
          <dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
            <dt class="opacity-50">Tx Hash:</dt>
            <a
              href="https://www.mintscan.io/secret/txs/${tx.transactionHash}"
              target="_blank"
              rel="noreferrer"
              class="!dark:text-success-500 !text-success-800"
            >
              <dd>View on block explorer</dd>
            </a>
            <dt class="opacity-50">Gas Used:</dt>
            <dd>${tx.gasUsed.toLocaleString()}</dd>
          </dl>
          </details>
        `,
        background: 'variant-glass-surface ring-2 ring-inset dark:ring-success-700 ring-success-700',
        autohide: false,
        classes: '-translate-y-4 font-semibold',
      };
      toastStore.trigger(t)
    } else {
      console.log(tx.rawLog)
			const t: ToastSettings = {
				message: `
				<h4>Transaction Failed</h4>
				<details class="text-sm">
					<summary>Details</summary>
					<dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
						<dt class="opacity-50">Raw Log:</dt>
						<dd>${tx.rawLog}</dd>
					</dl>
				</details>
				`,
				background: 'variant-glass-surface ring-2 ring-inset ring-error-500',
				autohide: false,
				classes: '-translate-y-4 font-semibold',
			};
			toastStore.trigger(t)
    }
  } catch (error) {
    const t: ToastSettings = {
			message: `
			<h4>Something went wrong 🤔</h4>
			<details class="text-sm">
				<summary>Details</summary>
				<dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
					<dt class="opacity-50">Action:</dt>
					<dd>Execute</dd>
					<dt class="opacity-50">Message:</dt>
					<dd class="text-error-700 dark:text-error-500">${error.message}</dd>
				</dl>
			</details>
			`,
			background: 'variant-glass-surface ring-2 ring-inset ring-secondary-500',
			autohide: false,
			classes: '-translate-y-4 font-semibold',
		};
		toastStore.trigger(t)
  }
}
