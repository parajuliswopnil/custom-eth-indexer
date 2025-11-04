#! /bin/bash
set -euo pipefail

RPC_URL="${RPC_URL:-http://127.0.0.1:8545}"
PK="${PK:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
ADDR="${ADDR:-0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266}"
COUNT="${COUNT:-2000}"
DEPOSIT_TO="${DEPOSIT_TO:-0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f}"
AMOUNT="${AMOUNT:-10}"

echo "RPC_URL=$RPC_URL"
echo "From ADDR=$ADDR"
echo "COUNT=$COUNT"

# Get starting nonce for ADDR
START_NONCE="$(cast nonce "$ADDR" --rpc-url "$RPC_URL")"
echo "[batch_deposit] Starting nonce: $START_NONCE"

# Fire COUNT transactions in parallel without waiting for receipts
for (( i=0; i<COUNT; i++ )); do
  NONCE=$(( START_NONCE + i ))
  (
    echo "[batch_deposit] Sending tx #$i (nonce=$NONCE)..."
    cast send $DEPOSIT_TO \
      --gas-limit 100000 \
      --value "$AMOUNT" \
      --rpc-url "$RPC_URL" \
      --private-key "$PK" \
      --nonce "$NONCE" \
      --async
  ) &
done

# Wait for all background sends to be issued
wait
echo "[batch_deposit] Dispatched $COUNT transactions."