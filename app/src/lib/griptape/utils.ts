import { Decimal } from 'decimal.js';

// The number set here is an arbitrary number.
Decimal.set({ toExpPos: 50 });

// Convert to/from human and machine.
export function coinConvert(
  number: number | string,
  decimals: number,
  type?: 'human' | 'machine',
  fixed?: number
): string {
  if (!number) return '';

  let theNumber = number;
  if (typeof number === 'number') {
    theNumber = number.toString();
  }

  if ((theNumber as string).indexOf('.') === -1) {
    // In case `number` is an integer

    let result: Decimal;

    if (type && type === 'machine') {
      result = new Decimal(10).toPower(decimals).times(number);
    } else {
      result = new Decimal(number).dividedBy(new Decimal(10).toPower(decimals));
    }

    if (typeof fixed !== 'undefined') {
      return result.toFixed(fixed);
    }

    return result.toString();
  } else {
    // In case is not an integer, we just handle it as float

    let result: Decimal;

    if (type && type === 'human') {
      result = new Decimal(number);
    } else {
      result = new Decimal(10).toPower(decimals).times(number);
    }

    if (typeof fixed !== 'undefined') {
      return result.toFixed(fixed);
    }

    return result.toString();
  }
}

export function bech32(str: string, abbrv: number): string {
  if (!str) return '';
  const half = abbrv / 2 || 8;
  return (
    str.substring(0, half) +
    '...' +
    str.substring(str.length - half, str.length)
  );
}

export function getWindow() {
  return typeof window !== 'undefined' ? window : undefined;
}
