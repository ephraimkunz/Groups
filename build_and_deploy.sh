#!/usr/bin/env bash
set -e

./package_wasm.sh

echo "Starting compiling server..."
cargo build --release --bin groups_server
echo "Finished compiling server"

echo "Stopping existing Tailscale config (if any)..."
sudo tailscale funnel off || true
sudo tailscale serve off || true

echo "Starting server with nohup..."
nohup env \
  ROCKET_ADDRESS=0.0.0.0 \
  ROCKET_PORT=8000 \
  ./target/release/groups_server \
  > server.log 2>&1 &

SERVER_PID=$!
echo "Server started with PID $SERVER_PID"

echo "Waiting for server to be ready..."
for i in {1..20}; do
  if curl -sf http://127.0.0.1:8000 >/dev/null; then
    break
  fi
  sleep 0.5
done

echo "Configuring Tailscale serve + funnel..."
sudo tailscale serve --bg 8000
sudo tailscale funnel --bg localhost:8000

sudo tailscale serve status

echo "Deployment complete"
echo "Logs: tail -f server.log"
