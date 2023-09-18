use crate::{
	pallet::{Governances, Proposals},
	Config, Pallet, ProposalOf, ProposalStatus,
};
use commons::traits::pallets::ActiveProposals;
use frame_support::traits::Get;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_std::prelude::*;

impl<T: Config> Pallet<T> {
	fn get_active_proposals(
		dao_id: Vec<u8>,
		current_block: BlockNumberFor<T>,
	) -> Vec<ProposalOf<T>> {
		let dao = match pallet_dao_core::Pallet::<T>::load_dao(dao_id.clone()) {
			Ok(dao) => dao,
			Err(_) => return Vec::<ProposalOf<T>>::new(),
		};

		let governance = match <Governances<T>>::get(dao.id) {
			Some(governance) => governance,
			None => return Vec::<ProposalOf<T>>::new(),
		};

		<Proposals<T>>::iter()
			.filter(|(_, proposal)| {
				proposal.dao_id == dao_id &&
					proposal.status == ProposalStatus::Running &&
					proposal.birth_block + governance.proposal_duration.into() >= current_block
			})
			.map(|(_, proposal)| proposal)
			.collect::<Vec<_>>()
	}
}
impl<T: Config> ActiveProposals<BlockNumberFor<T>> for Pallet<T> {
	fn max_proposals_limit() -> u32 {
		T::MaxProposals::get()
	}

	fn active_proposals_starting_time(
		dao_id: Vec<u8>,
		current_block: BlockNumberFor<T>,
	) -> Vec<BlockNumberFor<T>> {
		let active_proposals = Self::get_active_proposals(dao_id, current_block);
		active_proposals.iter().map(|proposal| proposal.birth_block).collect::<Vec<_>>()
	}
}
