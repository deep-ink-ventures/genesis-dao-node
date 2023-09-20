//! Functions for the Assets pallet.

use super::*;
use commons::traits::pallets::{ActiveProposals, UsableCheckpoints};
use frame_support::{traits::Get, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::{borrow::Borrow, fmt::Debug};

// The main implementation block for the module.
impl<T: Config> Pallet<T> {
	/// Get DaoId
	pub fn dao_id(asset_id: &T::AssetId) -> Vec<u8> {
		Metadata::<T>::get(asset_id).symbol.to_vec()
	}

	/// Get the asset `id` free balance of `who`, or zero if the asset-account doesn't exist.
	pub fn balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_balance(id, who).unwrap_or_default()
	}

	/// Get the asset `id` total (including reserved) balance of `who`,
	/// or zero if the asset-account doesn't exist.
	pub fn total_balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_total_balance(id, who).unwrap_or_default()
	}

	/// Get the asset `id` reserved balance of `who`, or zero if the asset-account doesn't exist.
	pub fn reserved(id: T::AssetId, who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::maybe_reserved(id, who).unwrap_or_default()
	}

	/// Get the asset `id` free balance of `who` if the asset-account exists.
	pub fn maybe_balance(id: T::AssetId, who: impl Borrow<T::AccountId>) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.balance)
	}

	/// Get the asset `id` total (including reserved) balance of `who` if the asset-account exists.
	pub fn maybe_total_balance(
		id: T::AssetId,
		who: impl Borrow<T::AccountId>,
	) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.balance + a.reserved)
	}

	/// Get the asset `id` reserved balance of `who` if the asset-account exists.
	pub fn maybe_reserved(id: T::AssetId, who: impl Borrow<T::AccountId>) -> Option<T::Balance> {
		Account::<T>::get(id, who.borrow()).map(|a| a.reserved)
	}

	/// Get the total supply of an asset `id`.
	pub fn total_supply(id: T::AssetId) -> T::Balance {
		Self::maybe_total_supply(id).unwrap_or_default()
	}

	/// Get the total supply of an asset `id` if the asset exists.
	pub fn maybe_total_supply(id: T::AssetId) -> Option<T::Balance> {
		Asset::<T>::get(id).map(|x| x.supply)
	}

	/// Get the total historical supply of an asset `id` at a certain `block`.
	/// Result may be None, if the age of the requested block is at or beyond
	/// the HistoryHorizon and history has been removed.
	pub fn total_historical_supply(id: T::AssetId, block: BlockNumberFor<T>) -> Option<T::Balance> {
		Self::search_history(SupplyHistory::<T>::get(id), block)
	}

	/// Remove the whole account history under this assetId
	/// This is mainly used when this account is to be dead
	pub fn remove_account_history(asset_id: T::AssetId, account: &T::AccountId) {
		let limit = T::ActiveProposals::max_proposals_limit() + 1;
		let result = AccountHistory::<T>::clear_prefix((asset_id, account), limit, None);

		// By this we except that all history of this pair will be removed
		// but since clear_prefix does not provide this gurantee
		if result.maybe_cursor.is_some() {
			// this case should not happen in idea case but
			// just in case. we write the account history to 0
			let block_num = frame_system::Pallet::<T>::block_number();
			AccountHistory::<T>::insert((asset_id, &account), block_num, CheckpointOf::<T>::zero());
		}
	}

	/// Action to perfrom when any call is made that
	/// changes the asset balance of involved account
	pub fn mutate_account(
		asset_id: T::AssetId,
		who: impl Borrow<T::AccountId>,
		balance: T::Balance,
	) {
		let current_block = frame_system::Pallet::<T>::block_number();
		let dao_id = Self::dao_id(&asset_id);

		// get all proposals
		let proposal_start_dates = <T::ActiveProposals as ActiveProposals::<BlockNumberFor<T>>>::active_proposals_starting_time(dao_id, current_block);
		// get all checkpoints
		let (mut checkpoint_blocks, (_last_chp_block, last_chp)) =
			Self::get_checkpoint_blocks(&asset_id, who.borrow());
		// also include checkpoint we are processing
		checkpoint_blocks.push(current_block);

		// Remove unused checkpoints
		Self::remove_unused_checkpoint(
			&asset_id,
			&proposal_start_dates,
			&checkpoint_blocks,
			who.borrow(),
		);

		// Finally insert
		let checkpoint = CheckpointOf::<T> { mutated: balance, ..last_chp };
		AccountHistory::<T>::insert((asset_id, who.borrow()), current_block, checkpoint);
	}

	/// Remove unused checkpoints that are not associated with any proposals
	fn remove_unused_checkpoint(
		asset_id: &T::AssetId,
		proposal_starts: &Vec<BlockNumberFor<T>>,
		checkpoint_blocks: &Vec<BlockNumberFor<T>>,
		who: &T::AccountId,
	) {
		// Get only checkpoints that are associated with proposals
		let mut usable_checkpoints =
			Self::proposal_checkpoint_pair(proposal_starts, checkpoint_blocks)
				.into_iter()
				.map(|(_prop, ch)| ch)
				.collect::<Vec<_>>();
		// We keep the one inserted in this block
		usable_checkpoints.push(frame_system::Pallet::<T>::block_number());

		// remove those that are not
		for ch in checkpoint_blocks {
			if !usable_checkpoints.contains(&ch) {
				AccountHistory::<T>::remove((asset_id, who.borrow()), ch);
			}
		}
	}

	/// Get the total historical balance of an asset `id` at a certain `block` for an account `who`.
	pub fn total_historical_balance(
		asset_id: T::AssetId,
		who: impl Borrow<T::AccountId>,
		block: BlockNumberFor<T>,
	) -> (BlockNumberFor<T>, AssetBalanceOf<T>) {
		let mut latest = (Zero::zero(), Zero::zero());

		for (block_num, chp) in AccountHistory::<T>::iter_prefix((asset_id, who.borrow()))
			.filter(|(bl_num, chp)| bl_num >= &block)
		{
			if block_num > latest.0 {
				let amount = chp.mutated.saturating_add(*chp.delegated_amount());
				latest = (block_num, amount);
			}
		}

		latest
	}

	/// Search a history for the value at a specific block.
	/// Result may be None, if the age of the requested block is at or beyond
	/// the HistoryHorizon and history has been removed.
	fn search_history<V: Copy + Zero, B: Get<u32>>(
		history: Option<BoundedBTreeMap<BlockNumberFor<T>, V, B>>,
		block: BlockNumberFor<T>,
	) -> Option<V> {
		let default = || {
			let current_block = frame_system::Pallet::<T>::block_number();
			if current_block - block < T::HistoryHorizon::get().into() {
				Some(Zero::zero())
			} else {
				None
			}
		};
		history.map_or_else(default, |history| {
			history.range(..=block).next_back().map(|item| *item.1).or_else(default)
		})
	}

	pub(super) fn update_supply_history(id: T::AssetId, supply: T::Balance) {
		// update history
		let history = Self::update_history(SupplyHistory::<T>::get(id).unwrap_or_default(), supply);

		// record new history
		SupplyHistory::<T>::insert(id, history);
	}

	pub(super) fn update_account_history(id: T::AssetId, who: &T::AccountId, balance: T::Balance) {
		Self::mutate_account(id, who, balance);
	}

	fn update_history<V: Copy + Debug + Zero, B: Get<u32>>(
		mut history: BoundedBTreeMap<BlockNumberFor<T>, V, B>,
		value: V,
	) -> BoundedBTreeMap<BlockNumberFor<T>, V, B> {
		let current_block = frame_system::Pallet::<T>::block_number();

		// the oldest block for which we need to be able to retrieve history
		let inner_horizon_block =
			current_block.saturating_sub((T::HistoryHorizon::get() - 1).into());

		// if there is enough history, find block that has history for inner_horizon_block
		if let Some(block) = history.range(..=inner_horizon_block).next_back().map(|i| *i.0) {
			// and remove everything older than that block
			history = BoundedBTreeMap::try_from(history.into_inner().split_off(&block)).unwrap();
		}

		// insert new history item
		history.try_insert(current_block, value).expect("Enough space has been freed");
		history
	}

	pub(super) fn new_account(
		who: &T::AccountId,
		d: &mut AssetDetailsOf<T>,
	) -> Result<AssetAccountOf<T>, DispatchError> {
		d.accounts = d.accounts.checked_add(1).ok_or(ArithmeticError::Overflow)?;
		frame_system::Pallet::<T>::inc_providers(who);
		Ok(AssetAccountOf::<T> { balance: Zero::zero(), reserved: Zero::zero() })
	}

	pub(super) fn dead_account(
		_id: T::AssetId,
		who: &T::AccountId,
		details: &mut AssetDetailsOf<T>,
	) {
		let _ = frame_system::Pallet::<T>::dec_providers(who);
		details.accounts.saturating_dec();
	}

	/// Returns `true` when the balance of `account` can be increased by `amount`.
	///
	/// - `id`: The id of the asset that should be increased.
	/// - `who`: The account of which the balance should be increased.
	/// - `amount`: The amount by which the balance should be increased.
	/// - `increase_supply`: Will the supply of the asset be increased by `amount` at the same time
	///   as crediting the `account`.
	pub(super) fn can_increase(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		increase_supply: bool,
	) -> DepositConsequence {
		use DepositConsequence::*;
		let details = match Asset::<T>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};
		// check for supply overflow
		if increase_supply && details.supply.checked_add(&amount).is_none() {
			return Overflow
		}

		match Self::maybe_balance(id, who) {
			// check for balance overflow
			Some(balance) if balance.checked_add(&amount).is_none() => Overflow,
			None if amount < details.min_balance => BelowMinimum,
			_ => Success,
		}
	}

	/// Return the consequence of a withdraw.
	pub(super) fn can_decrease(
		id: T::AssetId,
		who: &T::AccountId,
		amount: T::Balance,
		keep_alive: bool,
	) -> WithdrawConsequence<T::Balance> {
		use WithdrawConsequence::*;
		let details = match Asset::<T>::get(id) {
			Some(details) => details,
			None => return UnknownAsset,
		};
		if details.supply.checked_sub(&amount).is_none() {
			return Underflow
		}
		if amount.is_zero() {
			return Success
		}
		let account = match Account::<T>::get(id, who) {
			Some(a) => a,
			None => return Underflow,
		};
		if let Some(rest) = account.balance.checked_sub(&amount) {
			if rest < details.min_balance {
				if keep_alive {
					WouldDie
				} else {
					ReducedToZero(rest)
				}
			} else {
				Success
			}
		} else {
			Underflow
		}
	}

	// Maximum `amount` that can be passed into `can_withdraw` to result in a `WithdrawConsequence`
	// of `Success`.
	pub(super) fn reducible_balance(
		id: T::AssetId,
		who: &T::AccountId,
		keep_alive: bool,
	) -> Result<T::Balance, DispatchError> {
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let account = Account::<T>::get(id, who).ok_or(Error::<T>::NoAccount)?;
		Ok(if keep_alive {
			account.balance.saturating_sub(details.min_balance)
		} else {
			account.balance
		})
	}

	/// Make preparatory checks for debiting some funds from an account. Flags indicate requirements
	/// of the debit.
	///
	/// - `amount`: The amount desired to be debited. The actual amount returned for debit may be
	///   less (in the case of `best_effort` being `true`) or greater by up to the minimum balance
	///   less one.
	/// - `keep_alive`: Require that `target` must stay alive.
	/// - `best_effort`: The debit amount may be less than `amount`.
	///
	/// On success, the amount which should be debited (this will always be at least `amount` unless
	/// `best_effort` is `true`).
	///
	/// If no valid debit can be made then return an `Err`.
	pub(super) fn prep_debit(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
	) -> Result<T::Balance, DispatchError> {
		let actual = Self::reducible_balance(id, target, f.keep_alive)?.min(amount);
		ensure!(f.best_effort || actual >= amount, Error::<T>::BalanceLow);

		let conseq = Self::can_decrease(id, target, actual, f.keep_alive);
		let actual = match conseq.into_result(f.keep_alive) {
			Ok(dust) => actual.saturating_add(dust), //< guaranteed by reducible_balance
			Err(e) => {
				debug_assert!(false, "passed from reducible_balance; qed");
				return Err(e)
			},
		};

		Ok(actual)
	}

	/// Make preparatory checks for crediting some funds from an account. Flags indicate
	/// requirements of the credit.
	///
	/// - `amount`: The amount desired to be credited.
	/// - `debit`: The amount by which some other account has been debited. If this is greater than
	///   `amount`, then the `burn_dust` parameter takes effect.
	/// - `burn_dust`: Indicates that in the case of debit being greater than amount, the additional
	///   (dust) value should be burned, rather than credited.
	///
	/// On success, the amount which should be credited (this will always be at least `amount`)
	/// together with an optional value indicating the value which should be burned. The latter
	/// will always be `None` as long as `burn_dust` is `false` or `debit` is no greater than
	/// `amount`.
	///
	/// If no valid credit can be made then return an `Err`.
	pub(super) fn prep_credit(
		id: T::AssetId,
		dest: &T::AccountId,
		amount: T::Balance,
		debit: T::Balance,
		burn_dust: bool,
	) -> Result<(T::Balance, Option<T::Balance>), DispatchError> {
		let (credit, maybe_burn) = match (burn_dust, debit.checked_sub(&amount)) {
			(true, Some(dust)) => (amount, Some(dust)),
			_ => (debit, None),
		};
		Self::can_increase(id, dest, credit, false).into_result()?;
		Ok((credit, maybe_burn))
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub fn do_mint(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		Self::increase_balance(id, beneficiary, amount, |details| -> DispatchResult {
			debug_assert!(
				T::Balance::max_value() - details.supply >= amount,
				"checked in prep; qed"
			);
			details.supply.saturating_accrue(amount);

			Self::update_supply_history(id, details.supply);

			Ok(())
		})?;
		Self::mutate_account(id, beneficiary, amount);
		Self::deposit_event(Event::Issued {
			asset_id: id,
			owner: beneficiary.clone(),
			total_supply: amount,
		});
		Ok(())
	}

	/// Increases the asset `id` balance of `beneficiary` by `amount`.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_mint` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error or will increase the amount by exactly `amount`.
	pub(super) fn increase_balance(
		id: T::AssetId,
		beneficiary: &T::AccountId,
		amount: T::Balance,
		check: impl FnOnce(&mut AssetDetailsOf<T>) -> DispatchResult,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}

		Self::can_increase(id, beneficiary, amount, true).into_result()?;
		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);
			check(details)?;

			let mut account = Account::<T>::try_get(id, beneficiary)
				.or_else(|_| Self::new_account(beneficiary, details))?;
			account.balance.saturating_accrue(amount);
			ensure!(account.balance >= details.min_balance, TokenError::BelowMinimum);
			Self::update_account_history(id, beneficiary, account.balance + account.reserved);
			Account::<T>::insert(id, beneficiary, account);
			Ok(())
		})
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// This alters the registered supply of the asset and emits an event.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	#[cfg(test)]
	pub(super) fn do_burn(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
	) -> Result<T::Balance, DispatchError> {
		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let actual = Self::decrease_balance(id, target, amount, f, |actual, details| {
			debug_assert!(details.supply >= actual, "checked in prep; qed");
			details.supply.saturating_reduce(actual);

			Self::update_supply_history(id, details.supply);

			Ok(())
		})?;
		Self::deposit_event(Event::Burned { asset_id: id, owner: target.clone(), balance: actual });
		Ok(actual)
	}

	/// Reduces asset `id` balance of `target` by `amount`. Flags `f` can be given to alter whether
	/// it attempts a `best_effort` or makes sure to `keep_alive` the account.
	///
	/// LOW-LEVEL: Does not alter the supply of asset or emit an event. Use `do_burn` if you need
	/// that. This is not intended to be used alone.
	///
	/// Will return an error and do nothing or will decrease the amount and return the amount
	/// reduced by.
	pub(super) fn decrease_balance(
		id: T::AssetId,
		target: &T::AccountId,
		amount: T::Balance,
		f: DebitFlags,
		check: impl FnOnce(T::Balance, &mut AssetDetailsOf<T>) -> DispatchResult,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let actual = Self::prep_debit(id, target, amount, f)?;

		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			check(actual, details)?;

			let mut account = Account::<T>::take(id, target).ok_or(Error::<T>::NoAccount)?;
			debug_assert!(account.balance >= actual, "checked in prep; qed");
			account.balance.saturating_reduce(actual);
			if account.balance < details.min_balance {
				// account already removed by take
				Self::dead_account(id, target, details);
				debug_assert!(account.balance.is_zero(), "checked in prep; qed");
				return Ok(())
			};
			Self::update_account_history(id, target, account.balance + account.reserved);
			Account::<T>::insert(id, target, account);
			Ok(())
		})?;

		Ok(actual)
	}

	/// Reserves some `amount` of asset `id` balance of `target`.
	pub fn do_reserve(
		id: T::AssetId,
		target: impl Borrow<T::AccountId>,
		amount: T::Balance,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		let f = DebitFlags { keep_alive: true, best_effort: false };

		let actual = Self::prep_debit(id, target.borrow(), amount, f)?;

		Account::<T>::try_mutate(id, target.borrow(), |maybe_account| -> DispatchResult {
			let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;
			debug_assert!(account.balance >= actual, "checked in prep; qed");

			// Make the reservation.
			account.balance.saturating_reduce(actual);
			account.reserved.saturating_accrue(actual);
			*maybe_account = Some(account);
			Ok(())
		})?;

		Ok(actual)
	}

	/// Unreserves some `amount` of asset `id` balance of `target`.
	/// If `amount` is greater than reserved balance, then the whole reserved balance is unreserved.
	pub fn do_unreserve(
		id: T::AssetId,
		target: impl Borrow<T::AccountId>,
		mut amount: T::Balance,
	) -> Result<T::Balance, DispatchError> {
		if amount.is_zero() {
			return Ok(amount)
		}

		// check asset is live
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		Account::<T>::try_mutate(id, target.borrow(), |maybe_account| -> DispatchResult {
			let mut account = maybe_account.take().ok_or(Error::<T>::NoAccount)?;

			// Unreserve the minimum of amount and reserved balance
			amount = amount.min(account.reserved);
			account.balance = account.balance.saturating_add(amount);
			account.reserved = account.reserved.saturating_sub(amount);
			*maybe_account = Some(account);
			Ok(())
		})?;

		Ok(amount)
	}

	/// Reduces the asset `id` balance of `source` by some `amount` and increases the balance of
	/// `dest` by (similar) amount.
	///
	/// Returns the actual amount placed into `dest`. Exact semantics are determined by the flags
	/// `f`.
	///
	/// Will fail if the amount transferred is so small that it cannot create the destination due
	/// to minimum balance requirements.
	pub(super) fn do_transfer(
		id: T::AssetId,
		source: &T::AccountId,
		dest: &T::AccountId,
		amount: T::Balance,
		f: TransferFlags,
	) -> Result<T::Balance, DispatchError> {
		// Early exit if no-op.
		if amount.is_zero() {
			return Ok(amount)
		}
		let details = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		// Figure out the debit and credit, together with side-effects.
		let debit = Self::prep_debit(id, source, amount, f.into())?;
		let (credit, maybe_burn) = Self::prep_credit(id, dest, amount, debit, f.burn_dust)?;

		Asset::<T>::try_mutate(id, |maybe_details| -> DispatchResult {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Skip if source == dest
			if source == dest {
				return Ok(())
			}

			// Burn any dust if needed.
			if let Some(burn) = maybe_burn {
				// Debit dust from supply; this will not saturate since it's already checked in
				// prep.
				debug_assert!(details.supply >= burn, "checked in prep; qed");
				details.supply = details.supply.saturating_sub(burn);
			}

			// Debit balance from source; this will not saturate since it's already checked in prep.
			let mut source_account = Account::<T>::get(id, source).expect("checked in prep; qed");
			debug_assert!(source_account.balance >= debit, "checked in prep; qed");
			source_account.balance.saturating_reduce(debit);

			let mut account =
				Account::<T>::try_get(id, dest).or_else(|_| Self::new_account(dest, details))?;
			// Calculate new balance; this will not saturate since it's already checked in prep.
			debug_assert!(account.balance.checked_add(&credit).is_some(), "checked in prep; qed");
			account.balance.saturating_accrue(credit);
			Self::update_account_history(id, dest, account.balance + account.reserved);
			Account::<T>::insert(id, dest, account);

			// Remove source account if it's now dead.
			if source_account.balance < details.min_balance {
				debug_assert!(source_account.balance.is_zero(), "checked in prep; qed");
				Self::dead_account(id, source, details);
				Account::<T>::remove(id, source);
				return Ok(())
			}
			Self::update_account_history(
				id,
				source,
				source_account.balance + source_account.reserved,
			);
			Account::<T>::insert(id, source, &source_account);
			Ok(())
		})?;

		Self::deposit_event(Event::Transferred {
			asset_id: id,
			from: source.clone(),
			to: dest.clone(),
			amount: credit,
		});
		Ok(credit)
	}

	/// Create a new asset without taking a deposit.
	///
	/// * `id`: The `AssetId` you want the new asset to have. Must not already be in use.
	/// * `owner`: The owner of this asset upon creation.
	/// * `min_balance`: The minimum balance a user is allowed to have of this asset before they are
	///   considered dust and cleaned up.
	pub fn do_force_create(
		id: T::AssetId,
		owner: T::AccountId,
		min_balance: T::Balance,
	) -> DispatchResult {
		ensure!(!Asset::<T>::contains_key(id), Error::<T>::InUse);
		ensure!(!min_balance.is_zero(), Error::<T>::MinBalanceZero);

		Asset::<T>::insert(
			id,
			AssetDetails {
				owner: owner.clone(),
				supply: Zero::zero(), // no need to record a supply of zero in the SupplyHistory
				min_balance,
				accounts: 0,
				approvals: 0,
				status: AssetStatus::Live,
			},
		);

		Self::deposit_event(Event::ForceCreated { asset_id: id, owner });
		Ok(())
	}

	/// Start the process of destroying an asset, by setting the asset status to `Destroying`, and
	/// emitting the `DestructionStarted` event.
	pub(super) fn do_start_destroy(
		id: T::AssetId,
		maybe_check_owner: Option<T::AccountId>,
	) -> DispatchResult {
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			if let Some(check_owner) = maybe_check_owner {
				ensure!(details.owner == check_owner, Error::<T>::NoPermission);
			}
			details.status = AssetStatus::Destroying;
			SupplyHistory::<T>::remove(id);

			Self::deposit_event(Event::DestructionStarted { asset_id: id });
			Ok(())
		})
	}

	/// Destroy accounts associated with a given asset up to the max (T::RemoveItemsLimit).
	///
	/// Each call emits the `Event::AccountsDestroyed` event.
	/// Returns the number of destroyed accounts.
	pub(super) fn do_destroy_accounts(
		id: T::AssetId,
		max_items: u32,
	) -> Result<u32, DispatchError> {
		let mut dead_accounts = 0;
		let mut remaining_accounts = 0;
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Should only destroy accounts while the asset is in a destroying state
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);

			for (who, _) in Account::<T>::drain_prefix(id).take(max_items as usize) {
				// account already removed by drain
				Self::dead_account(id, &who, details);
				dead_accounts += 1;

				// todo: weather to remove the history or rewrite to 0?
				Self::remove_account_history(id, &who);
			}
			remaining_accounts = details.accounts;
			Ok(())
		})?;

		Self::deposit_event(Event::AccountsDestroyed {
			asset_id: id,
			accounts_destroyed: dead_accounts,
			accounts_remaining: remaining_accounts,
		});
		Ok(dead_accounts)
	}

	/// Destroy approvals associated with a given asset up to the max (T::RemoveItemsLimit).
	///
	/// Each call emits the `Event::ApprovalsDestroyed` event
	/// Returns the number of destroyed approvals.
	pub(super) fn do_destroy_approvals(
		id: T::AssetId,
		max_items: u32,
	) -> Result<u32, DispatchError> {
		let mut removed_approvals = 0;
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;

			// Should only destroy accounts while the asset is in a destroying state.
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);

			for ((owner, _), approval) in Approvals::<T>::drain_prefix((id,)) {
				T::Currency::unreserve(&owner, approval.deposit);
				removed_approvals = removed_approvals.saturating_add(1);
				details.approvals = details.approvals.saturating_sub(1);
				if removed_approvals >= max_items {
					break
				}
			}
			Self::deposit_event(Event::ApprovalsDestroyed {
				asset_id: id,
				approvals_destroyed: removed_approvals,
				approvals_remaining: details.approvals,
			});
			Ok(())
		})?;
		Ok(removed_approvals)
	}

	/// Complete destroying an asset and unreserve the deposit.
	///
	/// On success, the `Event::Destroyed` event is emitted.
	pub(super) fn do_finish_destroy(id: T::AssetId) -> DispatchResult {
		Asset::<T>::try_mutate_exists(id, |maybe_details| -> Result<(), DispatchError> {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(details.status == AssetStatus::Destroying, Error::<T>::IncorrectStatus);
			ensure!(details.accounts == 0, Error::<T>::InUse);
			ensure!(details.approvals == 0, Error::<T>::InUse);

			let _ = Metadata::<T>::take(id); // erase metadata
			details.status = AssetStatus::Destroyed;
			Self::deposit_event(Event::Destroyed { asset_id: id });
			Ok(())
		})
	}

	/// Creates an approval from `owner` to spend `amount` of asset `id` tokens by 'delegate'
	/// while reserving `T::ApprovalDeposit` from owner
	///
	/// If an approval already exists, the new amount is added to such existing approval
	pub(super) fn do_approve_transfer(
		id: T::AssetId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		let mut d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		Approvals::<T>::try_mutate((id, &owner, &delegate), |maybe_approved| -> DispatchResult {
			let mut approved = match maybe_approved.take() {
				// an approval already exists and is being updated
				Some(a) => a,
				// a new approval is created
				None => {
					d.approvals.saturating_inc();
					Default::default()
				},
			};
			let deposit_required = T::ApprovalDeposit::get();
			if approved.deposit < deposit_required {
				T::Currency::reserve(owner, deposit_required - approved.deposit)?;
				approved.deposit = deposit_required;
			}
			approved.amount = approved.amount.saturating_add(amount);
			*maybe_approved = Some(approved);
			Ok(())
		})?;
		Asset::<T>::insert(id, d);
		Self::deposit_event(Event::ApprovedTransfer {
			asset_id: id,
			source: owner.clone(),
			delegate: delegate.clone(),
			amount,
		});

		Ok(())
	}

	/// Reduces the asset `id` balance of `owner` by some `amount` and increases the balance of
	/// `dest` by (similar) amount, checking that 'delegate' has an existing approval from `owner`
	/// to spend`amount`.
	///
	/// Will fail if `amount` is greater than the approval from `owner` to 'delegate'
	/// Will unreserve the deposit from `owner` if the entire approved `amount` is spent by
	/// 'delegate'
	pub(super) fn do_transfer_approved(
		id: T::AssetId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
		destination: &T::AccountId,
		amount: T::Balance,
	) -> DispatchResult {
		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);

		Approvals::<T>::try_mutate_exists(
			(id, &owner, delegate),
			|maybe_approved| -> DispatchResult {
				let mut approved = maybe_approved.take().ok_or(Error::<T>::Unapproved)?;
				let remaining =
					approved.amount.checked_sub(&amount).ok_or(Error::<T>::Unapproved)?;

				let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
				Self::do_transfer(id, owner, destination, amount, f)?;

				if remaining.is_zero() {
					T::Currency::unreserve(owner, approved.deposit);
					Asset::<T>::mutate(id, |maybe_details| {
						if let Some(details) = maybe_details {
							details.approvals.saturating_dec();
						}
					});
				} else {
					approved.amount = remaining;
					*maybe_approved = Some(approved);
				}
				Ok(())
			},
		)?;

		Ok(())
	}

	/// Do set metadata
	pub fn do_set_metadata(
		id: T::AssetId,
		from: &T::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, T::StringLimit> =
			name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
		let bounded_symbol: BoundedVec<u8, T::StringLimit> =
			symbol.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

		let d = Asset::<T>::get(id).ok_or(Error::<T>::Unknown)?;
		ensure!(d.status == AssetStatus::Live, Error::<T>::AssetNotLive);
		ensure!(from == &d.owner, Error::<T>::NoPermission);

		Metadata::<T>::try_mutate_exists(id, |metadata| {
			*metadata =
				Some(AssetMetadata { name: bounded_name, symbol: bounded_symbol, decimals });

			Self::deposit_event(Event::MetadataSet { asset_id: id, name, symbol, decimals });
			Ok(())
		})
	}

	pub fn change_owner(id: T::AssetId, new_owner: T::AccountId) -> DispatchResult {
		Asset::<T>::try_mutate(id, |maybe_details| {
			let details = maybe_details.as_mut().ok_or(Error::<T>::Unknown)?;
			ensure!(details.status == AssetStatus::Live, Error::<T>::AssetNotLive);
			details.owner = new_owner;
			Ok(())
		})
	}

	pub fn allowance(
		asset: T::AssetId,
		owner: &<T as SystemConfig>::AccountId,
		delegate: &<T as SystemConfig>::AccountId,
	) -> T::Balance {
		Approvals::<T>::get((asset, &owner, &delegate))
			.map(|x| x.amount)
			.unwrap_or_else(Zero::zero)
	}

	/// Return all block numbere where checkpoint of this asset_id,account pair is stored
	/// Also return the latest one
	pub fn get_checkpoint_blocks(
		asset_id: &T::AssetId,
		who: &T::AccountId,
	) -> (Vec<BlockNumberFor<T>>, (BlockNumberFor<T>, CheckpointOf<T>)) {
		let mut latest = (Zero::zero(), CheckpointOf::<T>::zero());
		let mut blocks = vec![];

		for (block_num, chp) in AccountHistory::<T>::iter_prefix((asset_id, who)) {
			blocks.push(block_num);
			if block_num > latest.0 {
				latest = (block_num, chp)
			}
		}

		(blocks, latest)
	}

	/// Delegate source's balance to target's
	pub fn do_delegate(
		asset_id: &T::AssetId,
		source: &T::AccountId,
		target: &T::AccountId,
		is_revoke: bool,
	) -> DispatchResult {
		// get source's latest checkpoint
		// get target's latest checkpoint
		let current_block = frame_system::Pallet::<T>::block_number();
		let (mut source_checkpoints, (_src_bl, mut src_chp)) =
			Self::get_checkpoint_blocks(asset_id, source);
		let (mut target_checkpoints, (_tg_bl, mut tg_chp)) =
			Self::get_checkpoint_blocks(asset_id, target);

		if is_revoke {
			src_chp.revoke_delegation(source, &mut tg_chp);
		} else {
			tg_chp.delegate_to(target, &mut src_chp).ok_or(Error::<T>::DelegationLimit)?;
		}

		AccountHistory::<T>::insert((asset_id, source), current_block, src_chp);
		AccountHistory::<T>::insert((asset_id, target), current_block, tg_chp);

		let dao_id = Self::dao_id(asset_id);
		let proposal_start_dates =
			T::ActiveProposals::active_proposals_starting_time(dao_id, current_block);
		source_checkpoints.push(current_block);
		target_checkpoints.push(current_block);
		Self::remove_unused_checkpoint(
			&asset_id,
			&proposal_start_dates,
			&source_checkpoints,
			&source,
		);
		Self::remove_unused_checkpoint(
			&asset_id,
			&proposal_start_dates,
			&target_checkpoints,
			&target,
		);

		Ok(())
	}
}

