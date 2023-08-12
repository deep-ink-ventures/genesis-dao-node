use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use codec::Encode;

#[test]
fn register_global_callback() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        let contract = CONTRACT;
        let origin = RuntimeOrigin::signed(ALICE);
        assert_ok!(HookPoints::register_global_callback(origin, contract));
        assert_eq!(HookPoints::callbacks(ALICE), Some(CONTRACT));
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
        assert_eq!(HookPoints::specific_callbacks(ALICE, id), Some(CONTRACT));
        System::assert_last_event(Event::SpecificCallbackRegistered { who: ALICE }.into());
    });
}


#[test]
fn get_callback_returns_specific_callback() {
    new_test_ext().execute_with(|| {
        let id: Vec<u8> = b"specific_cb".to_vec();

        // Register a specific callback for ALICE
        assert_ok!(HookPoints::register_specific_callback(
            RuntimeOrigin::signed(ALICE).into(),
            CONTRACT,
            id.clone()
        ));

        // Now retrieve the callback using `get_callback`
        let result = HookPoints::get_callback(&ALICE, id);
        assert_eq!(result, Some(CONTRACT));
    });
}

#[test]
fn get_callback_returns_global_callback_if_specific_not_found() {
    new_test_ext().execute_with(|| {
        let specific_id: Vec<u8> = b"specific_cb".to_vec();
        let unknown_id: Vec<u8> = b"unknown_cb".to_vec();

        // Register a specific callback and a global callback for ALICE
        assert_ok!(HookPoints::register_specific_callback(
            RuntimeOrigin::signed(ALICE).into(),
            CONTRACT,
            specific_id.clone()
        ));

        assert_ok!(HookPoints::register_global_callback(
            RuntimeOrigin::signed(ALICE).into(),
            CONTRACT,
        ));

        // Retrieving a callback with an unknown id should return the global callback
        let result = HookPoints::get_callback(&ALICE, unknown_id);
        assert_eq!(result, Some(CONTRACT));
    });
}

#[test]
fn get_callback_returns_none_if_no_callbacks_registered() {
    new_test_ext().execute_with(|| {
        let unknown_id: Vec<u8> = b"unknown_cb".to_vec();

        // No callbacks registered for ALICE
        let result = HookPoints::get_callback(&ALICE, unknown_id);
        assert_eq!(result, None);
    });
}

#[test]
fn it_creates_hook_point_from_hookpoints() {
    let hook_point = HookPoints::create(CALLBACK_NAME, ALICE, BOB);

    // Assertions
    assert_eq!(hook_point.owner, ALICE);
    assert_eq!(hook_point.origin, BOB);
    assert_eq!(hook_point.callback, CALLBACK_NAME.as_bytes().to_vec());

    // Check the first 4 bytes of the hashed callback name
    let hash = sp_core::blake2_256(CALLBACK_NAME.as_bytes());
    let expected_data = vec![hash[0], hash[1], hash[2], hash[3]];
    assert_eq!(hook_point.data, expected_data);
}

#[test]
fn it_appends_argument_to_hook_point_from_hookpoints() {
    let mut hook_point = HookPoints::create(CALLBACK_NAME, ALICE, BOB);

    // Starting with the initial data from the callback hash
    let initial_data = hook_point.data.clone();

    // Add an argument to the hook point
    let arg: u32 = 42;
    hook_point = hook_point.add_arg(arg);

    // Expected data
    let mut expected_data = initial_data;
    expected_data.append(&mut arg.encode());

    assert_eq!(hook_point.data, expected_data);
}


#[test]
fn install_and_call_ink_contract_works() {
    new_test_ext().execute_with(|| {
        let creator = ALICE;
        let contract_path = "test_contract.wasm";

		let mut data = 0x9bae9d5e_u32.to_be_bytes().to_vec();
		data.append(&mut 42u128.encode()); // argument DaoId
        let salt = vec![];

        // Attempt to install the contract
        let contract_address = HookPoints::install(
            creator.clone(),
            std::fs::read(contract_path).unwrap(),
            data.clone(),
            salt.clone()
        ).expect("Contract installation should be successful");

        // Register the contract for callbacks (if you have such a step)
        HookPoints::register_global_callback(RuntimeOrigin::signed(creator.clone()), contract_address.clone()).unwrap();

        // Create a HookPoint for the "get" function
        let hookpoint = HookPoints::create("get", creator.clone(), creator.clone());

        // Execute the "get" function using the HookPoint
        let result: Result<u128, _> = HookPoints::execute(hookpoint);

        // Ensure the result is Ok and equals to 42
        assert_eq!(result.unwrap(), 42);
    });
}


#[test]
fn execute_callback() {
    new_test_ext().execute_with(|| {
        let creator = ALICE;
        let contract_path = "test_contract.wasm";

		let mut data = 0x9bae9d5e_u32.to_be_bytes().to_vec();
		data.append(&mut 42u128.encode()); // argument DaoId
        let salt = vec![];

        // Attempt to install the contract
        let contract_address = HookPoints::install(
            creator.clone(),
            std::fs::read(contract_path).unwrap(),
            data.clone(),
            salt.clone()
        ).expect("Contract installation should be successful");

        // Register the contract for callbacks (if you have such a step)
        HookPoints::register_global_callback(RuntimeOrigin::signed(creator.clone()), contract_address.clone()).unwrap();

        // Create a HookPoint for the "get" function
        let hookpoint = HookPoints::create("multiply", creator.clone(), creator.clone())
            .add_arg(2u128);

        // Execute the "get" function using the HookPoint
        let result: Result<u128, _> = HookPoints::execute(hookpoint);

        // Ensure the result is Ok and equals to 42
        assert_eq!(result.unwrap(), 84);
    });
}

