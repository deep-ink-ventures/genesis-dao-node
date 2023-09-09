use crate::{mock::*, test_utils::*, types::*, Config, Error, ProposalSlots, Proposals, Votes};
use frame_support::{assert_noop, assert_ok, traits::TypedGet};
use pallet_dao_core::{CurrencyOf, Error as DaoError};

#[test]
fn can_create_a_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		// cannot create a proposal without a DAO
		assert_noop!(
			DaoVotes::create_proposal(origin.clone(), dao_id.clone(),),
			DaoError::<Test>::DaoDoesNotExist
		);

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));

		// cannot create a proposal without DAO tokens existing (because they need to be reserved)
		assert_noop!(
			DaoVotes::create_proposal(origin.clone(), dao_id.clone(),),
			Error::<Test>::DaoTokenNotYetIssued
		);

		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));

		let dao = pallet_dao_core::Pallet::<Test>::load_dao(dao_id.clone()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		// check that no DAO tokens are reserved yet
		assert_eq!(
			pallet_dao_assets::pallet::Pallet::<Test>::reserved(asset_id, sender.clone()),
			Default::default()
		);

		let reserved_currency = CurrencyOf::<Test>::reserved_balance(sender.clone());

		// cannot create a proposal without a governance set
		assert_noop!(
			DaoVotes::create_proposal(origin.clone(), dao_id.clone()),
			Error::<Test>::GovernanceNotSet
		);

		// preparation: set governance
		let duration = 4200;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 10; // slightly less than 1 %
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));

		// check that a proposal does not exist yet
		assert!(!<ProposalSlots<Test>>::contains_key(DaoVotes::get_current_proposal_id()));

		// test creating a proposal
		assert_ok!(DaoVotes::create_proposal(origin, dao_id));

		// check that a proposal exists
		assert!(<ProposalSlots<Test>>::contains_key(DaoVotes::get_current_proposal_id()));

		// creating a proposal should reserve currency
		assert_eq!(
			CurrencyOf::<Test>::reserved_balance(sender.clone()),
			reserved_currency + <Test as Config>::ProposalDeposit::get()
		);

		// creating a proposal should reserve DAO tokens
		assert_eq!(
			pallet_dao_assets::pallet::Pallet::<Test>::reserved(asset_id, sender),
			token_deposit
		);
	});
}

#[test]
fn can_set_metadata() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		let dao_id = setup_dao_with_governance::<Test>(sender.clone());
		let prop_id = create_proposal_id::<Test>(sender, dao_id);

		let metadata = b"http://my.cool.proposal".to_vec();
		// https://en.wikipedia.org/wiki/SHA-3#Examples_of_SHA-3_variants
		let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();

		assert_ok!(DaoVotes::set_metadata(origin.clone(), prop_id, metadata.clone(), hash.clone()));
		// can only call once
		assert_noop!(
			DaoVotes::set_metadata(origin, prop_id, metadata, hash),
			Error::<Test>::ProposalDoesNotExist
		);
	});
}

#[test]
fn can_cast_and_remove_a_vote() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		let dao_id = setup_dao_with_governance::<Test>(sender.clone());
		let prop_id = setup_proposal::<Test>(sender, dao_id);

		// cannot create a vote without a proposal
		assert_noop!(DaoVotes::vote(origin, 0, None), Error::<Test>::ProposalDoesNotExist);

		let voter = BOB;
		let voter_origin = RuntimeOrigin::signed(voter.clone());
		let vote = true;
		// test creating a vote
		assert!(!<Votes<Test>>::contains_key(prop_id, voter.clone()));
		assert_ok!(DaoVotes::vote(voter_origin.clone(), prop_id, Some(vote)));
		assert_eq!(<Votes<Test>>::get(prop_id, voter.clone()), Some(vote));

		// test removing the same vote
		assert_ok!(DaoVotes::vote(voter_origin, prop_id, None));
		assert!(!<Votes<Test>>::contains_key(prop_id, voter));
	});
}

#[test]
fn can_fault_a_proposal() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		let dao_id = setup_dao_with_governance::<Test>(sender.clone());
		let prop_id = create_proposal_id::<Test>(sender.clone(), dao_id);
		let reason = b"Bad".to_vec();

		assert_noop!(
			DaoVotes::fault_proposal(origin.clone(), prop_id, reason.clone()),
			Error::<Test>::ProposalDoesNotExist
		);

		// setup proposal
		setup_proposal_with_id::<Test>(sender, prop_id);

		let non_owner = RuntimeOrigin::signed(BOB);
		assert_noop!(
			DaoVotes::fault_proposal(non_owner, prop_id, reason.clone()),
			Error::<Test>::SenderIsNotDaoOwner,
		);

		assert_ok!(DaoVotes::fault_proposal(origin, prop_id, reason));
	})
}

#[test]
fn can_finalize_a_proposal() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		let dao_id = setup_dao_with_governance_and_no_duration::<Test>(sender.clone());
		let prop_id = create_proposal_id::<Test>(sender.clone(), dao_id);

		assert_noop!(
			DaoVotes::finalize_proposal(origin.clone(), prop_id),
			Error::<Test>::ProposalDoesNotExist
		);

		// setup proposal
		setup_proposal_with_id::<Test>(sender, prop_id);

		// cannot finalize proposal that is still running
		assert_noop!(
			DaoVotes::finalize_proposal(origin.clone(), prop_id),
			Error::<Test>::ProposalDurationHasNotPassed
		);

		let mut block = System::block_number();
		block += 1;
		run_to_block::<Test>(block);
		assert_ok!(DaoVotes::finalize_proposal(origin, prop_id));
	})
}

