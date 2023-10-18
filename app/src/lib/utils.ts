import { tokenList } from './tokens'
import { coinConvert } from './griptape';

/**
 * Convert a price to the underlying binId.
 *
 * @param price - Price of the bin.
 * @param binStep - BinStep of the pair.
 * @return BinId of the underlying bin.
 */
export function getIdFromPrice(price: number, binStep: number): number {
    return (
        Math.trunc(Math.log(price) / Math.log(1 + binStep / 10_000)) + 8388608
    )
}

export function compactAddress(longAddress: string): string {
	const shortAddress = longAddress.substring(0, 6) + '...' + longAddress.substring(39)
	return shortAddress
}

export function formatAmountforToken(address: string, amount: string): string {
    const token = tokenList.find(it => it.address === address);
    if (!token) throw new Error('No token found for address ' + address);
    return coinConvert(amount, token.decimals, 'machine');
}

export function getCodeHash(address: string){
    const token = tokenList.find(it => it.address === address);
    if (!token) throw new Error('No token found for address ' + address);
    return token.codeHash;
}

export function fromLiteraltoEpoch(value: string){
    return Date.parse(value) / 1000;
}
