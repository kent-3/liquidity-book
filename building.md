# Building the Contracts

```sh
RUSTFLAGS='-C link-arg=-s' cargo build --lib --release --target wasm32-unknown-unknown
```

After building, you can use the provided script to collect, optimize, and prepare the contract `.wasm` files for deployment:

```sh
bash move-wasm.sh
```
