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

//! This module provides various utility functions to manage and operate on hook points within the system.
//! These functions range from retrieving a registered callback for a given owner, executing a hook point,
//! installing an ink! contract, to creating a new hook point.
//!
//! The primary purpose of this module is to act as a bridge between the core logic of the hook points
//! and the external components like the ink! contracts. By offering a clear API with well-defined
//! functions, it simplifies the interaction and management of hook points, making it easier for developers
//! to integrate and extend functionalities.

use codec::Decode;
use frame_support::BoundedVec;
use super::*;
use frame_support::weights::Weight;
use frame_support::pallet_prelude::DispatchError;
use pallet_contracts::{CollectEvents, DebugInfo, Determinism, Pallet as Contracts};
use pallet_contracts_primitives::Code;
use crate::builder::{ContractDeployment, HookPoint};

impl<T: Config> Pallet<T> {
    /// Retrieves the callback registered by a given owner for a specific callback name.
    ///
    /// This function first checks for a specific callback registered by the owner and, if not found,
    /// falls back to the global callback registered by the owner.
    ///
    /// # Arguments
    ///
    /// * `owner` - The account id of the owner of the registered callback.
    /// * `callback_name` - The name of the callback to be fetched.
    pub fn get_callback(owner: &T::AccountId, callback_name: Vec<u8>) -> Option<T::AccountId> {
        let call: BoundedVec<_, _> = callback_name.try_into().unwrap();
        Pallet::<T>::specific_callbacks(owner, call)
            .or_else(|| Pallet::<T>::callbacks(owner))
    }

    /// Executes the logic associated with a given hook point.
    ///
    /// This function is responsible for invoking the callback logic registered for a hook point.
    ///
    /// # Arguments
    ///
    /// * `hook_point` - A struct containing details about the hook point to be executed.
    pub fn execute<R>(hook_point: HookPoint<T::AccountId>) -> Result<R, DispatchError>
        where R: Decode
    {
        let callback = Pallet::<T>::get_callback(&hook_point.owner, hook_point.callback);
        let contract = callback.ok_or(DispatchError::Other("no contract"))?;
        let data = Contracts::<T>::bare_call(
            hook_point.origin,
            contract,
            0_u32.into(),
            Weight::from_all(10_000_000_000),
            Some(0_u32.into()),
            hook_point.data,
            DebugInfo::Skip,
            CollectEvents::Skip,
            Determinism::Enforced,
        ).result?.data;
        <Result<R, DispatchError>>::decode(&mut &data[..])
            .map_err(|_| DispatchError::Other("decoding error"))
            .unwrap()
    }

    /// Deploys an ink! contract into the runtime.
    ///
    /// This function handles the instantiation of an ink! contract, making it available for interaction.
    ///
    /// # Arguments
    ///
    /// * `contract_deployment`: An instance of the `ContractDeployment` struct containing all the necessary details for deployment.
    pub fn install(contract_deployment: ContractDeployment<T::AccountId>) -> Result<T::AccountId, DispatchError> {
        let ContractDeployment {
            creator,
            code,
            init_args,
            salt,
            ..
        } = contract_deployment;

        let contract_instantiate_result = Contracts::<T>::bare_instantiate(
            creator,
            0_u32.into(),
            Weight::MAX,
            Some(100_u32.into()),
            Code::Upload(code),
            init_args,
            salt,
            pallet_contracts::DebugInfo::UnsafeDebug,
            pallet_contracts::CollectEvents::UnsafeCollect,
        );
        Ok(contract_instantiate_result.result?.account_id)
    }

    /// Initializes and returns a new hook point.
    ///
    /// This function creates a new hook point, which can be further customized and then executed.
    ///
    /// # Arguments
    ///
    /// * `callback` - The fully qualified name of the callback to be associated with the hook point.
    /// * `owner` - The account id of the owner of the hook point.
    /// * `origin` - The account id initiating the creation of the hook point.
    pub fn create(callback: &str, owner: T::AccountId, origin: T::AccountId) -> HookPoint<T::AccountId> {
        HookPoint::<T::AccountId>::new(callback, owner, origin)
    }

    /// Prepares the details for a new contract deployment.
    ///
    /// This function assists in creating a `ContractDeployment` instance by setting up the necessary data,
    /// such as the selector derived from the given callback. The created instance can then be further customized
    /// with additional arguments using the `add_arg` method.
    ///
    /// # Arguments
    ///
    /// * `callback` - A string representing the callback identifier.
    /// * `creator` - The account ID of the entity deploying the contract.
    /// * `code` - The compiled wasm code of the ink! contract.
    /// * `salt` - A unique salt to ensure the resulting contract address is unique.
    ///
    /// # Returns
    ///
    /// A new `ContractDeployment` instance.
    pub fn prepare_deployment(callback: &str, creator: T::AccountId, code: Vec<u8>, salt: Vec<u8>) -> ContractDeployment<T::AccountId> {
        ContractDeployment::<T::AccountId>::new(callback, creator, code, salt)
    }
}
