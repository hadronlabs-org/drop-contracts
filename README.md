# Drop Protocol Contracts

![Rectangle 161433](https://github.com/hadronlabs-org/drop-contracts/assets/103267218/f0faf991-7954-4e65-8032-73e6e4840ef3)

This repository contains the smart contracts of Drop Protocol. The project is organized into three main directories:

- `contracts`: This directory contains the core smart contracts for the Drop Protocol, written in Rust using the Cosmwasm framework.

- `integration_tests`: This directory includes comprehensive integration tests written in TypeScript and leveraging the Vitest testing framework. These tests verify the contracts' behavior in various scenarios, ensuring correctness and reliability.

- `packages`: This directory contains reusable Rust packages that are shared across different contracts or tools. It helps modularize and streamline the codebase.

## Getting Started

### Prerequisites

Ensure that you have the following installed:

- Rust (via [rustup](https://rustup.rs/))
- Cosmwasm (check out the [Cosmwasm documentation](https://docs.cosmwasm.com/))
- Node.js (for TypeScript integration tests)
- Yarn (as an alternative package manager)
- Docker (to manage images)

### Building Contracts

All build, test, and code quality tasks are managed using `make`. Below are the primary commands:

- `make build`: Compile the contracts to WebAssembly (Wasm) format.
- `make check_contracts`: Verify all contracts for issues.
- `make clippy`: Run `clippy` for Rust linting and suggestions.
- `make compile`: Compile the contracts to the Wasm target.
- `make compile_neutron`: **Production build for Neutron mainnet.** Use this instead of `make compile` for anything that will be stored on chain (see below).
- `make compile_arm64`: Compile for the ARM64 architecture.
- `make fmt`: Format the Rust code.
- `make schema`: Generate and validate JSON schemas.
- `make test`: Run all tests to ensure contract correctness.
- `make build_ts_client`: Build the TypeScript client for the integration tests and coordinator. It uses generated json schema from the contracts. 

### Production builds for Neutron

Since cosmwasm-vm 3.0.9 (wasmvm 3.0.7, Neutron v11.1.0+), code upload performs a static check that
rejects any Wasm function with more than **100 locals** (and more than 10,000 locals in the whole
module). The output of `make compile` violates this for several contracts (core, factory, puppeteer,
staker, withdrawal manager, ...), and `neutrond tx wasm store` fails with:

```
Error during static Wasm validation: Wasm contract contains function with more than 100 locals: 113
```

The cause is the `wasm-opt -Os` step of the optimizer: binaryen inlines every function that has a
single call site regardless of its size, so all execute handlers end up inside the exported
`execute` function. Rust `#[inline(never)]` does not help, and newer optimizer images
(`cosmwasm/optimizer:0.16.x/0.17.x`) ship the same binaryen and produce the same result.

`make compile_neutron` uses the same image (`cosmwasm/workspace-optimizer:0.15.1`, `linux/amd64`)
and the same cargo build as `make compile`, and only adds `--one-caller-inline-max-function-size=0`
to `wasm-opt`. It does not change contract behaviour; the binaries are slightly larger and
the checksums differ from `make compile`.

When building or verifying a contract for upload:

1. Build with `make compile_neutron` on `linux/amd64` (an arm64 build gives different checksums).
2. Verify the checksum in `artifacts/checksums.txt` against the one in the upgrade proposal.
3. Dry-run the upload against mainnet, which runs the same static validation as the real upload:
   ```sh
   neutrond tx wasm store artifacts/<contract>.wasm --from <address> --chain-id neutron-1 \
     --node <rpc> --gas-prices 0.0055untrn --dry-run
   ```
   `cosmwasm-check` only enforces the locals limit from version 3.0.9.

Example: the dATOM withdrawal manager haircut (`fix/datom-wm-haircut`, commit `da87d2cf`) builds
to `04f4e59156c299405755a8d2b5a1daa2c91f29213719412f18716d1e2ac6420a` with `make compile_neutron`.
The `make compile` build (`d950fc76…`) is rejected by mainnet.

### Running Integration Tests

To run the integration tests located in the `integration_tests` directory:

1. Ensure you have Node.js and Yarn installed and you have build the Typescript client
2. Navigate to the `integration_tests` folder and install the dependencies:
   ```bash
   cd integration_tests
   yarn install
   ```
3. Prepare the necessary Docker images with:
   ```bash
   yarn build-images
   ```
4. Run the tests using the Vitest framework:
   ```bash
   yarn test
   ```

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.
