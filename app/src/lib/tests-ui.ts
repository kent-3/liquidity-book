import { chains } from './config'
import { toHex, SecretNetworkClient } from 'secretjs';
// import { keccak256 } from '@cosmjs/crypto';
import { toastStore, type ToastSettings, modalStore, type ModalSettings } from '@skeletonlabs/skeleton';

const SECRET_CHAIN_ID = chains['Secret Network'].chain_id
const SECRET_LCD = chains['Secret Network'].lcd
const secretjs = new SecretNetworkClient({
    url: "https://lcd.secret.express",
    chainId: "secret-4"
});

function toHexString(byteArray: Uint8Array): string {
	return Array.from(byteArray, (byte) => ("0" + (byte & 0xff).toString(16)).slice(-2)).join("");
}

export function testModal() {
    const action = "Set Viewing Key"
	const buf = new Uint8Array(32);
	const key = toHexString(window.crypto.getRandomValues(buf))
	const padding = "one amber club"

	const sender = "secret1e4u8f8exq54n5tsfu4yh40t02n0wv0tyq7x5m8"
	const contract_address = "secret1s09x2xvfd2lp2skgzm29w2xtena7s8fq98v852"
	const msg = {
		set_viewing_key: {
			key,
			padding,
		},
        create_viewing_key: {
			key,
			padding,
		},
        something_else: {
			key,
			padding,
		}
	}
	const gasLimit = 50_000
	const gasPriceInFeeDenom = 0.0125
	const feeDenom = "uscrt"
	
	// TODO add code highlighting
	const confirm: ModalSettings = {
		type: 'confirm',
		title: 'Review Message Details (test)',
		body: `
			<dl class="font-mono grid grid-cols-[6rem_minmax(0,_2fr)]">
                <dt class="dark:text-surface-400">Action:</dt>
				<dd class="overflow-x-auto">${action}</dd>
				<dt class="dark:text-surface-400">Sender:</dt>
				<dd class="overflow-x-auto">${sender}</dd>
				<dt class="dark:text-surface-400">Contract:</dt>
				<dd class="overflow-x-auto">${contract_address}</dd>
				<dt class="dark:text-surface-400">Message:</dt>
				<pre class="col-span-full mb-2 !text-xs !text-primary-500 !bg-surface-900 !whitespace-pre !rounded-xl">${JSON.stringify(msg,null,2)}</pre>
				<dt class="dark:text-surface-400">Gas Limit:</dt>
				<dd>${gasLimit.toLocaleString()}</dd>
				<dt class="dark:text-surface-400">Gas Fee:</dt>
				<dd>
					<span class="font-bold text-tertiary-600">${gasLimit * 0.0125}</span> /
					<span class="font-bold text-secondary-400">${gasLimit * 0.1}</span> /
					<span class="font-bold text-primary-500">${gasLimit * 0.25}</span> uscrt
				</dd>
			</dl>
		`,
		modalClasses: 'w-auto',
		// TRUE if confirm pressed, FALSE if cancel pressed
		// TODO have this call the actual message signing function, passing a properties object
		response: (r: boolean) => {if (!r) {return}},
		// Optionally override the button text
		buttonTextCancel: 'Cancel',
		buttonTextConfirm: 'Confirm',
	};
	modalStore.trigger(confirm);
}

export function testToasts() {
    const t: ToastSettings = {
        message: `
        <h4>Transaction Success 🥳</h4>
        <details class="text-sm">
            <summary>Details</summary>
            <dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
                <dt class="opacity-50">Tx Hash:</dt>
                <a
                    href="https://www.mintscan.io/secret/txs/"
                    target="_blank"
                    rel="noreferrer"
                >
                    <dd>View on block explorer</dd>
                </a>
                <dt class="opacity-50">Fee:</dt>
                <dd>0.000625 SCRT</dd>
                <dt class="opacity-50">Gas Used:</dt>
                <dd>35,458</dd>
            </dl>
            </details>
        `,
        classes: '-translate-y-4 font-semibold variant-glass-success',
        // background: 'variant-glass-surface !bg-success-900 !bg-opacity-50 sm:!bg-opacity-30 ring-2 ring-inset ring-success-800',
        autohide: false,
    };
    toastStore.trigger(t)

    const t2: ToastSettings = {
        message: `
            <h4>Something went wrong 🤔</h4>
            <details class="text-sm">
                <summary>Details</summary>
                <dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
                    <dt class="opacity-50">Action:</dt>
                    <dd>Create Viewing Key</dd>
                    <dt class="opacity-50">Message:</dt>
                    <dd class="text-error-500">No wallet connected</dd>
                </dl>
            </details>
        `,
        background: 'variant-glass',
        autohide: false,
        classes: '-translate-y-4 font-semibold'
    };
    toastStore.trigger(t2)

    const t3: ToastSettings = {
        message: `
            <h4>Transaction Failed 😩</h4>
            <details class="text-sm">
                <summary>Details</summary>
                <dl class="font-mono grid grid-cols-[5rem_minmax(0,_2fr)]">
                    <dt class="opacity-50">Raw Log:</dt>
                    <dd class="text-error-500">MetaMask Message Signature: User denied message signature.</dd>
                </dl>
            </details>
            `,
        background: 'variant-glass-error',
        autohide: false,
        classes: '-translate-y-4 font-semibold'
    };
    toastStore.trigger(t3)
}
