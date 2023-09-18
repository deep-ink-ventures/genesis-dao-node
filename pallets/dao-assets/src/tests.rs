//! Tests for Assets pallet.

use super::*;
use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungibles::InspectEnumerable, Currency},
};
use pallet_balances::Error as BalancesError;
use sp_runtime::TokenError;
use std::collections::BTreeMap;

fn asset_ids() -> Vec<u32> {
	let mut s: Vec<_> = Assets::asset_ids().collect();
	s.sort();
	s
}

// comapre unsorted arrays
macro_rules! assert_array {
	($h1:expr, $h2:expr) => {
		let mut h1 = $h1.to_vec();
		let mut h2 = $h2.to_vec();
		h1.sort();
		h2.sort();
		assert_eq!(h1, h2);
	};
}

const ALICE: u64 = 1;
const BOB: u64 = 2;
const CHARLIE: u64 = 3;

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::do_mint(0, &2, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(200));
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn approval_lifecycle_works() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 40));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 60);
		assert_eq!(Assets::balance(0, 3), 40);
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn transfer_approved_all_funds() {
	new_test_ext().execute_with(|| {
		// can't approve non-existent token
		assert_noop!(
			Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50),
			Error::<Test>::Unknown
		);
		// so we create it :)
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_eq!(Balances::reserved_balance(&1), 1);

		// transfer the full amount, which should trigger auto-cleanup
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 3), 50);
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn approval_deposits_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		let e = BalancesError::<Test>::InsufficientBalance;
		assert_noop!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50), e);

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Balances::reserved_balance(&1), 1);

		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 50));
		assert_eq!(Balances::reserved_balance(&1), 0);

		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Balances::reserved_balance(&1), 0);
	});
}

#[test]
fn cannot_transfer_more_than_approved() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		let e = Error::<Test>::Unapproved;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 51), e);
	});
}

#[test]
fn cannot_transfer_more_than_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 101));
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 101), e);
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 1, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(2), 0, 2),
			Error::<Test>::Unknown
		);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 3),
			Error::<Test>::Unknown
		);
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 1);
		assert_ok!(Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2));
		assert_eq!(Asset::<Test>::get(0).unwrap().approvals, 0);
		assert_noop!(
			Assets::cancel_approval(RuntimeOrigin::signed(1), 0, 2),
			Error::<Test>::Unknown
		);
	});
}

#[test]
fn lifecycle_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		let asset_id = 37;
		assert_ok!(Assets::do_force_create(asset_id, 1, 1));
		assert!(Asset::<Test>::contains_key(asset_id));

		assert_ok!(Assets::do_set_metadata(asset_id, &1, vec![0], vec![0], 10));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert!(Metadata::<Test>::contains_key(asset_id));

		Balances::make_free_balance_be(&10, 100);
		assert_ok!(Assets::do_mint(asset_id, &10, 100));
		Balances::make_free_balance_be(&20, 100);
		assert_ok!(Assets::do_mint(asset_id, &20, 100));
		assert_eq!(Account::<Test>::iter_prefix(asset_id).count(), 2);

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), asset_id));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), asset_id));

		assert_eq!(Balances::reserved_balance(&1), 0);

		assert!(Asset::<Test>::get(asset_id).unwrap().status == AssetStatus::Destroyed);
		assert!(!Metadata::<Test>::contains_key(asset_id));
		assert_eq!(Account::<Test>::iter_prefix(0).count(), 0);
	});
}

#[test]
fn destroy_should_refund_approvals() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &10, 100));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 3, 50));
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 4, 50));
		assert_eq!(Balances::reserved_balance(&1), 3);
		assert_eq!(asset_ids(), vec![0, 999]);

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		assert_eq!(Balances::reserved_balance(&1), 0);

		// all approvals are removed
		assert!(Approvals::<Test>::iter().count().is_zero())
	});
}

