if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found, installing"
    cargo install wasm-pack
fi

wasm-pack build --profiling --target web