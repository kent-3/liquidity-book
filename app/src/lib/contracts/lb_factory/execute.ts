import type { SecretNetworkClient } from "secretjs";
import type * as LBFactory from "./types"
import { LB_FACTORY, LB_PAIR_CODE, LB_TOKEN_CODE } from "$lib/contracts"
import { toastStore, type ToastSettings } from '@skeletonlabs/skeleton';


export async function executeSetLBPairImplementation(
  client: SecretNetworkClient,
) {
  const msg: LBFactory.SetLBPairImplementationMsg = {
    set_lb_pair_implementation: {
      lb_pair_implementation: {
          id: LB_PAIR_CODE.id,
          code_hash: LB_PAIR_CODE.hash,
      }
    }
  }

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: LB_FACTORY.address,
      code_hash: LB_FACTORY.code_hash,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200000,
    }
  );
  

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SetLBPairImplementation TX used ${tx.gasUsed} gas`);
}

export async function executeSetLBTokenImplementation(
  client: SecretNetworkClient,
) {
  const msg: LBFactory.SetLBTokenImplementationMsg = {
    set_lb_token_implementation: {
      lb_token_implementation: {
          id: LB_TOKEN_CODE.id,
          code_hash: LB_TOKEN_CODE.hash,
      }
    }
  }
  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: LB_FACTORY.address,
      code_hash: LB_FACTORY.code_hash,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SetLBTokenImplementation TX used ${tx.gasUsed} gas`);
}

// NOTE: this should be called from the Router

// export async function executeCreateLBPair(
//   client: SecretNetworkClient,
//   contractHashTokenA: string,
//   contractAddressTokenA: string,
//   contractHashTokenB: string,
//   contractAddressTokenB: string,
//   active_id: number,  // 8388607 is the middle bin
//   bin_step: number,   // 100 represents a 1% bin step
// ) {
//   const msg: LBFactory.CreateLBPairMsg = {
//     create_lb_pair: {
//       token_x: {
//         custom_token: {
//           contract_addr: contractAddressTokenA,
//           token_code_hash: contractHashTokenA,
//         }
//       },
//       token_y: {
//         custom_token: {
//           contract_addr: contractAddressTokenB,
//           token_code_hash: contractHashTokenB,
//         }
//       },
//       active_id: active_id,
//       bin_step: bin_step,
//     }
//   }

//   try {
//     const tx = await client.tx.compute.executeContract(
//       {
//         sender: client.address,
//         contract_address: LB_FACTORY.address,
//         code_hash: LB_FACTORY.code_hash,
//         msg: msg,
//         sent_funds: [],
//       },
//       {
//         gasLimit: 500000,
//       }
//     );

//     // TODO move all this toast logic somewhere else and make it reusable
//     if (tx.code === 0) {
//       const t: ToastSettings = {
//         message: `
//         <h4>Transaction Success 🥳</h4>
//         <details class="text-sm">
//           <summary>Details</summary>
//           <dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
//             <dt class="opacity-50">Tx Hash:</dt>
//             <a
//               href="https://www.mintscan.io/secret/txs/${tx.transactionHash}"
//               target="_blank"
//               rel="noreferrer"
//             >
//               <dd>View on block explorer</dd>
//             </a>
//             <dt class="opacity-50">Gas Used:</dt>
//             <dd>${tx.gasUsed.toLocaleString()}</dd>
//           </dl>
//           </details>
//         `,
//         background: 'variant-glass-surface !bg-success-900 !bg-opacity-50 sm:!bg-opacity-30 ring-2 ring-inset ring-success-800',
//         autohide: false,
//         classes: '-translate-y-4 font-semibold',
//       };
//       toastStore.trigger(t)
//     } else {
//       console.log(tx.rawLog)
// 			const t: ToastSettings = {
// 				message: `
// 				<h4>Transaction Failed</h4>
// 				<details class="text-sm">
// 					<summary>Details</summary>
// 					<dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
// 						<dt class="opacity-50">Raw Log:</dt>
// 						<dd>${tx.rawLog}</dd>
// 					</dl>
// 				</details>
// 				`,
// 				background: 'variant-glass-secondary ring-1 ring-inset ring-error-500',
// 				autohide: false,
// 				classes: '-translate-y-4 font-semibold',
// 			};
// 			toastStore.trigger(t)
//     }
//   } catch (error) {
//     const t: ToastSettings = {
// 			message: `
// 			<h4>Something went wrong 🤔</h4>
// 			<details class="text-sm">
// 				<summary>Details</summary>
// 				<dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
// 					<dt class="opacity-50">Action:</dt>
// 					<dd>Execute</dd>
// 					<dt class="opacity-50">Message:</dt>
// 					<dd class="text-error-500">${error.message}</dd>
// 				</dl>
// 			</details>
// 			`,
// 			background: 'variant-glass-secondary ring-1 ring-inset ring-secondary-500',
// 			autohide: false,
// 			classes: '-translate-y-4 font-semibold',
// 		};
// 		toastStore.trigger(t)
//   }
// }

export async function executeSetPreset(
  client: SecretNetworkClient,
  bin_step: number,
  base_factor: number,
  filter_period: number,
  decay_period: number,
  reduction_factor: number,
  variable_fee_control: number,
  protocol_share: number,
  max_volatility_accumulator: number,
  is_open: boolean,
) {
  const msg: LBFactory.SetPresetMsg = {
    set_preset: {
      // TODO: figure out approprate values to use
      bin_step,
      base_factor,
      filter_period,
      decay_period,
      reduction_factor,
      variable_fee_control,
      protocol_share,
      max_volatility_accumulator,
      is_open,
    }
  }

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: LB_FACTORY.address,
      code_hash: LB_FACTORY.code_hash,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`SetPreset TX used ${tx.gasUsed} gas`);
}

export async function executeAddQuoteAsset(
  client: SecretNetworkClient,
  contractHashQuoteAsset: string,
  contractAddressQuoteAsset: string,
) {
  const msg: LBFactory.AddQuoteAssetMsg = {
    add_quote_asset: {
      asset: {
        custom_token: {
          contract_addr: contractAddressQuoteAsset,
          token_code_hash: contractHashQuoteAsset,
        }
      }
    }
  }

  const tx = await client.tx.compute.executeContract(
    {
      sender: client.address,
      contract_address: LB_FACTORY.address,
      code_hash: LB_FACTORY.code_hash,
      msg: msg,
      sent_funds: [],
    },
    {
      gasLimit: 200000,
    }
  );

  if (tx.code !== 0) {
    throw new Error(
      `Failed with the following error:\n ${tx.rawLog}`
    );
  }

  //let parsedTransactionData = JSON.parse(fromUtf8(tx.data[0])); // In our case we don't really need to access transaction data
  console.log(`AddQuoteAsset TX used ${tx.gasUsed} gas`);
}
