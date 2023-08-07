use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn register_global_callback() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let contract = CONTRACT;
		let origin = RuntimeOrigin::signed(ALICE);
		assert_ok!(HookPoints::register_global_callback(origin, contract));
		assert_eq!(HookPoints::callbacks(ALICE), Some(ALICE));
		System::assert_last_event(Event::GlobalCallbackRegistered { who: ALICE }.into());
	});
}

#[test]
fn register_specific_callback() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let contract = CONTRACT;
		let who = ALICE;
		let origin = RuntimeOrigin::signed(who);

		// registration should fail when id is too long
		let id = b"registration id is constrained not to exceed MaxLengthId characters, so this registration id is likely too long".to_vec();
		assert_noop!(HookPoints::register_specific_callback(origin.clone(), contract.clone(), id.clone()),
					 Error::<Test>::IdTooLong);

		// registration should work
		let id = b"contract".to_vec();
		assert_ok!(HookPoints::register_specific_callback(origin, contract, id.clone()));
		let id: BoundedVec<_, _> = id.try_into().unwrap();
		assert_eq!(HookPoints::specific_callbacks(ALICE, id), Some(ALICE));
		System::assert_last_event(Event::SpecificCallbackRegistered { who: ALICE }.into());
	});
}

#[test]
fn hookpoint_execution() {
	new_test_ext().execute_with(|| {
		let sender = ALICE;
		let origin = RuntimeOrigin::signed(sender.clone());

		let contract_path = "./contract/contract.wasm";
		let mut data = 0x9bae9d5e_u32.to_be_bytes().to_vec();
		data.append(&mut 2_u32.to_be_bytes().to_vec());

		let contract = HookPoints::install(
			ALICE,
			std::fs::read(contract_path).unwrap(),
			data,
			vec![]
		).expect("code deployed");

		assert_ok!(HookPoints::register_specific_callback(
			origin,
			contract,
			"contract::multiply".into(),
		));

		let hp = HookPoints::create("contract::multiply", ALICE, ALICE)
			.add_arg::<u32>(16);

		let exec =  HookPoints::execute::<u32>(hp);

		assert_eq!(32, exec.unwrap());

	})

}