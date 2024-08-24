build target='native':
	cargo build --release --no-default-features --features {{target}}

test target='native':
	cargo test --release --no-default-features --features {{target}}

wasm-pack-on-change:
	watchexec --exts rs --restart wasm-pack build

build-and-test-on-change:
	watchexec --exts rs --restart "just build; just test"

