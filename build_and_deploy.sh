./package_wasm.sh

echo "Starting compiling server..."
cargo build --release --bin groups_server
echo "Finished compiling server"

echo "Running server..."
./target/release/groups_server
