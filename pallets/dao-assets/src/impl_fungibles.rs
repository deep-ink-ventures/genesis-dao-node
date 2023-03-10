//! Implementations for fungibles trait.

use super::*;

impl<T: Config> fungibles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	type AssetId = T::AssetId;
	type Balance = T::Balance;

	fn total_issuance(asset: Self::AssetId) -> Self::Balance {
		Asset::<T>::get(asset).map(|x| x.supply).unwrap_or_else(Zero::zero)
	}

	fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
		Asset::<T>::get(asset).map(|x| x.min_balance).unwrap_or_else(Zero::zero)
	}

	fn balance(asset: Self::AssetId, who: &<T as SystemConfig>::AccountId) -> Self::Balance {
		Pallet::<T>::balance(asset, who)
	}

	fn reducible_balance(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		keep_alive: bool,
	) -> Self::Balance {
		Pallet::<T>::reducible_balance(asset, who, keep_alive).unwrap_or(Zero::zero())
	}

	fn can_deposit(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
		mint: bool,
	) -> DepositConsequence {
		Pallet::<T>::can_increase(asset, who, amount, mint)
	}

	fn can_withdraw(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> WithdrawConsequence<Self::Balance> {
		Pallet::<T>::can_decrease(asset, who, amount, false)
	}

	fn asset_exists(asset: Self::AssetId) -> bool {
		Asset::<T>::contains_key(asset)
	}
}

impl<T: Config> fungibles::InspectMetadata<<T as SystemConfig>::AccountId> for Pallet<T> {
	/// Return the name of an asset.
	fn name(asset: &Self::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).name.to_vec()
	}

	/// Return the symbol of an asset.
	fn symbol(asset: &Self::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).symbol.to_vec()
	}

	/// Return the decimals of an asset.
	fn decimals(asset: &Self::AssetId) -> u8 {
		Metadata::<T>::get(asset).decimals
	}
}

impl<T: Config> fungibles::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn mint_into(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		Self::do_mint(asset, who, amount, None)
	}

	fn burn_from(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		let f = DebitFlags { keep_alive: false, best_effort: false };
		Self::do_burn(asset, who, amount, None, f)
	}

	fn slash(
		asset: Self::AssetId,
		who: &<T as SystemConfig>::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		let f = DebitFlags { keep_alive: false, best_effort: true };
		Self::do_burn(asset, who, amount, None, f)
	}
}

impl<T: Config> fungibles::Transfer<T::AccountId> for Pallet<T> {
	fn transfer(
		asset: Self::AssetId,
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: T::Balance,
		keep_alive: bool,
	) -> Result<T::Balance, DispatchError> {
		let f = TransferFlags { keep_alive, best_effort: false, burn_dust: false };
		Self::do_transfer(asset, source, dest, amount, None, f)
	}
}

impl<T: Config> fungibles::Unbalanced<T::AccountId> for Pallet<T> {
	fn set_balance(_: Self::AssetId, _: &T::AccountId, _: Self::Balance) -> DispatchResult {
		unreachable!("set_balance is not used if other functions are impl'd");
	}
	fn set_total_issuance(id: T::AssetId, amount: Self::Balance) {
		Asset::<T>::mutate_exists(id, |maybe_asset| {
			if let Some(ref mut asset) = maybe_asset {
				asset.supply = amount
			}
		});
	}
	fn decrease_balance(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		let f = DebitFlags { keep_alive: false, best_effort: false };
		Self::decrease_balance(asset, who, amount, f, |_, _| Ok(()))
	}
	fn decrease_balance_at_most(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Self::Balance {
		let f = DebitFlags { keep_alive: false, best_effort: true };
		Self::decrease_balance(asset, who, amount, f, |_, _| Ok(())).unwrap_or(Zero::zero())
	}
	fn increase_balance(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Self::increase_balance(asset, who, amount, |_| Ok(()))?;
		Ok(amount)
	}
	fn increase_balance_at_most(
		asset: T::AssetId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Self::Balance {
		match Self::increase_balance(asset, who, amount, |_| Ok(())) {
			Ok(()) => amount,
			Err(_) => Zero::zero(),
		}
	}
}

impl<T: Config> fungibles::Create<T::AccountId> for Pallet<T> {
	fn create(
		id: T::AssetId,
		admin: T::AccountId,
		is_sufficient: bool,
		min_balance: Self::Balance,
	) -> DispatchResult {
		Self::do_force_create(id, admin, is_sufficient, min_balance)
	}
}

impl<T: Config> fungibles::Destroy<T::AccountId> for Pallet<T> {
	fn start_destroy(id: T::AssetId, maybe_check_owner: Option<T::AccountId>) -> DispatchResult {
		Self::do_start_destroy(id, maybe_check_owner)
	}

	fn destroy_accounts(id: T::AssetId, max_items: u32) -> Result<u32, DispatchError> {
		Self::do_destroy_accounts(id, max_items)
	}

	fn destroy_approvals(id: T::AssetId, max_items: u32) -> Result<u32, DispatchError> {
		Self::do_destroy_approvals(id, max_items)
	}

	fn finish_destroy(id: T::AssetId) -> DispatchResult {
		Self::do_finish_destroy(id)
	}
}

impl<T: Config> fungibles::metadata::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn name(asset: T::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).name.to_vec()
	}

	fn symbol(asset: T::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset).symbol.to_vec()
	}

	fn decimals(asset: T::AssetId) -> u8 {
		Metadata::<T>::get(asset).decimals
	}
}

impl<T: Config> fungibles::metadata::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn set(
		asset: T::AssetId,
		from: &<T as SystemConfig>::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		Self::do_set_metadata(asset, from, name, symbol, decimals)
	}
}

impl<T: Config> fungibles::approvals::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	// Check the amount approved to be spent by an owner to a delegate
	fn allowance(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
	) -> T::Balance {
		Approvals::<T>::get((asset, &owner, &delegate))
			.map(|x| x.amount)
			.unwrap_or_else(Zero::zero)
	}
}

impl<T: Config> fungibles::approvals::Mutate<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn approve(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::do_approve_transfer(asset, owner, delegate, amount)
	}

	// Aprove spending tokens from a given account
	fn transfer_from(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
		dest: &<T as SystemConfig>::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::do_transfer_approved(asset, owner, delegate, dest, amount)
	}
}

impl<T: Config> fungibles::roles::Inspect<<T as SystemConfig>::AccountId> for Pallet<T> {
	fn owner(asset: T::AssetId) -> Option<<T as SystemConfig>::AccountId> {
		Asset::<T>::get(asset).map(|x| x.owner)
	}

	fn issuer(asset: T::AssetId) -> Option<<T as SystemConfig>::AccountId> {
		Asset::<T>::get(asset).map(|x| x.issuer)
	}

	fn admin(asset: T::AssetId) -> Option<<T as SystemConfig>::AccountId> {
		Asset::<T>::get(asset).map(|x| x.admin)
	}

	fn freezer(_asset: T::AssetId) -> Option<<T as SystemConfig>::AccountId> {
		None
	}
}

impl<T: Config> fungibles::InspectEnumerable<T::AccountId> for Pallet<T> {
	type AssetsIterator = KeyPrefixIterator<<T as Config>::AssetId>;

	/// Returns an iterator of the assets in existence.
	///
	/// NOTE: iterating this list invokes a storage read per item.
	fn asset_ids() -> Self::AssetsIterator {
		Asset::<T>::iter_keys()
	}
}