#[test]
fn partial_destroy_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_ok!(Assets::do_mint(0, &2, 100));
		assert_ok!(Assets::do_mint(0, &3, 100));
		assert_ok!(Assets::do_mint(0, &4, 100));
		assert_ok!(Assets::do_mint(0, &5, 100));
		assert_ok!(Assets::do_mint(0, &6, 100));
		assert_ok!(Assets::do_mint(0, &7, 100));

		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		// Asset is in use, as all the accounts have not yet been destroyed.
		// We need to call destroy_accounts or destroy_approvals again until asset is completely
		// cleaned up.
		assert_noop!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0), Error::<Test>::InUse);

		System::assert_has_event(RuntimeEvent::Assets(crate::Event::AccountsDestroyed {
			asset_id: 0,
			accounts_destroyed: 5,
			accounts_remaining: 2,
		}));
		System::assert_has_event(RuntimeEvent::Assets(crate::Event::ApprovalsDestroyed {
			asset_id: 0,
			approvals_destroyed: 0,
			approvals_remaining: 0,
		}));
		// Partially destroyed Asset should continue to exist
		assert!(Asset::<Test>::contains_key(0));

		// Second call to destroy on PartiallyDestroyed asset
		assert_ok!(Assets::destroy_accounts(RuntimeOrigin::signed(1), 0));
		System::assert_has_event(RuntimeEvent::Assets(crate::Event::AccountsDestroyed {
			asset_id: 0,
			accounts_destroyed: 2,
			accounts_remaining: 0,
		}));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::destroy_approvals(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		System::assert_has_event(RuntimeEvent::Assets(crate::Event::Destroyed { asset_id: 0 }));

		// Destroyed Asset should not exist
		assert!(Asset::<Test>::get(0).unwrap().status == AssetStatus::Destroyed);
	})
}

#[test]
fn min_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 10));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Cannot create a new account with a balance that is below minimum...
		assert_noop!(Assets::do_mint(0, &2, 9), TokenError::BelowMinimum);
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 9), TokenError::BelowMinimum);

		// When deducting from an account to below minimum, it should be reaped.
		// Death by `transfer`.
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 91));
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `force_transfer`
		let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
		let _ = Assets::do_transfer(0, &2, &1, 91, f).map(|_| ());
		assert!(Assets::maybe_balance(0, 2).is_none());
		assert_eq!(Assets::balance(0, 1), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `burn`.
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, 91, flags);
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 0);

		// Death by `transfer_approved`.
		assert_ok!(Assets::do_mint(0, &1, 100));

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 91));
	});
}

#[test]
fn querying_total_supply_should_work() {
	let asset_id = 7;
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(asset_id, 1, 1));
		assert_ok!(Assets::do_mint(asset_id, &1, 100));
		assert_eq!(Assets::balance(asset_id, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), asset_id, 2, 50));
		assert_eq!(Assets::balance(asset_id, 1), 50);
		assert_eq!(Assets::balance(asset_id, 2), 50);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), asset_id, 3, 31));
		assert_eq!(Assets::balance(asset_id, 1), 50);
		assert_eq!(Assets::balance(asset_id, 2), 19);
		assert_eq!(Assets::balance(asset_id, 3), 31);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(asset_id, &3, u64::MAX, flags);
		assert_eq!(Assets::total_supply(asset_id), 69);
	});
}

#[test]
fn transferring_amount_below_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
	});
}

#[test]
fn transferring_enough_to_kill_source_when_keep_alive_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 10));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 91),
			Error::<Test>::BalanceLow
		);
		assert_ok!(Assets::transfer_keep_alive(RuntimeOrigin::signed(1), 0, 2, 90));
		assert_eq!(Assets::balance(0, 1), 10);
		assert_eq!(Assets::balance(0, 2), 90);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn origin_guards_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_noop!(
			Assets::start_destroy(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, flags);
		assert_eq!(Assets::balance(0, 1), 0);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 1, 50),
			Error::<Test>::NoAccount
		);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(2), 0, 1, 51),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn transferring_less_than_one_unit_is_fine() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 0));
		// `ForceCreated` and `Issued` but no `Transferred` event.
		assert_eq!(System::events().len(), 2);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 101),
			Error::<Test>::BalanceLow
		);
	});
}

#[test]
fn burning_asset_balance_with_positive_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, flags);
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_eq!(Assets::balance(0, 1), 0);
	});
}

