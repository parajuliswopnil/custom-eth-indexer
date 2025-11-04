# Custom indexer

## start reth with 
```sh
reth \
    node \
    --http \
    --ws \
    --datadir /tmp/twine \
    --rpc.eth-proof-window 1000 \
    --rpc.proof-permits 1000 --authrpc.jwtsecret jwt.hex --chain genesis.json
```

## start the sequencer with 
```sh
cargo run --release
```
