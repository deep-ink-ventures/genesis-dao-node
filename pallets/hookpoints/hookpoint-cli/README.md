# Hookpoint CLI

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-yellow.svg)](https://www.apache.org/licenses/LICENSE-2.0)

The Hookpoint CLI is a powerful tool designed to streamline the integration of hook points in Substrate and ink! smart contracts. With this CLI, developers can effortlessly bridge the gap between blockchain runtime and smart contract development, ensuring a cohesive and efficient workflow.


## Features

- **Interactive Configuration**: A step-by-step configuration wizard simplifies the setup of `hookpoints.json`.
- **Automatic Code Generation**: Say goodbye to manual boilerplate! Generate hook code for both Substrate pallets and associated ink! contracts.
- **Type Mapping**: Provides seamless mapping between ink! and Substrate types, ensuring compatibility and type safety across platforms.
- **Modularity**: Crafted with extensibility in mind, allowing for easy expansion and compatibility with other tools.
- **Integrated with Hookpoint Pallet**: The Hookpoint CLI is designed to work seamlessly with the `hookpoint` pallet. Integrate the pallet into your Substrate node to unlock the full potential of the CLI.
- **Dependency on `pallet_contracts`**: Built on top of the robust `pallet_contracts` pallet from Parity.

## Installation

```bash
# Clone the repository
git clone [repository_url] hookpoint-cli

# Navigate to the directory
cd hookpoint-cli

# Build the project
cargo build --release

## Usage

Configure hookpoints interactively:

```shell
./hookpoint-cli configure --substrate-dir [YOUR_SUBSTRATE_DIRECTORY]
```

This will create a `hookpoint.json` with all the definitions of your hookpoints.

> You don't have to give your substrate dir if it's the current directory!


Generate hooks:

```shell
./hookpoint-cli generate --substrate-dir [YOUR_SUBSTRATE_DIRECTORY]
```

This will generate three folders under `./contracts/hooks`:

- an ink! trait with all your callback functions into the ink! universe
- an ink! boilerplate contract, batteries included with all callbacks and it's default values, working unit tests & functional e2e tests.
- an ink! test contract that you can use to test the integration in your substrate code
- within each pallet that has configured hooks an abstracted-everything-away `hooks.rs` with simple functions to call from within substrate. No bytes-wrestling, no decoding/encoding, no interaction with ink/substrate wiring. It just works.

Of course, here's the markdown from the "Documentation" section onwards:

## Documentation

Dive deeper into the functionalities and modules by exploring the inline documentation available in each source file.

## Contributing

Contributions are warmly welcomed! Check out [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on how to get involved.

## License

Licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for more details.