#[test]
fn burning_asset_balance_with_zero_balance_does_nothing() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		assert_eq!(Assets::balance(0, 2), 0);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		assert_noop!(Assets::do_burn(0, &2, u64::MAX, flags), Error::<Test>::NoAccount);
		assert_eq!(Assets::balance(0, 2), 0);
		assert_eq!(Assets::total_supply(0), 100);
	});
}

#[test]
fn set_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// Cannot add metadata to unknown asset
		assert_noop!(
			Assets::do_set_metadata(0, &1, vec![0u8; 10], vec![0u8; 10], 10),
			Error::<Test>::Unknown,
		);
		assert_ok!(Assets::do_force_create(0, 1, 1));
		// Cannot add metadata to unowned asset
		assert_noop!(
			Assets::do_set_metadata(0, &2, vec![0u8; 10], vec![0u8; 10], 10),
			Error::<Test>::NoPermission,
		);

		// Cannot add oversized metadata
		assert_noop!(
			Assets::do_set_metadata(0, &1, vec![0u8; 100], vec![0u8; 10], 10),
			Error::<Test>::BadMetadata,
		);
		assert_noop!(
			Assets::do_set_metadata(0, &1, vec![0u8; 10], vec![0u8; 100], 10),
			Error::<Test>::BadMetadata,
		);

		// Successfully add metadata
		assert_ok!(Assets::do_set_metadata(0, &1, vec![0u8; 10], vec![0u8; 10], 10));
	});
}

/// Destroying an asset works
#[test]
fn finish_destroy_asset_destroys_asset() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 50));
		// Destroy the accounts.
		assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::finish_destroy(RuntimeOrigin::signed(1), 0));

		// Asset is gone
		assert!(Asset::<Test>::get(0).unwrap().status == AssetStatus::Destroyed);
	})
}

#[test]
fn imbalances_should_work() {
	use frame_support::traits::tokens::fungibles::Balanced;

	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, 1));

		let imb = Assets::issue(0, 100);
		assert_eq!(Assets::total_supply(0), 100);
		assert_eq!(imb.peek(), 100);

		let (imb1, imb2) = imb.split(30);
		assert_eq!(imb1.peek(), 30);
		assert_eq!(imb2.peek(), 70);

		drop(imb2);
		assert_eq!(Assets::total_supply(0), 30);

		assert!(Assets::resolve(&1, imb1).is_ok());
		assert_eq!(Assets::balance(0, 1), 30);
		assert_eq!(Assets::total_supply(0), 30);
	});
}

#[test]
fn assets_from_genesis_should_exist() {
	new_test_ext().execute_with(|| {
		assert_eq!(asset_ids(), vec![999]);
		assert!(Metadata::<Test>::contains_key(999));
		assert_eq!(Assets::balance(999, ALICE), 100);
		assert_eq!(Assets::total_supply(999), 1200);
	});
}

#[test]
fn querying_allowance_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::approvals::Mutate;
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, 100));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve(0, &1, &2, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 50);
		// Transfer asset 0, from owner 1 and delegate 2 to destination 3
		assert_ok!(Assets::transfer_from(0, &1, &2, &3, 50));
		assert_eq!(Assets::allowance(0, &1, &2), 0);
	});
}

#[test]
fn transfer_large_asset() {
	new_test_ext().execute_with(|| {
		let amount = u64::pow(2, 63) + 2;
		assert_ok!(Assets::do_force_create(0, 1, 1));
		assert_ok!(Assets::do_mint(0, &1, amount));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, amount - 1));
	})
}

#[test]
fn reserving_and_unreserving_should_work() {
	new_test_ext().execute_with(|| {
		// establish situation before
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::reserved(999, 1), 0);

		// do reservation
		assert_ok!(Assets::do_reserve(999, 1, 40));

		// check reservation worked
		assert_eq!(Assets::balance(999, 1), 60);
		assert_eq!(Assets::reserved(999, 1), 40);

		// undo reservation
		assert_ok!(Assets::do_unreserve(999, 1, 10));

		// check undoing reservation worked
		assert_eq!(Assets::balance(999, 1), 70);
		assert_eq!(Assets::reserved(999, 1), 30);

		// undo the maximum
		assert_ok!(Assets::do_unreserve(999, 1, 1500));

		// check undoing reservation worked
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::reserved(999, 1), 0);
	})
}

