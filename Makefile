.PHONY: build test optimize clean deploy

build:
	cd contracts/soroban-wave-stake && cargo build --target wasm32-unknown-unknown --release

test:
	cd contracts/soroban-wave-stake && cargo test

optimize: build
	stellar contract optimize --wasm contracts/soroban-wave-stake/target/wasm32-unknown-unknown/release/soroban_wave_stake.wasm

clean:
	cd contracts/soroban-wave-stake && cargo clean
	rm -rf contracts/soroban-wave-stake/target
