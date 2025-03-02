# This builds and deploys the server in groups_server_local. To build the same server as the public website uses,
# use `cargo shuttle run ...`

./build_local.sh

# Run the local webserver for testing (provides random students)
# cargo run --release --bin groups_server_local 

# Run the local webserver version of the shuttle service.
cd groups_server
shuttle run