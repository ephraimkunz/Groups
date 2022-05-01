# Build wasm.
cd groups_core
./build.sh

# Copy it to the static file server directory.
cd ../
rm -r groups_server/static/pkg
cp -r groups_core/pkg groups_server/static/

# Remove the .gitignore file, which if present prevents the deploy script from packaging that directory for Shuttle.
rm groups_server/static/pkg/.gitignore

# Deploy server to Shuttle.
cd groups_server
./deploy.sh
cd ../