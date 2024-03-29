#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Get;
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use pallet_dao_assets::WeightInfo;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::StaticLookup, DispatchError, ModuleError};
use sp_std::marker::PhantomData;

enum AssetsFunc {
	Transfer = 100,
	TransferKeepAlive = 101,
	ApproveTransfer = 102,
	CancelApproval = 103,
	TransferApproved = 104,
	BalanceOf = 105,
	TotalSupply = 106,
	Allowance = 107,
}

impl TryFrom<u16> for AssetsFunc {
	type Error = DispatchError;

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		match value {
			100 => Ok(AssetsFunc::Transfer),
			101 => Ok(AssetsFunc::TransferKeepAlive),
			102 => Ok(AssetsFunc::ApproveTransfer),
			103 => Ok(AssetsFunc::CancelApproval),
			104 => Ok(AssetsFunc::TransferApproved),
			105 => Ok(AssetsFunc::BalanceOf),
			106 => Ok(AssetsFunc::TotalSupply),
			107 => Ok(AssetsFunc::Allowance),
			_ => Err(DispatchError::Other("PalletDaoAssetsExtension: Unimplemented func_id")),
		}
	}
}

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Outcome {
	/// Success
	Success = 0,
	/// Account balance must be greater than or equal to the transfer amount.
	BalanceLow = 1,
	/// The account to alter does not exist.
	NoAccount = 2,
	/// The signing account has no permission to do the operation.
	NoPermission = 3,
	/// The given asset ID is unknown.
	Unknown = 4,
	/// The asset ID is already taken.
	InUse = 5,
	/// Invalid witness data given.
	BadWitness = 6,
	/// Minimum balance should be non-zero.
	MinBalanceZero = 7,
	/// Invalid metadata given.
	BadMetadata = 8,
	/// No approval exists that would allow the transfer.
	Unapproved = 9,
	/// The source account would not survive the transfer and it needs to stay alive.
	WouldDie = 10,
	/// The asset-account already exists.
	AlreadyExists = 11,
	/// The operation would result in funds being burned.
	WouldBurn = 12,
	/// The asset is not live, and likely being destroyed.
	AssetNotLive = 13,
	/// The asset status is not the expected status.
	/// Unknown error
	RuntimeError = 99,
	OriginCannotBeCaller = 100,
}

impl From<DispatchError> for Outcome {
	fn from(input: DispatchError) -> Self {
		let error_text = match input {
			DispatchError::Module(ModuleError { message, .. }) => message,
			_ => Some("No module error Info"),
		};
		match error_text {
			Some("BalanceLow") => Outcome::BalanceLow,
			Some("NoAccount") => Outcome::NoAccount,
			Some("NoPermission") => Outcome::NoPermission,
			Some("Unknown") => Outcome::Unknown,
			Some("InUse") => Outcome::InUse,
			Some("BadWitness") => Outcome::BadWitness,
			Some("MinBalanceZero") => Outcome::MinBalanceZero,
			Some("BadMetadata") => Outcome::BadMetadata,
			Some("Unapproved") => Outcome::Unapproved,
			Some("WouldDie") => Outcome::WouldDie,
			Some("AlreadyExists") => Outcome::AlreadyExists,
			Some("WouldBurn") => Outcome::WouldBurn,
			Some("AssetNotLive") => Outcome::AssetNotLive,
			_ => Outcome::RuntimeError,
		}
	}
}

pub struct AssetsExtension<T>(PhantomData<T>);

