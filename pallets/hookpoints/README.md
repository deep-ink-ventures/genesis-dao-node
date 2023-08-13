# HookPoints Pallet

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-yellow.svg)](https://www.apache.org/licenses/LICENSE-2.0)

The HookPoints pallet provides a flexible framework to interact with registered callbacks within the Substrate runtime. Users can register global callbacks and specific callbacks, offering a robust mechanism to invoke extensions from different developers or protocols.

## Conceptual Overview
Hook points draw inspiration from the web2 era, where platforms like Magento, WordPress, and Shopify thrived due to their extensible plugin systems. Similarly, Substrate offers a specialized blockchain core that emphasizes the Unix principle of doing one thing right. Hook points build on this, introducing a plugin system that enhances core functionalities and allows for added innovations.

Using ink! and pallet_contracts, hook points enable seamless integration between Substrate pallets and ink modules. They abstract away challenges like byte handling and encoding, providing a higher-level interface for developers. This bridges the Substrate and ink environments, simplifying development.

To complement this, a CLI tool is available for hook points. It automates code generation for both Substrate and ink, handling tests and other essentials. This minimizes boilerplate, letting developers focus on core logic, and streamlining the blockchain development process.

## Features

- **Global Callbacks**: Allow users to set a default callback that can be triggered universally.
  
- **Specific Callbacks**: Override the default by specifying which callback to trigger under certain conditions. This feature enables more nuanced control and can be used to integrate different extensions or protocols.

- **Flexible Encoding**: The pallet is designed to work with bytes directly, allowing the users to encode their data in their preferred way before sending it.

- **Runtime Benchmarking**: Built-in support for runtime benchmarks to ensure performance and efficiency.

## Prerequisites
Ensure that the pallet_contracts is included in your runtime, as HookPoints relies on it for contract interactions.

Install the CLI Tool:

```shell
> cargo install hookpoint-cli
```

## Installation
To add the HookPoints pallet to your runtime, add it as a dependency in your Cargo.toml and then include it in your runtime's list of pallets.

```rust
impl pallet_hookpoints::Config for Runtime { 
    // your event of choice 
    type RuntimeEvent = RuntimeEvent;
    // max length of a callback, this is typically ContractName::function_name 
    type MaxLengthId = ConstU32<64>;
    // runtime weights, if in doubt use ours 
    type WeightInfo = pallet_hookpoints::weights::SubstrateWeight<Runtime>;
}
```

## Usage
Move to your substrate root folder and run:

```shell
hookpoint-cli configure
```

This will guide you through the setup process of creating hook points. You give your project a name (this will uy be the name of the trait that ink! devs need to implement).

Afterwards you can interactively create hookpoints:

- add a name, that's the ink-function to call
- add arguments by name and type
- add a return value and a default (the default will be used to generate boilerplate code and can a primtive or one of the defined arguments).

For example, at [GenesisDao](https://github.com/deep-ink-ventures/genesis-dao-node), we use hookpoints everywhere.

For an example, we want to have a hookpoint where DAO owners can alter the voting power of their token holders to implement alternative voting models, add vesting wallets and so on.
So we have a hook point called `on_vote`, added the arguments `voter` as `AccountId` and `voting_power` as `Balance`. We return the changed `voting_power` as balance and per default just the unaltered `voting_power`.

The CLI will create a hookpoints.json in your root folder.

Run

```shell
hookpoint-cli generate
```

And the CLI will generate

- an ink! trait with all your callback functions into the ink! universe
- an ink! boilerplate contract, batteries included with all callbacks and it's default values, working unit tests & functional e2e tests.
- an ink! test contract that you can use to test the integration in your substrate code
- within each pallet that has configured hooks an abstracted-everything-away hooks.rs with simple functions to call from within substrate. No bytes-wrestling, no decoding/encoding, no interaction with ink/substrate wiring. It just works.

Go to your pallet where you want to implement the hookpoint and just use the provided function in hooks.rs.

In our example this would be:

```rust
let new_voting_power = on_vote(dao_owner, original_caller_of_the_extrinsic, voter, voting_power);
```

That's it.

### Extrinsics 

Users can register a global callback, which becomes the default point of interaction for any calls unless a specific callback is defined for that particular interaction.

This is normally done by someone who "owns" a part of your application and wants to alter it's behaviour. In the example above it's DAO.

## Registering a Specific Callback
Users can also define specific callbacks, which will take precedence over global callbacks when triggered.

## Registering a Specific Callback
Users can also define specific callbacks, which will take precedence over global callbacks when triggered.


## Testing
Here's how a typical test might look like:

```rust
#[test]
fn execute_callback() {
    new_test_ext().execute_with(|| {
        let creator = AccountId32::new([1u8; 32]);
        
        // your compiled wasm ink contract!
        let contract_path = "test_contract.wasm";

        let contract_deployment = HookPoints::prepare_deployment("new", creator.clone(), std::fs::read(contract_path).unwrap(), vec![])
            .add_init_arg(42u128);

        let contract_address = HookPoints::install(contract_deployment)
            .expect("Contract installation should be successful");

        // Register the contract for callbacks (if you have such a step)
        HookPoints::register_global_callback(RuntimeOrigin::signed(creator.clone()), contract_address.clone()).unwrap();

        // Create a HookPoint for the "multiply" function
        let hookpoint = HookPoints::create("multiply", creator.clone(), creator.clone())
            .add_arg(2u128);

        // Execute the "multiply" function using the HookPoint
        let result: Result<u128, _> = HookPoints::execute(hookpoint);

        // Ensure the result is Ok and equals to 84 (since 42 * 2 = 84)
        assert_eq!(result.unwrap(), 84);
    });
}
```

This uses our test contract from the `./contract` folder.

## License
Licensed under the Apache License, Version 2.0. You may not use this pallet except in compliance with the License.