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

//! HookPoint Pallet
//!
//! The HookPoint pallet provides a mechanism to manage, register, and execute hook points in the runtime.
//! These hook points act as predefined points in code execution where custom logic (in the form of
//! callbacks) can be injected, allowing for extensibility and custom behaviors.
//!
//! The primary components of this pallet are:
//! - `GlobalCallbacks`: A storage map that holds global callbacks registered for a specific owner.
//!   These callbacks act as default behavior when no specific callback is registered for an event.
//! - `SpecificCallbacks`: A double map storage that holds specific callbacks for particular events.
//!   These callbacks take precedence over global callbacks.
//! - Registration Functions: Allows users to register global or specific callbacks.
//! - Execution Function: Allows the execution of a hook point, invoking the associated callback.
//!
//! This pallet interacts closely with the `pallet_contracts` to manage and execute contracts as callbacks.

#![cfg_attr(not(feature = "std"), no_std)]
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod functions;
mod builder;
pub mod weights;
use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_contracts::Config {

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum callback identifier length
		#[pallet::constant]
		type MaxLengthId: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn callbacks)]
	pub type GlobalCallbacks<T: Config> =
		StorageMap<Hasher = Blake2_128Concat, Key = T::AccountId, Value = T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn specific_callbacks)]
	pub type SpecificCallbacks<T: Config> = StorageDoubleMap<
		Hasher1 = Blake2_128Concat,
		Key1 = T::AccountId,
		Hasher2 = Blake2_128Concat,
		Key2 = BoundedVec<u8, T::MaxLengthId>,
		Value = T::AccountId,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		GlobalCallbackRegistered {
			who: T::AccountId,
		},
		SpecificCallbackRegistered {
			who: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// id is too long
		IdTooLong,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a global ink! callback for an address.
		///
		/// This contract is used for each callback within the substrate system.
		/// It allows a user to set a default callback contract that will be triggered
        /// for any hook point unless a specific callback is set.
		///
		/// - `contract`: The contract address to use as the endpoint for all callbacks.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::register_global_callback())]
		pub fn register_global_callback(
			origin: OriginFor<T>,
			contract: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			GlobalCallbacks::<T>::insert(&who, contract);
			Self::deposit_event(Event::GlobalCallbackRegistered { who });
			Ok(())
		}

		/// Register a specific ink! callback for an address.
		///
		/// Allows a user to set a callback for a specific hook point.
        /// This will take precedence over the global callback for the given hook point.
		///
		/// - `contract`: Contract address to use as the endpoint for this specific callback.
		/// - `id`: Identifier for this registration.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::register_specific_callback())]
		pub fn register_specific_callback(
			origin: OriginFor<T>,
			contract: T::AccountId,
			id: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let id: BoundedVec<_, _> = id.try_into().map_err(|_| Error::<T>::IdTooLong)?;
			SpecificCallbacks::<T>::insert(&who, id, contract);
			Self::deposit_event(Event::SpecificCallbackRegistered { who });
			Ok(())
		}
	}
}
