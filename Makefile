.PHONY: schema test clippy build fmt compile check_contracts

schema:
	@find contracts/* -maxdepth 2 -type f -name Cargo.toml -execdir cargo schema \;
test:
	@cargo test

clippy:
	@cargo clippy --all --all-targets -- -D warnings

fmt:
	@cargo fmt -- --check

doc:
	@cargo doc

compile:
	@docker run --rm -v "$(CURDIR)":/code \
		--mount type=volume,source="$(notdir $(CURDIR))_cache",target=/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.15.0
        @sudo chown -R $(shell id -u):$(shell id -g) artifacts

compile_arm64:
	@docker run --rm -v "$(CURDIR)":/code \
		--mount type=volume,source="$(notdir $(CURDIR))_cache",target=/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/arm64 \
		cosmwasm/workspace-optimizer-arm64:0.15.0
        @sudo chown -R $(shell id -u):$(shell id -g) artifacts
	@cd artifacts && for file in *-aarch64.wasm; do cp -f "$$file" "$${file%-aarch64.wasm}.wasm"; done

check_contracts:
	@cargo install cosmwasm-check
	@cosmwasm-check --available-capabilities iterator,staking,stargate,neutron,cosmwasm_1_1,cosmwasm_1_2 artifacts/*.wasm

build: schema clippy test fmt doc compile check_contracts
