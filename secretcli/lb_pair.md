# lb_pair

## Instantiate Message

```sh
secretcli tx compute instantiate 1 '{
  "factory": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
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
  "pair_parameters": {
    "base_factor": 5000,
    "filter_period": 30,
    "decay_period": 600,
    "reduction_factor": 5000,
    "variable_fee_control": 40000,
    "protocol_share": 1000,
    "max_volatility_accumulator": 350000
  },
  "active_id": 8388608,
  "lb_token_implementation": {
    "id": 0,
    "code_hash": ""
  },
  "viewing_key": "viewing_key",
  "entropy": "entropy",
  "admin_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  },
  "query_auth": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}'
```

## Execute Messages

### swap_tokens

```sh
secretcli tx compute execute secret1foobar '{
  "swap": {
    "swap_for_y": true,
    "to": "secret1...recipient"
  }
}'
```

### swap_tokens_invoke

```sh
secretcli tx compute execute secret1foobar '{
  "swap": {
    "swap_for_y": true,
    "to": "secret1...recipient"
  }
}'
```

### collect_protocol_fees

```sh
secretcli tx compute execute secret1foobar '{
  "collect_protocol_fees": {}
}'
```

### set_static_fee_parameters

```sh
secretcli tx compute execute secret1foobar '{
  "set_static_fee_parameters": {
    "base_factor": 5000,
    "filter_period": 30,
    "decay_period": 600,
    "reduction_factor": 5000,
    "variable_fee_control": 40000,
    "protocol_share": 1000,
    "max_volatility_accumulator": 350000
  }
}'
```

### force_decay

```sh
secretcli tx compute execute secret1foobar '{
  "force_decay": {}
}'
```

### set_contract_status

```sh
secretcli tx compute execute secret1foobar '{
  "set_contract_status": {
    "contract_status": "freeze_all"
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

### get_token_x

```sh
secretcli query compute query secret1foobar '{
  "get_token_x": {}
}'
```

#### Response

```json
{
  "token_x": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_token_y

```sh
secretcli query compute query secret1foobar '{
  "get_token_y": {}
}'
```

#### Response

```json
{
  "token_y": {
    "custom_token": {
      "contract_addr": "secret1...foobar",
      "token_code_hash": "0123456789ABCDEF"
    }
  }
}
```

### get_bin_step

```sh
secretcli query compute query secret1foobar '{
  "get_bin_step": {}
}'
```

#### Response

```json
{
  "bin_step": 100
}
```

### get_reserves

```sh
secretcli query compute query secret1foobar '{
  "get_reserves": {}
}'
```

#### Response

```json
{
  "reserve_x": "1000",
  "reserve_y": "1000"
}
```

### get_active_id

```sh
secretcli query compute query secret1foobar '{
  "get_active_id": {}
}'
```

#### Response

```json
{
  "active_id": 8388608
}
```

### get_bin

```sh
secretcli query compute query secret1foobar '{
  "get_bin": {
    "id": 8388608
  }
}'
```

#### Response

```json
{
  "bin_id": 8388608,
  "bin_reserve_x": "1000",
  "bin_reserve_y": "1000"
}
```

### get_next_non_empty_bin

```sh
secretcli query compute query secret1foobar '{
  "get_next_non_empty_bin": {
    "swap_for_y": true,
    "id": 1
  }
}'
```

#### Response

```json
{
  "next_id": 8388609
}
```

### get_protocol_fees

```sh
secretcli query compute query secret1foobar '{
  "get_protocol_fees": {}
}'
```

#### Response

```json
{
  "protocol_fee_x": 1000,
  "protocol_fee_y": 1000
}
```

### get_static_fee_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_static_fee_parameters": {}
}'
```

#### Response

```json
{
  "base_factor": 5000,
  "filter_period": 30,
  "decay_period": 600,
  "reduction_factor": 5000,
  "variable_fee_control": 40000,
  "protocol_share": 1000,
  "max_volatility_accumulator": 350000
}
```

### get_variable_fee_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_variable_fee_parameters": {}
}'
```

