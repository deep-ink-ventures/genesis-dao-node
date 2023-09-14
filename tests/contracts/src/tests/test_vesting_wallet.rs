use crate::mock::*;
use codec::Encode;
use frame_support::assert_ok;

fn create_vesting_wallet_for_bob() -> AccountId {
	let (vesting_contract, asset_contract) = create_vesting_wallet_contract();

	let mut data = selector_from_str("PSP22::approve");
	data.append(&mut vesting_contract.clone().encode());
	data.append(&mut 100_u128.encode());
	assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

	let mut data = selector_from_str("create_vesting_wallet_for");
	data.append(&mut BOB.encode());
	data.append(&mut 100_u128.encode());
	data.append(&mut 1000_u32.encode());
	assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));
	vesting_contract
}

#[test]
fn test_create_vesting_wallet() {
	new_test_ext().execute_with(|| {
		let (vesting_contract, asset_contract) = create_vesting_wallet_contract();

		// check that the vesting contract has the correct token
		let account_id =
			call::<AccountId>(ALICE, vesting_contract.clone(), selector_from_str("get_token"))
				.expect("call success");
		assert_eq!(account_id, asset_contract);

		let asset_id =
			call::<u32>(ALICE, asset_contract.clone(), selector_from_str("get_asset_id"))
				.expect("call success");

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 0);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		// ALICE must approve the vesting wallet for withdrawal
		let mut data = selector_from_str("PSP22::approve");
		data.append(&mut vesting_contract.clone().encode());
		data.append(&mut 100_u128.encode());
		assert_ok!(call::<()>(ALICE, asset_contract.clone(), data));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 0);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		// // unable to fund because 101 > 100
		let mut data = selector_from_str("create_vesting_wallet_for");
		data.append(&mut BOB.encode());
		data.append(&mut 101_u128.encode());
		data.append(&mut 1000_u32.encode());
		assert!(call::<()>(ALICE, vesting_contract.clone(), data).is_err());

		let mut data = selector_from_str("create_vesting_wallet_for");
		data.append(&mut BOB.encode());
		data.append(&mut 100_u128.encode());
		data.append(&mut 1000_u32.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 100);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);
	});
}

#[test]
fn test_vesting_wallet_returns_0_if_non_exists() {
	new_test_ext().execute_with(|| {
		let (vesting_contract, _) = create_vesting_wallet_contract();

		let mut data = selector_from_str("get_unvested");
		data.append(&mut CHARLIE.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut CHARLIE.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_total");
		data.append(&mut CHARLIE.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("withdraw");
		data.append(&mut CHARLIE.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));
	});
}

#[test]
fn test_vesting_wallet_returns_correct_amounts() {
	new_test_ext().execute_with(|| {
		let vesting_contract = create_vesting_wallet_for_bob();
		let asset_id = get_asset_id_from_contract(vesting_contract.clone());

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 100);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		let mut data = selector_from_str("get_unvested");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_total");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		let mut data = selector_from_str("withdraw");
		data.append(&mut BOB.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		forward_by_blocks(100);

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 100);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		let mut data = selector_from_str("get_unvested");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(90));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(10));

		let mut data = selector_from_str("get_total");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		let mut data = selector_from_str("withdraw");
		data.append(&mut BOB.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(10));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 90);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 10);

		forward_by_blocks(900);

		let mut data = selector_from_str("get_unvested");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(90));

		let mut data = selector_from_str("get_total");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(90));

		let mut data = selector_from_str("withdraw");
		data.append(&mut BOB.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 0);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);

		// should do nothing right after
		let mut data = selector_from_str("get_unvested");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_total");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("withdraw");
		data.append(&mut BOB.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 0);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);

		forward_by_blocks(10);

		// should do nothing in the future
		let mut data = selector_from_str("get_unvested");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_available_for_withdraw");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("get_total");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(0));

		let mut data = selector_from_str("withdraw");
		data.append(&mut BOB.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		let mut data = selector_from_str("get_withdrawn");
		data.append(&mut BOB.encode());
		assert_eq!(call::<u128>(ALICE, vesting_contract.clone(), data), Ok(100));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 0);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);
	});
}
