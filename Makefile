.PHONY: schema test clippy build fmt compile check_contracts

schema:
	@find contracts/* -maxdepth 2 -type f -name Cargo.toml -execdir cargo schema \;
test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

compile:
	@./build_release.sh

check_contracts:
	@cargo install cosmwasm-check
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron,cosmwasm_1_1 artifacts/*.wasm

build: schema clippy test fmt compile check_contracts