#### Response

```json
{
  "volatility_accumulator": 0,
  "volatility_reference": 0,
  "id_reference": 0,
  "time_of_last_update": 0
}
```

### get_oracle_parameters

```sh
secretcli query compute query secret1foobar '{
  "get_oracle_parameters": {}
}'
```

#### Response

```json
{
  "sample_lifetime": 120,
  "size": 10,
  "active_size": 10,
  "last_updated": 1703403384,
  "first_timestamp": 1703403383
}
```

### get_oracle_sample_at

```sh
secretcli query compute query secret1foobar '{
  "get_oracle_sample_at": {
    "lookup_timestamp": 12345
  }
}'
```

#### Response

```json
{
  "cumulative_id": 100,
  "cumulative_volatility": 200,
  "cumulative_bin_crossed": 50
}
```

### get_price_from_id

```sh
secretcli query compute query secret1foobar '{
  "get_price_from_id": {
    "id": 8388608
  }
}'
```

#### Response

```json
{
  "price": "42008768657166552252904831246223292524636112144"
}
```

### get_id_from_price

```sh
secretcli query compute query secret1foobar '{
  "get_id_from_price": {
    "price": "42008768657166552252904831246223292524636112144"
  }
}'
```

#### Response

```json
{
  "id": 8388608
}
```

### get_swap_in

```sh
secretcli query compute query secret1foobar '{
  "get_swap_in": {
    "amount_out": "100000",
    "swap_for_y": true
  }
}'
```

#### Response

```json
{
  "amount_in": "1000",
  "amount_out_left": "10",
  "fee": "10"
}
```

### get_swap_out

```sh
secretcli query compute query secret1foobar '{
  "get_swap_out": {
    "amount_in": "100000",
    "swap_for_y": true
  }
}'
```

#### Response

```json
{
  "amount_in_left": "1000",
  "amount_out": "10",
  "fee": "100"
}
```

### get_lb_token

```sh
secretcli query compute query secret1foobar '{
  "get_lb_token": {}
}'
```

#### Response

```json
{
  "lb_token": {
    "address": "secret1...foobar",
    "code_hash": "0123456789ABCDEF"
  }
}
```

### get_lb_token_supply

```sh
secretcli query compute query secret1foobar '{
  "get_lb_token_supply": {
    "id": 1
  }
}'
```

#### Response

```json
{
  "total_supply": "4200876899744891917384329470959789995640432360000000"
}
```

### get_bins

```sh
secretcli query compute query secret1foobar '{
  "get_bins": {
    "ids": [
      8388607,
      8388608,
      8388609
    ]
  }
}'
```

#### Response

```json
[
  {
    "bin_id": 8388607,
    "bin_reserve_x": "1000",
    "bin_reserve_y": "0"
  },
  {
    "bin_id": 8388608,
    "bin_reserve_x": "1000",
    "bin_reserve_y": "1000"
  },
  {
    "bin_id": 8388609,
    "bin_reserve_x": "0",
    "bin_reserve_y": "1000"
  }
]
```

### get_all_bins

```sh
secretcli query compute query secret1foobar '{
  "get_all_bins": {
    "id": null,
    "page": null,
    "page_size": null
  }
}'
```

#### Response

```json
{
  "reserves": [
    {
      "bin_id": 8388607,
      "bin_reserve_x": "1000",
      "bin_reserve_y": "0"
    },
    {
      "bin_id": 8388608,
      "bin_reserve_x": "1000",
      "bin_reserve_y": "1000"
    },
    {
      "bin_id": 8388609,
      "bin_reserve_x": "0",
      "bin_reserve_y": "1000"
    }
  ],
  "last_id": 8388609,
  "current_block_height": 123456
}
```

