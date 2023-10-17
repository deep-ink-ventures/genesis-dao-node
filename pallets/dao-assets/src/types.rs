//! Various basic types for use in the assets pallet.

use super::*;
pub use commons::types::assets::*;
use frame_support::{pallet_prelude::*, traits::fungible};
pub use pallet_dao_core::AssetIdOf;

// Type alias for `frame_system`'s account id.
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
// This pallet's asset id and balance type.
pub type AssetBalanceOf<T> = <T as Config>::Balance;
// Generic fungible balance type.
pub type BalanceOf<F, T> = <F as fungible::Inspect<AccountIdOf<T>>>::Balance;
// The deposit balance type
pub type DepositBalanceOf<T> =
	<<T as pallet_dao_core::Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
// The account data for an asset
pub type AssetAccountOf<T> = AssetAccount<AssetBalanceOf<T>>;
pub type AssetDetailsOf<T> = AssetDetails<AssetBalanceOf<T>, AccountIdOf<T>>;
// Checkpoint alias
pub type CheckpointOf<T> =
	Checkpoint<AccountIdOf<T>, AssetBalanceOf<T>, <T as Config>::MaxDelegation>;

/// Data concerning an approval.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo)]
pub struct Approval<Balance, DepositBalance> {
	/// The amount of funds approved for the balance transfer from the owner to some delegated
	/// target.
	pub(super) amount: Balance,
	/// The amount reserved on the owner's account to hold this item in storage.
	pub(super) deposit: DepositBalance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetAccount<Balance> {
	/// Free balance.
	pub(super) balance: Balance,
	/// Reserved balance.
	pub(super) reserved: Balance,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct AssetMetadata<BoundedString> {
	/// The user friendly name of this asset. Limited in length by `StringLimit`.
	pub(super) name: BoundedString,
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	pub(super) symbol: BoundedString,
	/// The number of decimals this asset uses to represent one unit.
	pub(super) decimals: u8,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct TransferFlags {
	/// The debited account must stay alive at the end of the operation; an error is returned if
	/// this cannot be achieved legally.
	pub(super) keep_alive: bool,
	/// Less than the amount specified needs be debited by the operation for it to be considered
	/// successful. If `false`, then the amount debited will always be at least the amount
	/// specified.
	pub(super) best_effort: bool,
	/// Any additional funds debited (due to minimum balance requirements) should be burned rather
	/// than credited to the destination account.
	pub(super) burn_dust: bool,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(super) struct DebitFlags {
	/// The debited account must stay alive at the end of the operation; an error is returned if
	/// this cannot be achieved legally.
	pub(super) keep_alive: bool,
	/// Less than the amount specified needs be debited by the operation for it to be considered
	/// successful. If `false`, then the amount debited will always be at least the amount
	/// specified.
	pub(super) best_effort: bool,
}

impl From<TransferFlags> for DebitFlags {
	fn from(f: TransferFlags) -> Self {
		Self { keep_alive: f.keep_alive, best_effort: f.best_effort }
	}
}

/// Represent a single checkpoint
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(DelegatedMax))]
pub struct Checkpoint<AccountId: Ord, Balance: Zero, DelegatedMax: Get<u32>> {
	// how much is through self mutating action
	pub amount: Balance,
	// how much is through delegation from someone
	pub(crate) delegated: BoundedBTreeMap<AccountId, Balance, DelegatedMax>,
	// total sum of mutated and delegated. this saves up having to iterate
	// on delegated every time total amount is needed
	pub(super) total_delegation: Balance,
}

impl<
		AccountId: Ord + Clone + sp_std::fmt::Debug,
		Balance: CheckedAdd + Clone + Zero + Saturating + sp_std::fmt::Debug,
		DelegatedMax: Get<u32> + sp_std::fmt::Debug,
	> Checkpoint<AccountId, Balance, DelegatedMax>
{
	pub fn delegated_amount(&self) -> &Balance {
		&self.total_delegation
	}

	pub fn revoke_delegation(&mut self, from: &AccountId, from_chp: &mut Self) {
        let amount = self.delegated.remove(from).unwrap_or_else(Balance::zero);
        self.total_delegation = self.total_delegation.clone().saturating_sub(amount.clone());
        from_chp.amount = from_chp.amount.clone().saturating_add(amount);
	}

	pub fn add_delegation(&mut self, from: &AccountId, from_chp: &mut Self) -> Option<()> {
		let amount = self
			.delegated
			.get(from)
			.cloned()
			.unwrap_or_else(Balance::zero)
			.checked_add(&from_chp.amount)?;
		match self.delegated.try_insert(from.clone(), amount.clone()) {
			Ok(_) => {
                from_chp.amount = Zero::zero();
				self.total_delegation = self.total_delegation.clone().saturating_add(amount);
				Some(())
			},
			Err(_) => None,
		}
	}

	pub fn zero() -> Self {
		Self::default()
	}
}

impl<AccountId: Ord, Balance: Zero, DelegatedMax: Get<u32>> Default
	for Checkpoint<AccountId, Balance, DelegatedMax>
{
	fn default() -> Self {
		Self {
			amount: Zero::zero(),
			total_delegation: Zero::zero(),
			delegated: BoundedBTreeMap::new(),
		}
	}
}
