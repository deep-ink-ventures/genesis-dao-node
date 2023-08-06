use codec::Decode;
use frame_support::BoundedVec;
use super::*;
use frame_support::weights::Weight;
use frame_support::pallet_prelude::DispatchError;
use pallet_contracts::{CollectEvents, DebugInfo, Determinism, Pallet as Contracts};
use pallet_contracts_primitives::Code;
use crate::builder::{Callback, HookPoint};

impl<T: Config> Pallet<T> {

	/// Loads a callback for the given owner.
	///
	/// - `owner` - the account id of the owner of the register callback
	/// - `callback_name` - the callback name to call
	pub fn get_callback(owner: &T::AccountId, callback_name: &str) -> Option<T::AccountId> {
		let call: BoundedVec<_, _> = callback_name.as_bytes().to_vec().try_into().unwrap();
		Pallet::<T>::specific_callbacks(owner, call)
			.or_else(|| Pallet::<T>::callbacks(owner))
	}

	/// Executes a hook point. The caller needs to encode the data, we're all bytes here.
	///
	/// - `owner` - the account id of the owner of the register callback
	/// - `args` - Arguments with type bounds, we'll do the encoding for you
	pub fn execute<R>(hook_point: HookPoint<T::AccountId>) -> Result<R, DispatchError>
	where R: Decode
	{
		let callback = Pallet::<T>::get_callback(&hook_point.owner, hook_point.callback.name.as_str());
		let contract = callback.ok_or(DispatchError::Other("no contract"))?;
		let data = Contracts::<T>::bare_call(
			hook_point.origin,
			contract,
			0_u32.into(),
			Weight::from_all(10_000_000_000),
			Some(0_u32.into()),
			hook_point.callback.data,
			DebugInfo::Skip,
			CollectEvents::Skip,
			Determinism::Enforced,
		).result?.data;
		<Result<R, DispatchError>>::decode(&mut &data[..])
			.map_err(|_| DispatchError::Other("decoding error"))
			.unwrap()
	}

	/// Installs an ink! contract.
	///
	/// - `creator` - the account deploying the contracts
	/// - `init_args` - encoded initial arguments (prefixed by constructor name)
	/// - `salt` - the salt for the contract deployment
	pub fn install(creator: T::AccountId, code: Vec<u8>, init_args: Vec<u8>, salt: Vec<u8>) -> Result<T::AccountId, DispatchError> {
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

	pub fn create(owner: T::AccountId, origin: T::AccountId, mod_name: &str, callback: Callback) -> HookPoint<T::AccountId> {
		HookPoint::<T::AccountId>::new(owner, origin, mod_name, callback)
	}

	pub fn callback(func_name: &str) -> Callback {
       Callback::new(func_name)
   }
}
