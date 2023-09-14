use crate::mock::*;
use codec::Encode;
use frame_support::assert_ok;

#[test]
fn test_init() {
	new_test_ext().execute_with(|| {
		let dao = DaoCore::load_dao(create_dao()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		let mut data = selector_from_str("new");
		data.append(&mut asset_id.clone().encode());
		let asset_contract = install(ALICE, ASSET_CONTRACT_PATH, data).expect("code deployed");

		let asset_id_on_chain =
			call::<u32>(ALICE, asset_contract.clone(), selector_from_str("get_asset_id"))
				.expect("call success");

		assert_eq!(asset_id, asset_id_on_chain);
	});
}

#[test]
fn test_transfer_extension() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;

		let dao = DaoCore::load_dao(create_dao()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		let mut data = selector_from_str("new");
		data.append(&mut asset_id.clone().encode());
		let contract_address =
			install(sender.clone(), ASSET_CONTRACT_PATH, data).expect("code deployed");

		let mut data = selector_from_str("PSP22::transfer");
		data.append(&mut BOB.clone().encode());
		data.append(&mut 100_u128.encode());
		data.append(&mut "empty".encode());
		call::<()>(sender.clone(), contract_address.clone(), data).expect("call success");

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);

		// native funcs
		let mut data = selector_from_str("PSP22::balance_of");
		data.append(&mut ALICE.encode());
		let alice_balance =
			call::<u128>(sender.clone(), contract_address.clone(), data).expect("call success");
		assert_eq!(alice_balance, 900);

		let mut data = selector_from_str("PSP22::balance_of");
		data.append(&mut BOB.encode());
		let bob_balance =
			call::<u128>(sender.clone(), contract_address.clone(), data).expect("call success");
		assert_eq!(bob_balance, 100);

		assert_eq!(
			call::<u128>(
				sender.clone(),
				contract_address,
				selector_from_str("PSP22::total_supply")
			)
			.unwrap(),
			1000
		);
	});
}

#[test]
fn test_transfer_keep_alive_extension() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;

		let dao = DaoCore::load_dao(create_dao()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		let mut data = selector_from_str("new");
		data.append(&mut asset_id.clone().encode());

		let contract_address =
			install(sender.clone(), ASSET_CONTRACT_PATH, data).expect("code deployed");

		let mut data = selector_from_str("transfer_keep_alive");
		data.append(&mut BOB.clone().encode());
		data.append(&mut 1000_u128.encode());

		assert!(call::<()>(sender.clone(), contract_address, data).is_err());

		// `transfer_keep_alive` failed, so the balances should not change.

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);
	});
}

#[test]
fn test_approved_transfer_flow_extension() {
	new_test_ext().execute_with(|| {
		let dao = DaoCore::load_dao(create_dao()).unwrap();
		let asset_id = dao.asset_id.unwrap();

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

		let mut data = selector_from_str("new");
		data.append(&mut asset_id.clone().encode());

		let contract_address = install(ALICE, ASSET_CONTRACT_PATH, data).expect("code deployed");

		// Alice didn't allow anything
		let mut data = selector_from_str("PSP22::allowance");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut BOB.clone().encode());
		assert_eq!(call::<u128>(ALICE, contract_address.clone(), data).unwrap(), 0);

		// ALICE approves 100 to BOB
		let mut data = selector_from_str("PSP22::approve");
		data.append(&mut BOB.clone().encode());
		data.append(&mut 100_u128.encode());
		assert_ok!(call::<()>(ALICE, contract_address.clone(), data));

		let mut data = selector_from_str("PSP22::allowance");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut BOB.clone().encode());
		assert_eq!(call::<u128>(ALICE, contract_address.clone(), data).unwrap(), 100);

		// BOB transfers 10 from ALICE to CHARLIE
		let mut data = selector_from_str("PSP22::transfer_from");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut CHARLIE.clone().encode());
		data.append(&mut 10_u128.encode());
		data.append(&mut "empty".encode());
		assert_ok!(call::<()>(BOB, contract_address.clone(), data));

		assert_eq!(Assets::balance(asset_id.clone(), ALICE), 990);
		assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);
		assert_eq!(Assets::balance(asset_id.clone(), CHARLIE), 10);

		// BOB should still have 90 open
		let mut data = selector_from_str("PSP22::allowance");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut BOB.clone().encode());
		assert_eq!(call::<u128>(BOB, contract_address.clone(), data).unwrap(), 90);

		// ALICE decreases by 80 to 10
		let mut data = selector_from_str("PSP22::decrease_allowance");
		data.append(&mut BOB.clone().encode());
		data.append(&mut 80_u128.encode());

		assert_ok!(call::<()>(ALICE, contract_address.clone(), data));

		let mut data = selector_from_str("PSP22::allowance");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut BOB.clone().encode());
		assert_eq!(call::<u128>(ALICE, contract_address.clone(), data).unwrap(), 10);

		// BOB transfers 20 from ALICE to CHARLIE, but he can't
		let sender = BOB;
		let mut data = selector_from_str("PSP22::transfer_from");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut CHARLIE.clone().encode());
		data.append(&mut 20_u128.encode());
		data.append(&mut "empty".encode());

		assert!(call::<()>(sender.clone(), contract_address.clone(), data).is_err());

		// ALICE increases by 42
		let sender = ALICE;
		let mut data = selector_from_str("PSP22::increase_allowance");
		data.append(&mut BOB.clone().encode());
		data.append(&mut 42_u128.encode());

		assert_ok!(call::<()>(sender.clone(), contract_address.clone(), data));

		let mut data = selector_from_str("PSP22::allowance");
		data.append(&mut ALICE.clone().encode());
		data.append(&mut BOB.clone().encode());
		// 10 where still around, so 10 + 42 = 52
		assert_eq!(call::<u128>(sender.clone(), contract_address, data).unwrap(), 52);
	});
}
