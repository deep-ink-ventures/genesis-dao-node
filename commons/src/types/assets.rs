use codec::Encode;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// AssetStatus holds the current state of the asset. It could either be Live and available for use,
/// or in a Destroying state.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum AssetStatus {
	/// The asset is active and able to be used.
	Live,
	/// The asset is currently being destroyed, and all actions are no longer permitted on the
	/// asset. Once set to `Destroying`, the asset can never transition back to a `Live` state.
	Destroying,
	/// The asset has been destroyed
	Destroyed,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetDetails<Balance, AccountId> {
	/// Can destroy this asset
	pub owner: AccountId,
	/// The total supply across all accounts.
	pub supply: Balance,
	/// The ED for virtual accounts.
	pub min_balance: Balance,
	/// The total number of accounts.
	pub accounts: u32,
	/// The total number of approvals.
	pub approvals: u32,
	/// The status of the asset
	pub status: AssetStatus,
}
