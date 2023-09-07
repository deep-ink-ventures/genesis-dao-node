use core::borrow::Borrow;

use frame_support::{
	pallet_prelude::*,
	sp_runtime::{traits::One, Saturating},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::{marker::PhantomData, prelude::*};

pub trait ActiveProposals<BlockNumber> {
	/// Get the starting time of all active proposals.
	///
	/// This is an easy wrapper around `get_active_proposals` that returns only the starting time of
	/// each proposal, without the proposal itself; avoiding the need to implement unnecessary
	/// traits.
	///
	/// - `dao_id`: the unique identifier for the DAO
	/// - `current_block`: the current block number
	fn active_proposals_starting_time(
		dao_id: Vec<u8>,
		current_block: BlockNumber,
	) -> Vec<BlockNumber>;
}

pub struct ActiveProposalsMock<T> {
	// include any fields that might depend on T
	_marker: PhantomData<T>,
}

impl<T: frame_system::Config> ActiveProposals<BlockNumberFor<T>> for ActiveProposalsMock<T> {
	fn active_proposals_starting_time(
		_dao_id: Vec<u8>,
		_current_block: BlockNumberFor<T>,
	) -> Vec<BlockNumberFor<T>> {
		return vec![100_u32.into()]
	}
}

pub trait AssetInterface {
	type AccountId;
	type BlockNumber;
	type Balance;
	type AssetId;
	type AssetInfo;

	/// Given assetIf, give the AssetDetails
	fn get_asset(asset_id: Self::AssetId) -> Option<Self::AssetInfo>;

	/// Force create an asset
	fn force_create(
		id: Self::AssetId,
		owner: Self::AccountId,
		min_balance: Self::Balance,
	) -> DispatchResult;

	/// Mint
	fn mint(
		id: Self::AssetId,
		beneficiary: &Self::AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Set metadata
	fn set_metadata(
		id: Self::AssetId,
		from: &Self::AccountId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> DispatchResult;

	/// Change the owner
	fn change_owner(id: Self::AssetId, new_owner: Self::AccountId) -> DispatchResult;

	/// Reserve
	fn reserve(
		id: Self::AssetId,
		target: impl Borrow<Self::AccountId>,
		amount: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	/// Get total historical supply
	fn total_historical_supply(
		id: Self::AssetId,
		block: Self::BlockNumber,
	) -> Option<Self::Balance>;

	/// Get total historical balance
	fn total_historical_balance(
		id: Self::AssetId,
		who: impl Borrow<Self::AccountId>,
		block: Self::BlockNumber,
	) -> Option<Self::Balance>;
}
