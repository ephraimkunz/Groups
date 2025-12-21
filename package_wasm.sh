# Build wasm.
cd groups_core
./build.sh

# Copy it to the static file server directory.
echo "Copying wasm pkg to server..."
cd ../
sudo rm -rf groups_server/static/pkg
cp -r groups_core/pkg groups_server/static/
echo "Finished copying wasm pkg to server"
