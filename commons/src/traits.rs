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
