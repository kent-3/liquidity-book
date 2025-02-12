# lb_router

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "factory": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}'
```

## Execute Messages

### create_lb_pair

```sh
secretcli tx compute execute secret1foobar '{
  "create_lb_pair": {
    "token_x": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    },
    "token_y": {
      "custom_token": {
        "contract_addr": "secret1...foobar",
        "token_code_hash": "0123456789ABCDEF"
      }
    },
    "active_id": 8388608,
    "bin_step": 100
  }
}'
```

### add_liquidity

```sh
secretcli tx compute execute secret1foobar '{
  "add_liquidity": {
    "liquidity_parameters": {
      "token_x": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "token_y": {
        "custom_token": {
          "contract_addr": "secret1...foobar",
          "token_code_hash": "0123456789ABCDEF"
        }
      },
      "bin_step": 100,
      "amount_x": "1000000",
      "amount_y": "1000000",
      "amount_x_min": "950000",
      "amount_y_min": "950000",
      "active_id_desired": 8388608,
      "id_slippage": 10,
      "delta_ids": [
        -5,
        -4,
        -3,
        -2,
        -1,
        0,
        1,
        2,
        3,
        4,
        5
      ],
      "distribution_x": [
        "0",
        "0",
        "0",
        "0",
        "0",
        "90909000000000000",
        "181818000000000000",
        "181818000000000000",
        "181818000000000000",
        "181818000000000000",
        "181818000000000000"
      ],
      "distribution_y": [
        "181818000000000000",
        "181818000000000000",
        "181818000000000000",
        "181818000000000000",
        "181818000000000000",
        "90909000000000000",
        "0",
        "0",
        "0",
        "0",
        "0"
      ],
      "to": "secret1...recipient",
      "refund_to": "secret1...sender",
      "deadline": "1739306006"
    }
  }
}'
```

### swap_exact_tokens_for_tokens

```sh
secretcli tx compute execute secret1foobar '{
  "swap_exact_tokens_for_tokens": {
    "amount_in": "1000000",
    "amount_out_min": "950000",
    "path": {
      "pair_bin_steps": [
        100
      ],
      "versions": [
        "v2_2"
      ],
      "token_path": [
        {
          "custom_token": {
            "contract_addr": "secret1...foobar",
            "token_code_hash": "0123456789ABCDEF"
          }
        },
        {
          "custom_token": {
            "contract_addr": "secret1...foobar",
            "token_code_hash": "0123456789ABCDEF"
          }
        }
      ]
    },
    "to": "secret1...sender",
    "deadline": "1739317404"
  }
}'
```

## Query Messages with responses

### get_factory

```sh
secretcli query compute query secret1foobar '{
  "get_factory": {}
}'
```

#### Response

```json
{
  "factory": "secret1...foobar"
}
```

### get_id_from_price

```sh
secretcli query compute query secret1foobar '{
  "get_id_from_price": {
    "lb_pair": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "price": "1000000000000000000"
  }
}'
```

#### Response

```json
{
  "id": 8388608
}
```

### get_price_from_id

```sh
secretcli query compute query secret1foobar '{
  "get_price_from_id": {
    "lb_pair": {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    },
    "id": 8388608
  }
}'
```

#### Response

```json
{
  "price": "1000000000000000000"
}
```

