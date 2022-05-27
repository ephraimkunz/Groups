if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found, installing"
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

wasm-pack build --profiling --target web