use codec::Encode;
use frame_support::assert_ok;
use frame_support::dispatch::RawOrigin;
use crate::mock::*;

/// Sets up Bob with:
/// - a vesting wallet worth 100 tokens for 1000 blocks
/// - a vote escrow worth 100 tokens for 1000 blocks
///
/// Returns the dao, vesting and voting contracts
fn warmup_on_vote() -> (AccountId, AccountId, AccountId) {
    let asset_contract = create_assets_contract();
    let mut data = selector_from_str("new");
	data.append(&mut asset_contract.clone().encode());
	let vesting_contract = install(ALICE, VESTING_WALLET_CONTRACT_PATH, data).expect("code deployed");

    let mut data = selector_from_str("new");
	data.append(&mut asset_contract.clone().encode());
	data.append(&mut 1000_u32.encode());
	data.append(&mut 4_u8.encode());
    let voting_contract = install(ALICE, VOTE_ESCROW_CONTRACT_PATH, data).expect("code deployed");

    let dao_contract = install(ALICE, DAO_CONTRACT_PATH, selector_from_str("new")).expect("code deployed");

    // create a vesting wallet for Bob
    let mut data = selector_from_str("PSP22::approve");
	data.append(&mut vesting_contract.clone().encode());
	data.append(&mut 100_u128.encode());
	assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

	let mut data = selector_from_str("create_vesting_wallet_for");
	data.append(&mut BOB.encode());
	data.append(&mut 100_u128.encode());
	data.append(&mut 1000_u32.encode());
	assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

    // create vote escrow for Bob
    let mut data = selector_from_str("PSP22::transfer");
    data.append(&mut BOB.clone().encode());
    data.append(&mut 200_u128.encode());
    data.append(&mut "empty".encode());
    call::<()>(ALICE, asset_contract.clone(), data).expect("call success");

    let mut data = selector_from_str("PSP22::approve");
	data.append(&mut voting_contract.clone().encode());
	data.append(&mut 100_u128.encode());
	assert_ok!(call::<()>(BOB, asset_contract.clone(), data));

	let mut data = selector_from_str("create_lock");
	data.append(&mut 100_u128.encode());
	data.append(&mut 1000_u32.encode());
	assert_ok!(call::<()>(BOB, voting_contract.clone(), data));

	assert_ok!(HookPoints::register_global_callback(
		RawOrigin::Signed(ALICE).into(),
		dao_contract.clone()
	));

    (dao_contract, vesting_contract, voting_contract)
}

// sets up a proposal that we wanna use for voting
fn create_test_proposals() -> u32 {
	let dao_id: Vec<u8> = b"GDAO".to_vec();
	assert_ok!(DaoVotes::set_governance_majority_vote(
		RawOrigin::Signed(ALICE).into(),
		dao_id.clone(),
		1000,
		1_u32.into(),
		10
	));
	assert_ok!(DaoVotes::create_proposal(RawOrigin::Signed(ALICE).into(), dao_id));
	let proposal_id = DaoVotes::get_current_proposal_id();
	let metadata = b"http://my.cool.proposal".to_vec();
	let hash = b"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a".to_vec();
	assert_ok!(DaoVotes::set_metadata(RawOrigin::Signed(ALICE).into(), proposal_id, metadata, hash));
	proposal_id
}

#[test]
fn test_on_vote_plugins() {
	new_test_ext().execute_with(|| {
        let (dao_contract, vesting_contract, voting_contract, ) = warmup_on_vote();

		let asset_id = get_asset_id_from_contract(vesting_contract.clone());
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);

		let prop_id = create_test_proposals();
        forward_by_blocks(100);

		// let's register both plugins!
		let mut data = selector_from_str("register_vote_plugin");
		data.append(&mut vesting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		let mut data = selector_from_str("register_vote_plugin");
		data.append(&mut voting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		// now let's vote
		assert_ok!(DaoVotes::vote(RuntimeOrigin::signed(BOB), prop_id, Some(true)));

		// ok so:
		// 100 tokens were locked in the vote escrow with a boost of 4 and a max of 1000
		// so after 100 blocks, the voting power should be:
		//
		// locked * (max - blocks_passed) / max * boost + locked
		//
		// 100 * (1000 - 100) / 1000 * 4 + 100 = 460
		//
		// 100 tokens are vested over 100 blocks and none withdrawn so there should be another 100
		//
		// 100 is the balance of Bob
		//
		// so the total should be
		// 460 + 100 + 100 = 660
		assert_eq!(DaoVotes::proposals(prop_id).unwrap().in_favor, 660);

		// note: removing now is not counting to already casted votes!
	});
}

#[test]
fn test_add_and_remove_vote_plugins() {
	new_test_ext().execute_with(|| {
		let (dao_contract, vesting_contract, voting_contract, ) = warmup_on_vote();

		let mut data = selector_from_str("register_vote_plugin");
		data.append(&mut vesting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		let mut data = selector_from_str("register_vote_plugin");
		data.append(&mut voting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		// check they exists
		let contracts = call::<Vec<AccountId>>(ALICE, dao_contract.clone(), selector_from_str("get_vote_plugins")).unwrap();
		assert_eq!(contracts.len(), 2);
		assert_eq!(contracts[0], vesting_contract);
		assert_eq!(contracts[1], voting_contract);

		// remove 'em all
		let mut data = selector_from_str("remove_vote_plugin");
		data.append(&mut vesting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		let mut data = selector_from_str("remove_vote_plugin");
		data.append(&mut voting_contract.clone().encode());
		assert_ok!(call::<()>(ALICE, dao_contract.clone(), data));

		// check they are gone
		let contracts = call::<Vec<AccountId>>(ALICE, dao_contract.clone(), selector_from_str("get_vote_plugins")).unwrap();
		assert_eq!(contracts.len(), 0);
	});
}
