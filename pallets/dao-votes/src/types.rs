use codec::MaxEncodedLen;
use frame_support::{
	codec::{Decode, Encode},
	traits::ConstU32,
	BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct ProposalSlot<DaoId, AccountId> {
	pub dao_id: DaoId,
	pub creator: AccountId,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Proposal<DaoId, AccountId, BlockId, Balance, Metadata> {
	pub dao_id: DaoId,
	pub creator: AccountId,
	pub birth_block: BlockId,
	pub meta: Metadata,
	pub meta_hash: BoundedVec<u8, ConstU32<64>>,
	pub status: ProposalStatus,
	pub in_favor: Balance,
	pub against: Balance,
}

#[derive(
	Clone, Copy, Encode, Decode, Eq, PartialEq, Default, RuntimeDebug, MaxEncodedLen, TypeInfo,
)]
pub enum ProposalStatus {
	#[default]
	Running,
	Accepted,
	Rejected,
	Faulty,
	Implemented,
}


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Governance<Balance> {
	// the number of blocks a proposal is open for voting
	pub proposal_duration: u32,
	// the token deposit required to create a proposal
	pub proposal_token_deposit: Balance,
	// the rules for accepting proposals
	pub voting: Voting,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Voting {
	// The default, majority vote
	Majority {
		// how many more ayes than nays there must be for proposal acceptance
		// thus proposal acceptance requires: ayes >= nays + token_supply / 1024 *
		// minimum_majority_per_1024
		minimum_majority_per_1024: u8,
	},
	// hook point entrypoint
	Custom
}
