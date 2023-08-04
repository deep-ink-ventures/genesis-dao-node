#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Maximum callback identifier length
		#[pallet::constant]
		type MaxLengthId: Get<u32>;
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
		///
		/// - `contract`: The contract address to use as endpoint for all callbacks
		#[pallet::call_index(0)]
		#[pallet::weight(0)] //#[pallet::weight(<T as Config>::WeightInfo::register_global_callback())]
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
		/// Callbacks registered here get precedence over the global callback and are therefore
		/// enabling an account or entity to utilize extensions from different developers or
		/// protocols
		///
		/// - `contract`: contract address to use as endpoint for all callbacks
		/// - `id`: id for this registration
		#[pallet::call_index(1)]
		#[pallet::weight(0)] //#[pallet::weight(<T as Config>::WeightInfo::register_specific_callback())]
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
