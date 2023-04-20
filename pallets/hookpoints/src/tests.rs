use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};

#[test]
fn register_global_callback() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let contract = 42;
		let who = 1;
		let origin = RuntimeOrigin::signed(who);
		assert_ok!(HookPoints::register_global_callback(origin, contract));
		assert_eq!(HookPoints::callbacks(who), Some(42));
		System::assert_last_event(Event::GlobalCallbackRegistered { who }.into());
	});
}

#[test]
fn register_specific_callback() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let contract = 42;
		let who = 1;
		let origin = RuntimeOrigin::signed(who);

		// registration should fail when id is too long
		let id = b"registration id is constrained not to exceed MaxLengthId characters, so this registration id is likely too long".to_vec();
		assert_noop!(HookPoints::register_specific_callback(origin.clone(), contract, id.clone()),
					 Error::<Test>::IdTooLong);

		// registration should work
		let id = b"contract".to_vec();
		assert_ok!(HookPoints::register_specific_callback(origin, contract, id.clone()));
		let id: BoundedVec<_, _> = id.try_into().unwrap();
		assert_eq!(HookPoints::named_callbacks(who, id), Some(42));
		System::assert_last_event(Event::SpecificCallbackRegistered { who }.into());
	});
}
