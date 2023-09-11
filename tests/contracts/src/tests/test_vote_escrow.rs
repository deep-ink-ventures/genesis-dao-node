use crate::mock::*;

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
	});
}
