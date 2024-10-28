# Building the Contracts

The build target and rustflags are set in [`.cargo/config.toml`](.cargo/config.toml). To build the contracts, simply run:

```sh
cargo build --lib --release
```

After building, you can use the provided script to collect, optimize, and prepare the contract `.wasm` files for deployment:

```sh
bash move-wasm.sh
```
