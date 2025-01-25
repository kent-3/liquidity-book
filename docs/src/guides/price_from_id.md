# Get Price From Bin Id

Every bin id corresponds a specific price, which varies depending on the bin step. Here are some examples to get the price from a given `bin_id`.

### Conversion Functions

{{#tabs }}
{{#tab name="JavaScript" }}

```js
/**
* Convert a binId to the underlying price.
*
* @param binId - Bin Id.
* @param binStep - Bin step of the pair in basis points.
* @return Price of the bin.
*/
function getPriceFromId(binId: number, binStep: number): number {
  return (1 + binStep / 10_000) ** (binId - 8388608);
}
```

{{#endtab }}
{{#tab name="Python" }}

```py
def getPriceFromId(bin_id: int, bin_step: int) -> float:
    """
    Convert a bin_id to the underlying price.

    :param bin_id: Bin Id.
    :param bin_step: Bin step of the pair in basis points.
    :return: Price of the bin.
    """

    return (1 + binStep / 10_000) ** (binId - 8388608)
```

{{#endtab }}
{{#tab name="Rust" }}

```rust
// TODO: write a version using only the standard library

// /// Calculates the price as a 128.128-binary fixed-point number
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

Here is an example to illustrate the conversion function with the sSCRT/SCRT pair which has a binStep of 5. We choose here a bin_id equal to 8388755. Price returned doesn't need to be adjusted, as both tokens have 6 decimals.

```py
getPriceFromId(8388755, 5)
>>> 1.0762487670087693
```

For second example, let's take SHD/SCRT pair which has a binStep of 10. We choose bin_id equal to 8394314.

```py
getPriceFromId(8394314, 10)
>>> 299.80998797504674
```

<br>

\\[ priceAdjusted = price \cdot 10^{(decimalsX - decimalsY)} \\]

\\[ priceAdjusted = 299.80 \cdot 10^{(8 - 6)} = 29800 \\]