impl<T: Config> commons::traits::pallets::AssetInterface for Pallet<T> {
	type AssetId = pallet_dao_core::AssetIdOf<T>;
	type AssetInfo = AssetDetails<AssetBalanceOf<T>, AccountIdOf<T>>;
	type AccountId = AccountIdOf<T>;
	type Balance = AssetBalanceOf<T>;
	type BlockNumber = BlockNumberFor<T>;

	fn get_asset(asset_id: Self::AssetId) -> Option<Self::AssetInfo> {
		Asset::<T>::get(asset_id)
	}

	fn mint(
		id: Self::AssetId,
		beneficiary: &Self::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		Pallet::<T>::do_mint(id, beneficiary, amount)
	}

	fn force_create(
		id: Self::AssetId,
		owner: Self::AccountId,
		min_balance: Self::Balance,
	) -> DispatchResult {
		Pallet::<T>::do_force_create(id, owner, min_balance)
	}

	fn set_metadata(
		id: Self::AssetId,
		from: &Self::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult {
		Pallet::<T>::do_set_metadata(id, from, name, symbol, decimals)
	}

	fn change_owner(id: Self::AssetId, new_owner: Self::AccountId) -> DispatchResult {
		Pallet::<T>::change_owner(id, new_owner)
	}

	fn reserve(
		id: Self::AssetId,
		target: impl Borrow<Self::AccountId>,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		Pallet::<T>::do_reserve(id, target, amount)
	}

	fn total_historical_supply(
		id: Self::AssetId,
		block: Self::BlockNumber,
	) -> Option<Self::Balance> {
		Pallet::<T>::total_historical_supply(id, block)
	}

	fn total_historical_balance(
		id: Self::AssetId,
		who: impl Borrow<Self::AccountId>,
		block: Self::BlockNumber,
	) -> Self::Balance {
		Pallet::<T>::total_historical_balance(id, who, block).1
	}

	fn delegate(
		id: impl Borrow<Self::AssetId>,
		from: impl Borrow<Self::AccountId>,
		to: impl Borrow<Self::AccountId>,
	) -> DispatchResult {
		Self::do_delegate(id.borrow(), from.borrow(), to.borrow(), false)
	}

	fn revoke_delegation(
		id: impl Borrow<Self::AssetId>,
		revoke_from: impl Borrow<Self::AccountId>,
		revert_to: impl Borrow<Self::AccountId>,
	) -> DispatchResult {
		Self::do_delegate(id.borrow(), revoke_from.borrow(), revert_to.borrow(), true)
	}
}

impl<T: Config> UsableCheckpoints for Pallet<T> {
	type BlockNumber = BlockNumberFor<T>;
	type BlockIter = Vec<Self::BlockNumber>;
	type Res = Vec<(Self::BlockNumber, Self::BlockNumber)>;

	fn proposal_checkpoint_pair(
		// where porposals starts
		proposals_starts: impl Borrow<Self::BlockIter>,
		// where checkpoint are made
		checkpoint_blocks: impl Borrow<Self::BlockIter>,
	) -> Self::Res {
		let mut usable_checkpoints = vec![];

		for prop_start in proposals_starts.borrow() {
			let mut checkpoint = 0_u32.into();
			for chp in checkpoint_blocks.borrow().iter().filter(|c| *c <= prop_start) {
				if chp >= &checkpoint {
					checkpoint = *chp;
				}
			}
			usable_checkpoints.push((*prop_start, checkpoint));
		}

		usable_checkpoints
	}
}
