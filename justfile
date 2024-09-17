export CFLAGS_wasm32_unknown_unknown := `echo "-I$(pwd)/wasm-sysroot -Wbad-function-cast -Wcast-function-type -fno-builtin"`

test:
	cargo test

build-native:
	cargo build --release

build-web:
	wasm-pack build --release

