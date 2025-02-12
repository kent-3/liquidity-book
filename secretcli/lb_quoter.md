# lb_quoter

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "factory_v2_2": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "router_v2_2": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}'
```

## Query Messages with responses

### get_factory

```sh
secretcli query compute query secret1foobar '{
  "get_factory_v2_2": {}
}'
```

#### Response

```json
{
  "factory_v2_2": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_router

```sh
secretcli query compute query secret1foobar '{
  "get_router_v2_2": {}
}'
```

#### Response

```json
{
  "router_v2_2": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}
```

### find_best_path_from_amount_in

```sh
secretcli query compute query secret1foobar '{
  "find_best_path_from_amount_in": {
    "route": [
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
    ],
    "amount_in": "1000000"
  }
}'
```

#### Response

```json
{
  "route": [
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
  ],
  "pairs": [
    {
      "address": "secret1...foobar",
      "code_hash": "0123456789ABCDEF"
    }
  ],
  "bin_steps": [
    100
  ],
  "versions": [
    "v2_2"
  ],
  "amounts": [
    "1000000",
    "980000"
  ],
  "virtual_amounts_without_slippage": [
    "1000000",
    "999000"
  ],
  "fees": [
    "1000000000000000"
  ]
}
```

