use core::borrow::Borrow;

use frame_support::pallet_prelude::*;
use sp_std::prelude::*;

pub trait ActiveProposals<BlockNumber> {
	/// Get the starting time of all active proposals.
	///
	/// This is an easy wrapper around `get_active_proposals` that returns only the starting time of
	/// each proposal, without the proposal itself; avoiding the need to implement unnecessary
	/// pallets.
	///
	/// - `dao_id`: the unique identifier for the DAO
	/// - `current_block`: the current block number
	fn active_proposals_starting_time(
		dao_id: Vec<u8>,
		current_block: BlockNumber,
	) -> Vec<BlockNumber>;

	/// Get the maximum number of proposals that can be active at the same time
	fn max_proposals_limit() -> u32;
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
	) -> Self::Balance;
}

pub trait UsableCheckpoints {
	type BlockNumber: Copy;
	type BlockIter;
	type Res;

	fn proposal_checkpoint_pair(
		// where porposals starts
		proposals_starts: impl Borrow<Self::BlockIter>,
		// where checkpoint are made
		checkpoint_blocks: impl Borrow<Self::BlockIter>,
	) -> Self::Res;
}
