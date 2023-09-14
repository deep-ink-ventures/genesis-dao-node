use crate::mock::*;
use codec::Encode;
use frame_support::assert_ok;

fn create_vote_escrow_for_alice() -> (AccountId, AccountId) {
	let (escrow_contract, asset_contract) = create_vote_escrow_contract();
	let mut data = selector_from_str("PSP22::approve");
	data.append(&mut escrow_contract.clone().encode());
	data.append(&mut 100_u128.encode());
	assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

	let mut data = selector_from_str("create_lock");
	data.append(&mut 100_u128.encode());
	data.append(&mut 1000_u32.encode());
	assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

	(escrow_contract, asset_contract)
}

#[test]
fn test_create_vote_escrow() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_contract();

		// check that the contract has the correct token
		let account_id =
			call::<AccountId>(ALICE, escrow_contract.clone(), selector_from_str("get_token"))
				.expect("call success");
		assert_eq!(account_id, asset_contract);

		// check max time is correct
		let max_time =
			call::<u32>(ALICE, escrow_contract.clone(), selector_from_str("get_max_time"))
				.expect("call success");
		assert_eq!(max_time, 1000);
	});
}

#[test]
fn test_create_lock() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_contract();
		let asset_id =
			call::<u32>(ALICE, asset_contract.clone(), selector_from_str("get_asset_id"))
				.expect("call success");

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 0);

		// no lock now
		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) =
			call::<(u128, u32, u32)>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(amount, 0);
		assert_eq!(start_time, 0);
		assert_eq!(lock_time, 0);

		// ALICE must approve the vesting wallet for withdrawal
		let mut data = selector_from_str("PSP22::approve");
		data.append(&mut escrow_contract.clone().encode());
		data.append(&mut 100_u128.encode());
		assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

		// above max lock
		let mut data = selector_from_str("create_lock");
		data.append(&mut 100_u128.encode());
		data.append(&mut 1001_u32.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		// above approval
		let mut data = selector_from_str("create_lock");
		data.append(&mut 101_u128.encode());
		data.append(&mut 1000_u32.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 0);

		// all good
		let mut data = selector_from_str("create_lock");
		data.append(&mut 100_u128.encode());
		data.append(&mut 1000_u32.encode());
		assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

		// now there's a lock
		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) =
			call::<(u128, u32, u32)>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(amount, 100);
		assert_eq!(start_time, 1);
		assert_eq!(lock_time, 1000);

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 100);
	});
}

#[test]
fn test_escrow_lifecycle() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_for_alice();
		let asset_id =
			call::<u32>(ALICE, asset_contract.clone(), selector_from_str("get_asset_id"))
				.expect("call success");

		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) =
			call::<(u128, u32, u32)>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(amount, 100);
		assert_eq!(start_time, 1);
		assert_eq!(lock_time, 1000);

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 100);

		// get voting power
		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 500);

		forward_by_blocks(250);

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 400);

		forward_by_blocks(250);

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 300);

		forward_by_blocks(250);
		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 200);

		// cannot withdraw before lock time is over
		let mut data = selector_from_str("withdraw");
		data.append(&mut ALICE.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());
		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 100);

		forward_by_blocks(250);
		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 100);

		let mut data = selector_from_str("withdraw");
		data.append(&mut ALICE.encode());
		assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 0);

		// only once ...
		let mut data = selector_from_str("withdraw");
		data.append(&mut ALICE.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) =
			call::<(u128, u32, u32)>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(amount, 0);
		assert_eq!(start_time, 0);
		assert_eq!(lock_time, 0);
	});
}

#[test]
fn test_increase_amount() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_for_alice();
		let asset_id =
			call::<u32>(ALICE, asset_contract.clone(), selector_from_str("get_asset_id"))
				.expect("call success");
		forward_by_blocks(500);

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 300);

		// unknown account
		let mut data = selector_from_str("increase_amount");
		data.append(&mut 100_u128.encode());
		assert!(call::<()>(BOB, escrow_contract.clone(), data).is_err());

		// increase without approve should not work
		let mut data = selector_from_str("increase_amount");
		data.append(&mut 100_u128.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		let mut data = selector_from_str("PSP22::approve");
		data.append(&mut escrow_contract.clone().encode());
		data.append(&mut 100_u128.encode());
		assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

		let mut data = selector_from_str("increase_amount");
		data.append(&mut 100_u128.encode());
		assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 600);

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 800);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 200);
	});
}

#[test]
fn test_increase_lock_time() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, _) = create_vote_escrow_for_alice();
		forward_by_blocks(500);

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 300);

		// unknown account
		let mut data = selector_from_str("increase_unlock_time");
		data.append(&mut 500.encode());
		assert!(call::<()>(BOB, escrow_contract.clone(), data).is_err());

		// to much
		let mut data = selector_from_str("increase_unlock_time");
		data.append(&mut 1001.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		// can't be lower than remaining
		let mut data = selector_from_str("increase_unlock_time");
		data.append(&mut 499.encode());
		assert!(call::<()>(ALICE, escrow_contract.clone(), data).is_err());

		let mut data = selector_from_str("increase_unlock_time");
		data.append(&mut 1000.encode());
		assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

		let mut data = selector_from_str("voting_power");
		data.append(&mut ALICE.encode());
		let voting_power =
			call::<u128>(ALICE, escrow_contract.clone(), data).expect("call success");
		assert_eq!(voting_power, 500)
	});
}
