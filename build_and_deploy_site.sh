# Build wasm.
cd groups-core
./build.sh

# Copy it to the static file server directory.
cd ../
cp -r groups-core/pkg groups-server/static/

# Deploy server to Shuttle.
cd groups-server
./deploy.sh
cd ../