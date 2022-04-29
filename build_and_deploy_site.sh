# Build wasm.
cd groups-core
./build.sh

# Copy it to the static file server directory.
cd ../
cp -r groups-core/pkg groups-server/static/

# Remove the .gitignore file, which if present prevents the deploy script from packaging that directory for Shuttle.
rm groups-server/static/pkg/.gitignore

# Deploy server to Shuttle.
cd groups-server
./deploy.sh
cd ../