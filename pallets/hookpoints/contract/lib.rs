// Copyright (C) Deep Ink Ventures GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Helper contract for testing. This just provides a simple multiply function.
//! The contract is used in the tests of the hookpoints pallet.
//! We do export a `test_contract.wasm` file in this crate so tests run out of the box.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod contract {

    #[ink(storage)]
    pub struct Contract {
        value: u128,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(init_value: u128) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn multiply(&mut self, by: u128) -> u128 {
            self.value * by
        }

        #[ink(message)]
        pub fn get(&self) -> u128 {
            self.value
        }
    }
}
