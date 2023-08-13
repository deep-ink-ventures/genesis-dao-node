# HookPoints Pallet

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-yellow.svg)](https://www.apache.org/licenses/LICENSE-2.0)

The HookPoints pallet provides a flexible framework to interact with registered callbacks within the Substrate runtime. Users can register global callbacks and specific callbacks, offering a robust mechanism to invoke extensions from different developers or protocols.

## Basic idea
## Features

- **Global Callbacks**: Allow users to set a default callback that can be triggered universally.
  
- **Specific Callbacks**: Override the default by specifying which callback to trigger under certain conditions. This feature enables more nuanced control and can be used to integrate different extensions or protocols.

- **Flexible Encoding**: The pallet is designed to work with bytes directly, allowing the users to encode their data in their preferred way before sending it.

- **Runtime Benchmarking**: Built-in support for runtime benchmarks to ensure performance and efficiency.

## Usage

### Registering a Global Callback

Users can register a global callback, which becomes the default point of interaction for any calls unless a specific callback is defined for that particular interaction.

```rust
HookPoints::register_global_callback(origin, contract_address);
```

## Registering a Specific Callback
Users can also define specific callbacks, which will take precedence over global callbacks when triggered.

## Registering a Specific Callback
Users can also define specific callbacks, which will take precedence over global callbacks when triggered.

```rust
let hook_point = HookPoints::create(callback_name, owner, origin);
HookPoints::execute(hook_point);
```

## Prerequisites
Ensure that the pallet_contracts is included in your runtime, as HookPoints relies on it for contract interactions.

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

## Testing
Here's how a typical test might look like:

```rust
#[test]
fn execute_callback() {
    new_test_ext().execute_with(|| {
        let creator = AccountId32::new([1u8; 32]);
        
        // your compiled wasm ink contract!
        let contract_path = "test_contract.wasm";

        let mut contract_deployment = HookPoints::prepare_deployment("new", creator.clone(), std::fs::read(contract_path).unwrap(), vec![]);
        contract_deployment = contract_deployment.add_arg(42u128);

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