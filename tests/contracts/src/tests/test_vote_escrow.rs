use codec::Encode;
use frame_support::assert_ok;
use crate::mock::*;

fn create_vote_escrow_for_alice() -> AccountId {
	let (escrow_contract, asset_contract) = create_vote_escrow_contract();
	let mut data = selector_from_str("PSP22::approve");
	data.append(&mut escrow_contract.clone().encode());
	data.append(&mut 100_u128.encode());
	assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

	let mut data = selector_from_str("create_lock");
	data.append(&mut 100_u128.encode());
	data.append(&mut 1000_u32.encode());
	assert_ok!(call::<()>(ALICE, escrow_contract.clone(), data));

	escrow_contract
}



#[test]
fn test_create_vote_escrow() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_contract();

		// check that the contract has the correct token
		let account_id = call::<AccountId>(
			ALICE,
			escrow_contract.clone(),
			selector_from_str("get_token")
		).expect("call success");
		assert_eq!(account_id, asset_contract);

		// check max time is correct
		let max_time = call::<u32>(
			ALICE,
			escrow_contract.clone(),
			selector_from_str("get_max_time")
		).expect("call success");
		assert_eq!(max_time, 1000);
	});
}

#[test]
fn test_create_lock() {
	new_test_ext().execute_with(|| {
		let (escrow_contract, asset_contract) = create_vote_escrow_contract();
		let asset_id = call::<u32>(
			ALICE,
			asset_contract.clone(),
			selector_from_str("get_asset_id")
		).expect("call success");

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), escrow_contract.clone()), 0);

		// no lock now
		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) = call::<(u128, u32, u32)>(
			ALICE,
			escrow_contract.clone(),
			data
		).expect("call success");
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
		let (amount, start_time, lock_time) = call::<(u128, u32, u32)>(
			ALICE,
			escrow_contract.clone(),
			data
		).expect("call success");
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
		let escrow_contract = create_vote_escrow_for_alice();
		let mut data = selector_from_str("get_lock");
		data.append(&mut ALICE.encode());
		let (amount, start_time, lock_time) = call::<(u128, u32, u32)>(
			ALICE,
			escrow_contract.clone(),
			data
		).expect("call success");

		assert_eq!(amount, 100);
		assert_eq!(start_time, 1);
		assert_eq!(lock_time, 1000);
	});
}