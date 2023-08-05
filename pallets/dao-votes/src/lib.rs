#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

use frame_support::{
	sp_runtime::traits::{One, Saturating, Zero},
	storage::bounded_vec::BoundedVec,
	traits::ReservableCurrency,
};

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod test_utils;

mod types;
pub use types::*;

use pallet_dao_assets::{AssetBalanceOf, Pallet as Assets};
use pallet_dao_core::{
	AccountIdOf, CurrencyOf, DaoIdOf, DepositBalanceOf, Error as DaoError, Pallet as Core,
};

pub mod weights;
mod hooks;

use frame_system::pallet_prelude::BlockNumberFor;
use weights::WeightInfo;

type ProposalSlotOf<T> = ProposalSlot<DaoIdOf<T>, <T as frame_system::Config>::AccountId>;
type ProposalOf<T> = Proposal<
	DaoIdOf<T>,
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	AssetBalanceOf<T>,
	pallet_dao_core::MetadataOf<T>,
>;

type GovernanceOf<T> = Governance<AssetBalanceOf<T>>;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use crate::hooks::on_vote_callback;

	#[pallet::storage]
	pub(super) type Governances<T: Config> =
		StorageMap<_, Twox64Concat, DaoIdOf<T>, GovernanceOf<T>>;

	#[pallet::storage]
	pub(super) type ProposalSlots<T: Config> =
		StorageMap<_, Twox64Concat, T::ProposalId, ProposalSlotOf<T>>;

	#[pallet::storage]
	pub(super) type Proposals<T: Config> =
		StorageMap<_, Twox64Concat, T::ProposalId, ProposalOf<T>>;

	#[pallet::storage]
	pub(super) type Votes<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::ProposalId, Twox64Concat, AccountIdOf<T>, bool>;

	/// Internal incrementor of all proposals created by this module.
	#[pallet::storage]
	#[pallet::getter(fn get_current_proposal_id)]
	pub type CurrentProposalId<T: Config> = StorageValue<_, T::ProposalId, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_dao_core::Config + pallet_hookpoints::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type ProposalDeposit: Get<DepositBalanceOf<Self>>;

		type ProposalId: Default
			+ Member
			+ Parameter
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ One
			+ Saturating;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProposalCreated {
			dao_id: DaoIdOf<T>,
			creator: AccountIdOf<T>,
			proposal_id: T::ProposalId,
		},
		ProposalMetadataSet {
			proposal_id: T::ProposalId,
		},
		ProposalFaulted {
			proposal_id: T::ProposalId,
			reason: Vec<u8>,
		},
		ProposalAccepted {
			proposal_id: T::ProposalId,
		},
		ProposalRejected {
			proposal_id: T::ProposalId,
		},
		ProposalCounting {
			proposal_id: T::ProposalId,
		},
		ProposalImplemented {
			proposal_id: T::ProposalId,
		},
		VoteCast {
			proposal_id: T::ProposalId,
			voter: AccountIdOf<T>,
			in_favor: Option<bool>,
		},
		SetGovernanceMajorityVote {
			dao_id: DaoIdOf<T>,
			proposal_duration: u32,
			proposal_token_deposit: T::Balance,
			minimum_majority_per_1024: u8,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoTokenNotYetIssued,
		GovernanceNotSet,
		ProposalDoesNotExist,
		ProposalStatusNotRunning,
		ProposalStatusNotAccepted,
		ProposalDurationHasNotPassed,
		ProposalDurationHasPassed,
		SenderIsNotDaoOwner,
		SenderIsNotProposalCreator,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_proposal())]
		pub fn create_proposal(origin: OriginFor<T>, dao_id: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			let dao = pallet_dao_core::Pallet::<T>::load_dao(dao_id)?;
			let dao_id = dao.id;
			let asset_id = dao.asset_id.ok_or(Error::<T>::DaoTokenNotYetIssued)?;
			let governance =
				<Governances<T>>::get(dao_id.clone()).ok_or(Error::<T>::GovernanceNotSet)?;

			let deposit = <T as Config>::ProposalDeposit::get();

			// reserve currency
			CurrencyOf::<T>::reserve(&sender, deposit)?;

			// reserve DAO token, but unreserve currency if that fails
			if let Err(error) = pallet_dao_assets::Pallet::<T>::do_reserve(
				asset_id.into(),
				&sender,
				governance.proposal_token_deposit,
			) {
				CurrencyOf::<T>::unreserve(&sender, deposit);
				Err(error)?;
			};
			// increase proposal counter
			<CurrentProposalId<T>>::mutate(|id| id.saturating_inc());

			// store a proposal slot
			<ProposalSlots<T>>::insert(
				Self::get_current_proposal_id(),
				ProposalSlot { dao_id: dao_id.clone(), creator: sender.clone() },
			);
			// emit an event
			Self::deposit_event(Event::<T>::ProposalCreated {
				dao_id,
				creator: sender,
				proposal_id: Self::get_current_proposal_id(),
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)] //<T as pallet::Config>::WeightInfo::create_proposal())]
		pub fn set_metadata(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
			meta: Vec<u8>,
			hash: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let slot =
				ProposalSlots::<T>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;
			ensure!(sender == slot.creator, Error::<T>::SenderIsNotProposalCreator);

			let meta: BoundedVec<_, _> =
				meta.try_into().map_err(|_| DaoError::<T>::MetadataInvalidLengthTooLong)?;
			let hash: BoundedVec<_, _> =
				hash.try_into().map_err(|_| DaoError::<T>::HashInvalidWrongLength)?;

			let birth_block = <frame_system::Pallet<T>>::block_number();
			// store the proposal
			ProposalSlots::<T>::remove(proposal_id);
			Proposals::<T>::insert(
				proposal_id,
				Proposal {
					dao_id: slot.dao_id,
					creator: sender,
					birth_block,
					status: ProposalStatus::Running,
					in_favor: Zero::zero(),
					against: Zero::zero(),
					meta,
					meta_hash: hash,
				},
			);

			// emit an event
			Self::deposit_event(Event::<T>::ProposalMetadataSet { proposal_id });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::fault_proposal())]
		pub fn fault_proposal(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
			reason: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check that a proposal exists with the given id
			let mut proposal =
				<Proposals<T>>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			// check that sender is owner of the DAO
			ensure!(
				sender == Core::<T>::get_dao(&proposal.dao_id).expect("DAO exists").owner,
				Error::<T>::SenderIsNotDaoOwner
			);

			proposal.status = ProposalStatus::Faulty;
			<Proposals<T>>::insert(proposal_id, proposal.clone());

			// unreserve currency
			CurrencyOf::<T>::unreserve(&proposal.creator, <T as Config>::ProposalDeposit::get());

			Self::deposit_event(Event::<T>::ProposalFaulted { proposal_id, reason });

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::finalize_proposal(T::HistoryHorizon::get()))]
		pub fn finalize_proposal(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// check that a proposal exists with the given id
			let mut proposal =
				<Proposals<T>>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			// check that the proposal is currently running
			ensure!(
				proposal.status == ProposalStatus::Running,
				Error::<T>::ProposalStatusNotRunning
			);
			let governance =
				<Governances<T>>::get(&proposal.dao_id).ok_or(Error::<T>::GovernanceNotSet)?;
			let current_block = <frame_system::Pallet<T>>::block_number();

			// check that the proposal has run for its entire duration
			ensure!(
				current_block - proposal.birth_block > governance.proposal_duration.into(),
				Error::<T>::ProposalDurationHasNotPassed
			);

			let asset_id = Core::<T>::get_dao(&proposal.dao_id)
				.expect("DAO exists")
				.asset_id
				.expect("asset has been issued");

			// per default you just need to have more people in your favour than against ...
			if proposal.in_favor > proposal.against && {
				match governance.voting {
					// we ship a majority vote implementation as default, that is requiring a threshold
					// to be exceeded for a proposal to pass
					Voting::Majority { minimum_majority_per_1024 } => {
						let token_supply = Assets::<T>::total_historical_supply(
							asset_id.into(),
							proposal.birth_block,
						)
						.expect("History exists (horizon checked above)");
						let required_majority = token_supply /
							Into::<AssetBalanceOf<T>>::into(1024_u32) *
							minimum_majority_per_1024.into();
						// check for the required majority
						proposal.in_favor - proposal.against >= required_majority
					}
					// the custom voting mechanism allows for the interception with a hookpoint for custom logic.
					Voting::Custom => true
				}
			} {
				proposal.status = ProposalStatus::Accepted;
			} else {
				proposal.status = ProposalStatus::Rejected;
			}

			// unreserve proposal deposit
			CurrencyOf::<T>::unreserve(&sender, <T as Config>::ProposalDeposit::get());

			// record updated proposal status
			<Proposals<T>>::insert(proposal_id, proposal.clone());

			// emit event
			Self::deposit_event(match proposal.status {
				ProposalStatus::Accepted => Event::ProposalAccepted { proposal_id },
				ProposalStatus::Rejected => Event::ProposalRejected { proposal_id },
				_ => unreachable!(),
			});

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::vote())]
		pub fn vote(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
			in_favor: Option<bool>,
		) -> DispatchResult {
			let voter: <T as frame_system::Config>::AccountId = ensure_signed(origin)?;

			// check that a proposal exists with the given id
			let mut proposal =
				<Proposals<T>>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			// check that the proposal is running
			ensure!(
				proposal.status == ProposalStatus::Running,
				Error::<T>::ProposalStatusNotRunning
			);

			let governance =
				<Governances<T>>::get(&proposal.dao_id).ok_or(Error::<T>::GovernanceNotSet)?;

			// check that the proposal has not yet run for its entire duration
			ensure!(
				<frame_system::Pallet<T>>::block_number() - proposal.birth_block <=
					governance.proposal_duration.into(),
				Error::<T>::ProposalDurationHasPassed
			);

			let vote = <Votes<T>>::get(proposal_id, &voter);
			if vote == in_favor {
				// vote already stored
				return Ok(())
			}

			<Votes<T>>::set(proposal_id, &voter, in_favor);
			let dao = Core::<T>::get_dao(&proposal.dao_id).expect("DAO exists");
			let asset_id = dao.asset_id.expect("asset has been issued");
			let token_balance = Assets::<T>::total_historical_balance(
				asset_id.into(),
				&voter,
				proposal.birth_block,
			)
			.expect("history exists");

			let voting_power = on_vote_callback::<T>(dao.owner, voter.clone(), token_balance);

			// undo old vote
			match vote {
				Some(true) => {
					proposal.in_favor -= voting_power;
				},
				Some(false) => {
					proposal.against -= voting_power;
				},
				None => {},
			}
			// count new vote
			match in_favor {
				Some(true) => {
					proposal.in_favor += voting_power;
				},
				Some(false) => {
					proposal.against += voting_power;
				},
				None => {},
			}
			// record updated proposal counts
			<Proposals<T>>::insert(proposal_id, proposal);

			Self::deposit_event(Event::<T>::VoteCast { proposal_id, voter, in_favor });
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_governance_majority_vote())]
		pub fn set_governance_majority_vote(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			proposal_duration: u32,
			proposal_token_deposit: T::Balance,
			minimum_majority_per_1024: u8,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = pallet_dao_core::Pallet::<T>::load_dao(dao_id)?;
			let dao_id = dao.id;
			ensure!(dao.owner == sender, DaoError::<T>::DaoSignerNotOwner);
			let voting = Voting::Majority { minimum_majority_per_1024 };
			let gov = GovernanceOf::<T> { proposal_duration, proposal_token_deposit, voting };
			<Governances<T>>::set(dao_id.clone(), Some(gov));
			Self::deposit_event(Event::<T>::SetGovernanceMajorityVote {
				dao_id,
				proposal_duration,
				proposal_token_deposit,
				minimum_majority_per_1024,
			});
			Ok(())
		}

		#[pallet::call_index(7)]
		//#[pallet::weight(<T as pallet::Config>::WeightInfo::mark_implemented())]
		pub fn mark_implemented(
			origin: OriginFor<T>,
			proposal_id: T::ProposalId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			<Proposals<T>>::try_mutate(proposal_id, |maybe_proposal| -> DispatchResult {
				let proposal = maybe_proposal.as_mut().ok_or(Error::<T>::ProposalDoesNotExist)?;
				let dao = pallet_dao_core::Daos::<T>::get(&proposal.dao_id)
					.ok_or(DaoError::<T>::DaoDoesNotExist)?;
				ensure!(dao.owner == sender, DaoError::<T>::DaoSignerNotOwner);

				// check that the proposal has been accepted
				ensure!(
					proposal.status == ProposalStatus::Accepted,
					Error::<T>::ProposalStatusNotAccepted
				);

				proposal.status = ProposalStatus::Implemented;
				Ok(())
			})?;

			Self::deposit_event(Event::<T>::ProposalImplemented { proposal_id });
			Ok(())
		}
	}
}
