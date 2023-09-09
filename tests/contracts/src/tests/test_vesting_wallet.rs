use codec::Encode;
use frame_support::assert_ok;
use crate::mock::*;

#[test]
fn test_create_vesting_wallet() {
	new_test_ext().execute_with(|| {
		let (vesting_contract, asset_contract) = create_vesting_wallet();

		// check that the vesting contract has the correct token
		let account_id = call::<AccountId>(
			ALICE,
			vesting_contract.clone(),
			selector_from_str("get_token")
		).expect("call success");
		assert_eq!(account_id, asset_contract);

		let asset_id = call::<u32>(
			ALICE,
			asset_contract.clone(),
			selector_from_str("get_asset_id")
		).expect("call success");

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

		// unable to fund becaue 101 > 100
		let mut data = selector_from_str("create_vesting_wallet_for");
		data.append(&mut BOB.encode());
		data.append(&mut 101_u128.encode());
		data.append(&mut 1000_u64.encode());
		assert!(call::<()>(ALICE, vesting_contract.clone(), data).is_err());

		let mut data = selector_from_str("create_vesting_wallet_for");
		data.append(&mut BOB.encode());
		data.append(&mut 100_u128.encode());
		data.append(&mut 1000_u64.encode());
		assert_ok!(call::<()>(ALICE, vesting_contract.clone(), data));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), vesting_contract.clone()), 100);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

	});
}

