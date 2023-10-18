import type { Key } from '@keplr-wallet/types'
import { SecretNetworkClient } from 'secretjs'
import { chains } from './config'
import { type Writable, writable } from 'svelte/store'
import type { SecretAddress, Token } from './tokens'

const SECRET_CHAIN_ID = chains['Secret Network'].chain_id
const SECRET_LCD = chains['Secret Network'].lcd

export function resetStores() {
	isAccountAvailable.set(false)
	keplrKey.set({
		name: "",
		algo: "",
		pubKey: new Uint8Array,
		address: new Uint8Array,
		bech32Address: "",
		isNanoLedger: false,
		isKeystone: false
	})
	secretClient.set(new SecretNetworkClient({
		url: SECRET_LCD,
		chainId: SECRET_CHAIN_ID
	}))
	secretAddress.set("secret1...")
	scrtBalance.set("🤫")
	amberBalance.set("🤫")
	viewingKeys.set(new Map<SecretAddress, string>)
	balances.set(new Map<SecretAddress, number>)
	console.log("stores reset")
}

export const isAccountAvailable: Writable<boolean> = writable(false)
export const keplrKey: Writable<Key> = writable({
    name: "",
    algo: "",
    pubKey: new Uint8Array,
    address: new Uint8Array,
    bech32Address: "",
    isNanoLedger: false,
    isKeystone: false
})
export const secretClient: Writable<SecretNetworkClient> = writable(
    new SecretNetworkClient({
        url: SECRET_LCD,
        chainId: SECRET_CHAIN_ID
}))

export const secretAddress: Writable<string> = writable("secret1...")
export const scrtBalance: Writable<string> = writable('🤫')
export const amberBalance: Writable<string> = writable('🤫')

export const tokenStore: Writable<Map<SecretAddress, Token>> = writable(new Map<SecretAddress, Token>)
export const viewingKeys: Writable<Map<SecretAddress, string>> = writable(new Map<SecretAddress, string>)
export const balances: Writable<Map<SecretAddress, number>> = writable(new Map<SecretAddress, number>)

export const apiUrl: Writable<string> = writable(SECRET_LCD)