impl<T> Default for AssetsExtension<T> {
	fn default() -> Self {
		AssetsExtension(PhantomData)
	}
}
impl<T> ChainExtension<T> for AssetsExtension<T>
where
	T: pallet_dao_assets::Config + pallet_contracts::Config,
	<T as pallet_dao_core::Config>::AssetId: Copy,
	<<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
	<T as SysConfig>::AccountId: From<[u8; 32]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
	{
		let func_id = env.func_id().try_into()?;
		let mut env = env.buf_in_buf_out();

		let caller_account = env.ext().caller().account_id().cloned()?;
		let origin = RawOrigin::Signed(caller_account).into();

		let call_result = match func_id {
			AssetsFunc::BalanceOf => {
				let (id, who): (<T as pallet_dao_core::Config>::AssetId, T::AccountId) =
					env.read_as()?;

				env.charge_weight(T::DbWeight::get().reads(1_u64))?;

				let balance = pallet_dao_assets::Pallet::<T>::balance(id, who);
				env.write(&balance.encode(), false, None)?;
				Ok(())
			},
			AssetsFunc::TotalSupply => {
				let id: <T as pallet_dao_core::Config>::AssetId = env.read_as()?;

				env.charge_weight(T::DbWeight::get().reads(1_u64))?;

				let total_supply = pallet_dao_assets::Pallet::<T>::total_supply(id);
				env.write(&total_supply.encode(), false, None)?;
				Ok(())
			},
			AssetsFunc::Allowance => {
				let (id, owner, delegate): (
					<T as pallet_dao_core::Config>::AssetId,
					T::AccountId,
					T::AccountId,
				) = env.read_as()?;

				env.charge_weight(T::DbWeight::get().reads(1_u64))?;

				let allowance = pallet_dao_assets::Pallet::<T>::allowance(id, &owner, &delegate);
				env.write(&allowance.encode(), false, None)?;
				Ok(())
			},
			AssetsFunc::Transfer => {
				let (id, target, amount): (
					<T as pallet_dao_core::Config>::AssetId,
					T::AccountId,
					T::Balance,
				) = env.read_as()?;

				let weight = <T as pallet_dao_assets::Config>::WeightInfo::transfer();
				env.charge_weight(weight)?;

				env.ext().caller_is_origin();
				pallet_dao_assets::Pallet::<T>::transfer(origin, id.into(), target.into(), amount)
			},
			AssetsFunc::TransferKeepAlive => {
				let (id, target, amount): (
					<T as pallet_dao_core::Config>::AssetId,
					T::AccountId,
					T::Balance,
				) = env.read_as()?;

				let weight = <T as pallet_dao_assets::Config>::WeightInfo::transfer_keep_alive();
				env.charge_weight(weight)?;

				pallet_dao_assets::Pallet::<T>::transfer_keep_alive(
					origin,
					id.into(),
					target.into(),
					amount,
				)
			},
			AssetsFunc::ApproveTransfer => {
				let (id, delegate, amount): (
					<T as pallet_dao_core::Config>::AssetId,
					T::AccountId,
					T::Balance,
				) = env.read_as()?;

				let weight = <T as pallet_dao_assets::Config>::WeightInfo::approve_transfer();
				env.charge_weight(weight)?;

				pallet_dao_assets::Pallet::<T>::approve_transfer(
					origin,
					id.into(),
					delegate.into(),
					amount,
				)
			},
			AssetsFunc::CancelApproval => {
				let (id, delegate): (<T as pallet_dao_core::Config>::AssetId, T::AccountId) =
					env.read_as()?;

				let weight = <T as pallet_dao_assets::Config>::WeightInfo::cancel_approval();
				env.charge_weight(weight)?;

				pallet_dao_assets::Pallet::<T>::cancel_approval(origin, id.into(), delegate.into())
			},
			AssetsFunc::TransferApproved => {
				let (id, owner, destination, amount): (
					<T as pallet_dao_core::Config>::AssetId,
					T::AccountId,
					T::AccountId,
					T::Balance,
				) = env.read_as()?;

				let weight = <T as pallet_dao_assets::Config>::WeightInfo::transfer_approved();
				env.charge_weight(weight)?;

				pallet_dao_assets::Pallet::<T>::transfer_approved(
					origin,
					id.into(),
					owner.into(),
					destination.into(),
					amount,
				)
			},
		};
		match call_result {
			Err(e) => {
				let mapped_error = Outcome::from(e);
				Ok(RetVal::Converging(mapped_error as u32))
			},
			Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
		}
	}

	fn enabled() -> bool {
		true
	}
}
