# lb-deployer

A simple utility to automate deploying the Liquitidy Book contracts.

Set things in `src/constants.rs`.

Run [`localsecret`](https://github.com/scrtlabs/LocalSecret/pkgs/container/localsecret) in a separate terminal.

```sh
docker run -it \
	-p 1317:1317 -p 5000:5000 -p 9090:9090 -p 26657:26657 \
	--name localsecret ghcr.io/scrtlabs/localsecret:v1.15.0
```

or

```sh
docker run -it -e FAST_BLOCKS=true \
	-p 1317:1317 -p 5000:5000 -p 9090:9090 -p 26657:26657 \
	--name localsecret ghcr.io/scrtlabs/localsecret:v1.15.0
```

- 1317: LCD/REST API
- 5000: faucet
- 9090: gRPC
- 26657: RPC

Run with cargo:

```sh
cargo run
```

A file named `lb_contracts.json` will appear in the root directory, for use with the frontend.
