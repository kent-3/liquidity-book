# Get Price From Bin Id

Each bin holds the liquidity of the pair for a specific price range. Thus, it is possible to link a certain bin to a price by using the id of the underlying bin. We provide examples to get the price from a binId.

### Conversion Functions

In order to link a bin_id to a price it is necessary to know the bin_step of the underlying pair. Here is the conversion logic.

{{#tabs }}
{{#tab name="JavaScript" }}

```js
function getPriceFromId(binId: number, binStep: number): number {
  /**
   * Convert a binId to the underlying price.
   *
   * @param binId - Bin Id.
   * @param binStep - binStep of the pair.
   * @return Price of the bin.
   */

  return (1 + binStep / 10_000) ** (binId - 8388608);
}
```

{{#endtab }}
{{#tab name="Python" }}

```py
def getPriceFromId(binId: int, binStep: int) -> float:
    """
    Convert a binId to the underlying price.

    :param binId: Bin Id.
    :param binStep: BinStep of the pair.
    :return: Price of the bin.
    """

    return (1 + binStep / 10_000) ** (binId - 8388608)
```

{{#endtab }}
{{#tab name="Rust" }}

```rust
// TODO: write a version using only the standard library

/// Calculates the price as a 128.128-binary fixed-point number
// pub fn get_price_from_id(id: u32, bin_step: u16) -> Result<U256, U128x128MathError> {
//     let base = Self::get_base(bin_step);
//     let exponent = Self::get_exponent(id);
//
//     U128x128Math::pow(&base, exponent)
// }
```

{{#endtab }}
{{#endtabs }}

### Example

Here is an example to illustrate the conversion function with the sAVAX/AVAX pair which has a binStep of 5. We choose here a binId equal to 8388755. Price returned doesn't need to be adjusted, as both tokens have 18 decimals.

```py
getPriceFromId(8388755, 5)
>>> 1.0762487670087693
```

For second example, let's take BTC.b/USDC pair which has a binStep of 10. We choose binId equal to 8394314.

```py
getPriceFromId(8394314, 10)
>>> 299.80998797504674
```

<br>

\\[ priceAdjusted = price \cdot 10^{(decimalsX - decimalsY)} \\]

\\[ priceAdjusted = 299.80 \cdot 10^{(8 - 6)} = 29800 \\]
