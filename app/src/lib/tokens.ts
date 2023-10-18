export type SecretAddress = `secret1${string}`

export interface Token {
	address: SecretAddress
	codeHash: string
	name: string
	symbol: string
	logo: string
	decimals: number
}

export const tokenList: Array<Token> = [
	// token X for testing
	{
		address: 'secret1jm4t9at03y2ffxwqlek9la58m90y7wvr8fvt5x',
		codeHash: '0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159',
		name: 'token x',
		symbol: 'TOKENX',
		logo: '',
		decimals: 6,
	},
	// token Y for testing (this is the quote asset that needs to be whitelisted)
	{
		address: 'secret108htfk68emyj9ylz5rl9ras36n9usaut0hk795',
		codeHash: '0bbaa17a6bd4533f5dc3eae14bfd1152891edaabcc0d767f611bb70437b3a159',
		name: 'token y',
		symbol: 'TOKENY',
		logo: '',
		decimals: 6,
	},
	{
		address: 'secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg',
		codeHash: '9587d60b8e6b078ace12014ceeee089530b9fabcd76535d93666a6c127ad8813',
		name: 'Secret SCRT',
		symbol: 'sSCRT',
		logo: '/scrt.svg',
		decimals: 6,
	},
	{
		address: 'secret1gdhgaeq9jvzwyjqc32j7cp00gd6cqpnkmncxd3',
		codeHash: 'db93ffb6ee9d5b924bc8f70e30c73ed809d210bca9b8aaab14eea609b55de166',
		name: 'Amber',
		symbol: 'AMBER',
		logo: '/amber.svg',
		decimals: 6,
		
	},
	{
		address: 'secret1yw3dmyk3zw5v0pvhdtgu7wky6hcxvgf5q8t298',
		codeHash: '5266a630e2b8ef910fb2515e1d3b5be95d4bd48358732788d8fcd984ee966bc1',
		name: 'Shade',
		symbol: 'SHD',
		logo: '/shade.svg',
		decimals: 8,
	},
	{
		address: 'secret10u3rwj0cc2r04lryaxtkucjhvqw63kqzm5jlxw',
		codeHash: '680fbb3c8f8eb1c920da13d857daaedaa46ab8f9a8e26e892bb18a16985ec29e',
		name: 'SCRT Staking Derivatives',
		symbol: 'stkd-SCRT',
		logo: '/stkd-scrt.svg',
		decimals: 6,	
	},
	{
		address: 'secret16xz08fdtkp5m8m6arpfgnehlfl4t86l0p33xg0',
		codeHash: 'b6c896d21e46e037a2a1bca1d55af262d7ae4a5a175af055f3939722626b30c3',
		name: 'Silk',
		symbol: 'SILK',
		logo: '',
		decimals: 6,	
	}
]
