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
- `make compile_arm64`: Compile for the ARM64 architecture.
- `make fmt`: Format the Rust code.
- `make schema`: Generate and validate JSON schemas.
- `make test`: Run all tests to ensure contract correctness.

### Running Integration Tests

To run the integration tests located in the `integration_tests` directory:

1. Ensure you have Node.js and Yarn installed.
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
