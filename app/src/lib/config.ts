export type Chain = {
	/** display name of the chain */
	chain_name: string
	/** channel_id on the chain */
	deposit_channel_id: string
	/** gas limit for ibc transfer from the chain to Secret Network */
	deposit_gas: number
	/** channel_id on Secret Network */
	withdraw_channel_id: string
	/** gas limit for ibc transfer from Secret Network to the chain */
	withdraw_gas: number
	/** bech32 prefix of addresses on the chain */
	bech32_prefix: string
	/** logo of the chain */
	chain_image: string
	/** chain-id of the chain */
	chain_id: string
	/** lcd url of the chain */
	lcd: string
	/** rpc url of the chain */
	rpc: string
	/** explorer link for accounts */
	explorer_account: string
	/** explorer link for txs */
	explorer_tx?: string
}

export const chains: { [chain_name: string]: Chain } = {
	'Secret Network': {
		chain_name: 'Secret Network',
		deposit_channel_id: '',
		deposit_gas: 0,
		withdraw_channel_id: '',
		withdraw_gas: 0,
		// chain_id: 'secret-4',
		chain_id: 'pulsar-2',
		bech32_prefix: 'secret',
		// lcd: 'https://lcd.secret.express',
		lcd: 'https://lcd.testnet.secretsaturn.net',
		// rpc: 'https://rpc.secret.express',
		rpc: 'https://rpc.testnet.secretsaturn.net',
		chain_image: '/scrt.svg',
		// explorer_account: 'https://www.mintscan.io/secret/account/'
		explorer_account: 'https://secretnodes.com/pulsar/accounts'
	},
	'Secret Testnet': {
		chain_name: 'Pulsar',
		deposit_channel_id: '',
		deposit_gas: 0,
		withdraw_channel_id: '',
		withdraw_gas: 0,
		chain_id: 'pulsar-2',
		bech32_prefix: 'secret',
		lcd: 'https://lcd.testnet.secretsaturn.net',
		rpc: 'https://rpc.testnet.secretsaturn.net',
		chain_image: '/scrt.svg',
		explorer_account: 'https://secretnodes.com/pulsar/accounts'
	}
}

