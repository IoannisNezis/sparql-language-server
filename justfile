export CFLAGS_wasm32_unknown_unknown := `echo "-I$(pwd)/wasm-sysroot -Wbad-function-cast -Wcast-function-type -fno-builtin"`

test:
	cargo test --bin qlue-ls

start-monaco-editor:
	cd editor && npm install && npm run dev

build-native:
	cargo build --release --bin qlue-ls

build-wasm profile="release" target="bundler":
	notify-send -t 1000 "starting wasm build..."
	wasm-pack build --{{profile}} --target {{target}}
	notify-send -t 600 "build done"

watch-and-run recipe="test":
	watchexec --restart --exts rs --exts toml just {{recipe}}

publish:
	wasm-pack publish
	maturin publish
	cargo publish
