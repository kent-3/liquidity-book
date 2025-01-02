# Building the contracts

```sh
RUSTFLAGS='-C link-arg=-s' cargo build --lib --release --target wasm32-unknown-unknown
```

After building, you can use the provided script to collect, optimize, and prepare the contract `.wasm` files for deployment:

```sh
bash move-wasm.sh
```

## Project aliases

Deploy contracts on-chain (see `lb-deployer` readme for more info):

```sh
cargo deploy
```

Generate JSON schema definitions to `./schema`:

```sh
cargo schema
```

Generate example `secretcli` commands and responses to `./secretcli`:

```sh
cargo secretcli
```