#[test]
fn voting_outcome_unsuccessful_proposal() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));

		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1000));

		// preparation: set governance
		let duration = 0;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 0;
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));

		// preparation: create a proposal
		let prop_id = setup_proposal::<Test>(sender, dao_id);

		let voter = BOB;
		assert_ok!(Assets::transfer(origin.clone(), 1, voter.clone(), 500));
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id, Some(true)));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id, Some(false)));

		let block = System::block_number() + 1 + duration as u64;
		run_to_block::<Test>(block);
		assert_ok!(DaoVotes::finalize_proposal(origin, prop_id));
		let proposal = Proposals::<Test>::get(prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Rejected);
	})
}

#[test]
fn voting_outcome_successful_proposal_and_mark_implemented() {
	new_test_ext().execute_with(|| {
		let dao_id = b"DAO".to_vec();
		let dao_name = b"TEST DAO".to_vec();
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());

		// preparation: create a DAO
		assert_ok!(DaoCore::create_dao(origin.clone(), dao_id.clone(), dao_name));

		// preparation: issue token
		assert_ok!(DaoCore::issue_token(origin.clone(), dao_id.clone(), 1001));

		// preparation: set governance
		let duration = 0;
		let token_deposit = 100;
		let minimum_majority_per_1024 = 0;
		assert_ok!(DaoVotes::set_governance_majority_vote(
			origin.clone(),
			dao_id.clone(),
			duration,
			token_deposit,
			minimum_majority_per_1024
		));

		let prop_id = create_proposal_id::<Test>(sender.clone(), dao_id);

		assert_noop!(
			DaoVotes::mark_implemented(origin.clone(), prop_id),
			Error::<Test>::ProposalDoesNotExist
		);

		// preparation: create a proposal
		setup_proposal_with_id::<Test>(sender, prop_id);

		assert_noop!(
			DaoVotes::mark_implemented(origin.clone(), prop_id),
			Error::<Test>::ProposalStatusNotAccepted
		);

		let voter = BOB;
		let asset_id = 1;
		assert_ok!(Assets::transfer(origin.clone(), asset_id, voter.clone(), 501));
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id, Some(true)));
		assert_ok!(DaoVotes::vote(origin.clone(), prop_id, Some(false)));

		let block = System::block_number() + 1 + duration as u64;
		run_to_block::<Test>(block);
		assert_ok!(DaoVotes::finalize_proposal(origin.clone(), prop_id));

		let proposal = Proposals::<Test>::get(prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Accepted);

		assert_ok!(DaoVotes::mark_implemented(origin, prop_id));
		let proposal = Proposals::<Test>::get(prop_id).unwrap();
		assert_eq!(proposal.status, ProposalStatus::Implemented);
	})
}

#[test]
fn returns_active_proposals_for_a_dao() {
	new_test_ext().execute_with(|| {
		use commons::traits::pallets::ActiveProposals;

		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());

		let dao1 = setup_dao_by_name::<Test>(b"DAO".to_vec(), b"TEST DAO".to_vec(), sender.clone());
		let dao2 =
			setup_dao_by_name::<Test>(b"DAO2".to_vec(), b"TEST DAO2".to_vec(), sender.clone());

		let proposal_duration = 1000_u32;
		let proposal_token_deposit = 1_u32.into();
		let minimum_majority_per_1024 = 10;
		assert_eq!(
			DaoVotes::set_governance_majority_vote(
				origin.clone(),
				dao1.clone(),
				proposal_duration,
				proposal_token_deposit,
				minimum_majority_per_1024
			),
			Ok(())
		);
		assert_eq!(
			DaoVotes::set_governance_majority_vote(
				origin.clone(),
				dao2.clone(),
				proposal_duration,
				proposal_token_deposit,
				minimum_majority_per_1024
			),
			Ok(())
		);

		setup_proposal::<Test>(sender.clone(), dao1.clone());
		run_to_block::<Test>(100_u64);
		setup_proposal::<Test>(sender.clone(), dao2.clone());

		run_to_block::<Test>(500_u64);
		setup_proposal::<Test>(sender.clone(), dao1.clone());

		let proposals =
			DaoVotes::active_proposals_starting_time(dao1.clone(), System::block_number());

		assert_eq!(proposals.len(), 2);
		assert_eq!(proposals[0], 1_u64);
		assert_eq!(proposals[1], 500_u64);

		run_to_block::<Test>(1002_u64);
		let proposals = DaoVotes::active_proposals_starting_time(dao1, System::block_number());
		assert_eq!(proposals.len(), 1);
		assert_eq!(proposals[0], 500_u64);
	});
}

#[test]
fn on_vote_calculation_callback_works() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());
		let dao_id = setup_dao_with_governance::<Test>(sender.clone());
		let prop_id = setup_proposal::<Test>(sender.clone(), dao_id);

		let voter = BOB;
		let asset_id = 1;

		// this is our state before
		assert_ok!(Assets::transfer(origin.clone(), asset_id, voter.clone(), 50));
		assert_eq!(<Proposals<Test>>::get(prop_id).unwrap().in_favor, 0);

		let contract_path = "../../contracts/hooks/genesis-dao-contract-tests/test_contract.wasm";
		let contract_deployment = HookPoints::prepare_deployment(
			"new",
			sender.clone(),
			std::fs::read(contract_path).unwrap(),
			vec![],
		);

		let contract_account = HookPoints::install(contract_deployment).expect("code deployed");

		assert_ok!(HookPoints::register_specific_callback(
			origin,
			contract_account,
			"GenesisDao::on_vote".into(),
		));

		// now let's vote
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(voter), prop_id, Some(true)));
		// and this should be multiplied by 2, as defined in the above ink! contract
		assert_eq!(<Proposals<Test>>::get(prop_id).unwrap().in_favor, 100);
	});
}
