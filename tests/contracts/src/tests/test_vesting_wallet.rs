use crate::mock::*;

#[test]
fn test_vesting_wallet_lifecycle() {
	new_test_ext().execute_with(|| {
		let (vesting_contract, asset_contract) = create_vesting_wallet();

		let data = selector_from_str("get_token");
		let account_id = call::<AccountId>(ALICE, vesting_contract.clone(), data).expect("call success");
		assert_eq!(account_id, asset_contract);
	});
}

