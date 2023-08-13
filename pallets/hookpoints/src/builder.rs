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

//! HookPoint Module
//!
//! This module provides a mechanism to define and manage hook points in the system. A hook point allows specific callbacks
//! to be registered and executed based on various conditions or triggers. It's particularly useful for systems
//! that require extensibility or plugins, where third-party developers can register their own callbacks to enhance functionality.
//!
//! The main structure provided by this module is the `HookPoint`, which encapsulates information about the callback, its owner,
//! the origin that can invoke it, and any associated data. There are also methods to create a new `HookPoint` and to add
//! additional data or arguments to it.

use codec::{Encode};
use frame_support::sp_io::hashing::blake2_256;
use sp_std::prelude::*;

fn selector_from_str(callback: &str) -> Vec<u8> {
    let hash = blake2_256(callback.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]].to_vec()
}

/// Represents a contract in the system.
///
/// A `Contract` contains information about the contract's account ID and code hash.
pub struct ContractDeployment<AccountId> {
    /// The account ID of the contract.
    pub creator: AccountId,

    /// The contract's code hash.
    pub code: Vec<u8>,

    /// Data associated with the hook point.
    pub init_args: Vec<u8>,

    /// A random salt to init the contract and predict the address
    pub salt: Vec<u8>
}

impl<AccountId> ContractDeployment<AccountId> {
    pub fn new(callback: &str, creator: AccountId, code: Vec<u8>, salt: Vec<u8>) -> Self {
        ContractDeployment {
            creator,
            code,
            salt,
            init_args: selector_from_str(callback)
        }
    }

    pub fn add_init_arg<T>(mut self, arg: T) -> Self
    where
        T: Encode
    {
        self.init_args.append(&mut arg.encode());
        self
    }
}

/// Represents a hook point in the system.
///
/// A `HookPoint` contains information about the owner of the hook,
/// the origin for the call, the specific callback to be triggered,
/// and any associated data.
pub struct HookPoint<AccountId> {
    /// The account ID of the owner of the hook.
    pub owner: AccountId,

    /// The origin account ID initiating the call.
    pub origin: AccountId,

    /// The specific callback identifier to be triggered.
    pub callback: Vec<u8>,

    /// Data associated with the hook point.
    pub data: Vec<u8>
}

impl<AccountId> HookPoint<AccountId> {
    /// Creates a new `HookPoint` instance.
    ///
    /// The callback's first four bytes (Blake2b hash) are added to the data field for efficient lookup.
    ///
    /// # Arguments
    ///
    /// * `callback`: A string representing the callback identifier.
    /// * `owner`: The account ID of the owner of the hook.
    /// * `origin`: The origin account ID initiating the call.
    pub fn new(callback: &str, owner: AccountId, origin: AccountId) -> Self {
        HookPoint {
            owner,
            origin,
            callback: callback.into(),
            data: selector_from_str(callback)
        }
    }

    /// Appends an argument to the `HookPoint`'s data field.
    ///
    /// This allows for dynamic addition of data to the hook point.
    ///
    /// # Arguments
    ///
    /// * `arg`: The argument to be added. It should implement the `Encode` trait.
    pub fn add_arg<T>(mut self, arg: T) -> Self
    where
        T: Encode
    {
        self.data.append(&mut arg.encode());
        self
    }
}