fn run_to_block(n: u64) {
	use frame_support::traits::{OnFinalize, OnInitialize};
	while System::block_number() < n {
		let mut block = System::block_number();
		Assets::on_finalize(block);
		System::on_finalize(block);
		System::reset_events();
		block += 1;
		System::set_block_number(block);
		System::on_initialize(block);
		Assets::on_initialize(block);
	}
}

#[test]
fn checkpoint_behaviour_ok() {
	let total_amount =
		|checkpoint: &CheckpointOf<Test>| checkpoint.delegated_amount() + checkpoint.mutated;

	new_test_ext().execute_with(|| {
		let mut alice_checkpoint = CheckpointOf::<Test> {
			mutated: 100,
			delegated: BTreeMap::from([(BOB, 50_u32.into()), (CHARLIE, 10_u32.into())])
				.try_into()
				.unwrap(),
			total_delegation: 60,
		};

		let mut bob_checkpoint = CheckpointOf::<Test> { mutated: 200, ..Default::default() };

		// Total amount gives right value
		assert_eq!(total_amount(&alice_checkpoint), 160);
		assert_eq!(total_amount(&bob_checkpoint), 200);

		let old_alice_delegated = alice_checkpoint.delegated.clone();
		// can add to mutated
		assert_eq!(
			alice_checkpoint.delegate_to(&ALICE, &mut bob_checkpoint),
			Some(()),
			"max delegation reached"
		);
		assert_eq!(total_amount(&alice_checkpoint), 60);
		assert_eq!(total_amount(&bob_checkpoint), 300);
		assert_eq!(alice_checkpoint.delegated, old_alice_delegated);
		assert_eq!(bob_checkpoint.delegated, BTreeMap::from([(ALICE, 100)]));
		assert_eq!(bob_checkpoint.delegated_amount(), &100);
	});

	new_test_ext().execute_with(|| {
		const ASSET_ID: u32 = 999;

		let alice_checkpoint = CheckpointOf::<Test> {
			mutated: 100,
			delegated: std::collections::BTreeMap::from([
				(BOB, 50_u32.into()),
				(CHARLIE, 10_u32.into()),
			])
			.try_into()
			.unwrap(),
			..Default::default()
		};
		AccountHistory::<Test>::insert((ASSET_ID, ALICE), 5_u64, alice_checkpoint.clone());

		run_to_block(10);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(BOB), ASSET_ID, ALICE, 50));
		let new_checkpoint = AccountHistory::<Test>::get((ASSET_ID, ALICE), 10).unwrap();

		assert_eq!(total_amount(&new_checkpoint), total_amount(&alice_checkpoint) + 50);
		assert_eq!(new_checkpoint.delegated, alice_checkpoint.delegated);
	});
}

#[test]
fn account_history_is_ok() {
	new_test_ext().execute_with(|| {
		let asset_id = 999;

		let asset_balance = |accnt| Assets::balance(asset_id, accnt);
		let account_history_mutated = |account: AccountId| {
			AccountHistory::<Test>::iter()
				.filter_map(move |((asset, acnt), bl_num, chp)| {
					(asset_id == asset && acnt == account).then_some((bl_num, chp.mutated))
				})
				.collect::<Vec<_>>()
		};

		run_to_block(10);
		assert_eq!(asset_balance(ALICE), 100);
		assert_eq!(asset_balance(BOB), 300);
		assert_eq!(asset_balance(CHARLIE), 500);

		assert_ok!(Assets::transfer(RuntimeOrigin::signed(ALICE), asset_id, BOB, 50));
		assert_eq!(asset_balance(ALICE), 50);
		assert_eq!(asset_balance(BOB), 350);

		run_to_block(50);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(BOB), asset_id, CHARLIE, 10));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(ALICE), asset_id, BOB, 10));
		run_to_block(52);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(CHARLIE), asset_id, ALICE, 50));

		assert_array!(account_history_mutated(ALICE), vec![(10, 50), (52, 90)]);
	});
}
