//! Tests for Assets pallet.

use super::*;
use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungibles::InspectEnumerable, Currency},
};
use pallet_balances::Error as BalancesError;
use sp_runtime::{traits::ConvertInto, TokenError};

fn asset_ids() -> Vec<u32> {
	let mut s: Vec<_> = Assets::asset_ids().collect();
	s.sort();
	s
}

#[test]
fn basic_minting_should_work() {
	new_test_ext().execute_with(|| {
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::do_mint(0, &2, 100, Some(1)));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(200));
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn minting_too_many_insufficient_assets_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));
		assert_ok!(Assets::do_force_create(1, 1, false, 1));
		assert_ok!(Assets::do_force_create(2, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_ok!(Assets::do_mint(1, &1, 100, Some(1)));
		assert_noop!(Assets::do_mint(2, &1, 100, Some(1)), TokenError::CannotCreate);

		Balances::make_free_balance_be(&2, 1);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::do_mint(2, &1, 100, Some(1)));
		assert_eq!(asset_ids(), vec![0, 1, 2, 999]);
	});
}

#[test]
fn minting_insufficient_asset_with_deposit_should_work_when_consumers_exhausted() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));
		assert_ok!(Assets::do_force_create(1, 1, false, 1));
		assert_ok!(Assets::do_force_create(2, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_ok!(Assets::do_mint(1, &1, 100, Some(1)));
		assert_noop!(Assets::do_mint(2, &1, 100, Some(1)), TokenError::CannotCreate);

		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 2));
		assert_eq!(Balances::reserved_balance(&1), 10);

		assert_ok!(Assets::do_mint(2, &1, 100, Some(1)));
	});
}

#[test]
fn minting_insufficient_assets_with_deposit_without_consumer_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));
		assert_noop!(Assets::do_mint(0, &1, 100, Some(1)), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_eq!(System::consumers(&1), 0);
	});
}

#[test]
fn refunding_asset_deposit_with_nonzero_balance_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_noop!(Assets::refund(RuntimeOrigin::signed(1), 0), Error::<Test>::WouldBurn);
	});
}

#[test]
fn refunding_asset_deposit_with_zero_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));
		assert_noop!(Assets::do_mint(0, &1, 100, Some(1)), TokenError::CannotCreate);
		Balances::make_free_balance_be(&1, 100);
		assert_ok!(Assets::touch(RuntimeOrigin::signed(1), 0));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Assets::balance(0, 1), 0);
		assert_eq!(Balances::reserved_balance(&1), 10);
		assert_ok!(Assets::refund(RuntimeOrigin::signed(1), 0));
		assert_eq!(Balances::reserved_balance(&1), 0);
		assert_eq!(Assets::balance(1, 0), 0);
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		let e = Error::<Test>::Unapproved;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 51), e);
	});
}

#[test]
fn cannot_transfer_more_than_exists() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 101));
		let e = Error::<Test>::BalanceLow;
		assert_noop!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 101), e);
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(asset_id, 1, true, 1));
		assert!(Asset::<Test>::contains_key(asset_id));

		assert_ok!(Assets::set_metadata(RuntimeOrigin::signed(1), asset_id, vec![0], vec![0], 12));
		assert_eq!(Balances::reserved_balance(&1), 3);
		assert!(Metadata::<Test>::contains_key(asset_id));

		Balances::make_free_balance_be(&10, 100);
		assert_ok!(Assets::do_mint(asset_id, &10, 100, Some(1)));
		Balances::make_free_balance_be(&20, 100);
		assert_ok!(Assets::do_mint(asset_id, &20, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &10, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &2, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &3, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &4, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &5, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &6, 100, Some(1)));
		assert_ok!(Assets::do_mint(0, &7, 100, Some(1)));

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
fn non_providing_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, false, 1));

		Balances::make_free_balance_be(&0, 100);
		assert_ok!(Assets::do_mint(0, &0, 100, Some(1)));

		// ...cannot transfer...
		assert_noop!(
			Assets::transfer(RuntimeOrigin::signed(0), 0, 1, 50),
			TokenError::CannotCreate
		);
		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(0), 0, 1, 25));
		assert_eq!(asset_ids(), vec![0, 999]);
	});
}

#[test]
fn min_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 10));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Cannot create a new account with a balance that is below minimum...
		assert_noop!(Assets::do_mint(0, &2, 9, Some(1)), TokenError::BelowMinimum);
		assert_noop!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 9), TokenError::BelowMinimum);

		// When deducting from an account to below minimum, it should be reaped.
		// Death by `transfer`.
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 91));
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Assets::balance(0, 2), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `force_transfer`
		let f = TransferFlags { keep_alive: false, best_effort: false, burn_dust: false };
		let _ = Assets::do_transfer(0, &2, &1, 91, Some(1), f).map(|_| ());
		assert!(Assets::maybe_balance(0, 2).is_none());
		assert_eq!(Assets::balance(0, 1), 100);
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 1);

		// Death by `burn`.
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, 91, Some(1), flags);
		assert!(Assets::maybe_balance(0, 1).is_none());
		assert_eq!(Asset::<Test>::get(0).unwrap().accounts, 0);

		// Death by `transfer_approved`.
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));

		Balances::make_free_balance_be(&1, 1);
		assert_ok!(Assets::approve_transfer(RuntimeOrigin::signed(1), 0, 2, 100));
		assert_ok!(Assets::transfer_approved(RuntimeOrigin::signed(2), 0, 1, 3, 91));
	});
}

#[test]
fn querying_total_supply_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(2), 0, 3, 31));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 19);
		assert_eq!(Assets::balance(0, 3), 31);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &3, u64::MAX, Some(1), flags);
		assert_eq!(Assets::total_supply(0), 69);
	});
}

#[test]
fn transferring_amount_below_available_balance_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
	});
}

#[test]
fn transferring_enough_to_kill_source_when_keep_alive_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 10));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_noop!(
			Assets::transfer_ownership(RuntimeOrigin::signed(2), 0, 2),
			Error::<Test>::NoPermission
		);
		assert_noop!(
			Assets::set_team(RuntimeOrigin::signed(2), 0, 2, 2),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::do_mint(0, &2, 100, Some(2)), Error::<Test>::NoPermission);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		assert_noop!(Assets::do_burn(0, &1, u64::MAX, Some(2), flags), Error::<Test>::NoPermission);
		assert_noop!(
			Assets::start_destroy(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn transfer_owner_should_work() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 100);
		Balances::make_free_balance_be(&2, 100);
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_eq!(asset_ids(), vec![0, 999]);

		assert_ok!(Assets::transfer_ownership(RuntimeOrigin::signed(1), 0, 2));

		assert_noop!(
			Assets::transfer_ownership(RuntimeOrigin::signed(1), 0, 1),
			Error::<Test>::NoPermission
		);

		// Set metadata now and make sure that deposit gets transferred back.
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(2),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			12
		));
		assert_ok!(Assets::transfer_ownership(RuntimeOrigin::signed(2), 0, 1));
	});
}

#[test]
fn set_team_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::set_team(RuntimeOrigin::signed(1), 0, 2, 3));

		assert_ok!(Assets::do_mint(0, &2, 100, Some(2)));
	});
}

#[test]
fn transferring_amount_more_than_available_balance_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 50));
		assert_eq!(Assets::balance(0, 1), 50);
		assert_eq!(Assets::balance(0, 2), 50);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, Some(1), flags);

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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::balance(0, 1), 100);
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, 0));
		// `ForceCreated` and `Issued` but no `Transferred` event.
		assert_eq!(System::events().len(), 2);
	});
}

#[test]
fn transferring_more_units_than_total_supply_should_not_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(100));
		assert_eq!(Assets::balance(0, 1), 100);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		let _ = Assets::do_burn(0, &1, u64::MAX, Some(1), flags);
		assert_eq!(Assets::total_historical_supply(0, System::block_number()), Some(0));
		assert_eq!(Assets::balance(0, 1), 0);
	});
}

#[test]
fn burning_asset_balance_with_zero_balance_does_nothing() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
		assert_eq!(Assets::balance(0, 2), 0);
		let flags = DebitFlags { keep_alive: false, best_effort: true };
		assert_noop!(Assets::do_burn(0, &2, u64::MAX, Some(1), flags), Error::<Test>::NoAccount);
		assert_eq!(Assets::balance(0, 2), 0);
		assert_eq!(Assets::total_supply(0), 100);
	});
}

#[test]
fn set_metadata_should_work() {
	new_test_ext().execute_with(|| {
		// Cannot add metadata to unknown asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::Unknown,
		);
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		// Cannot add metadata to unowned asset
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(2), 0, vec![0u8; 10], vec![0u8; 10], 12),
			Error::<Test>::NoPermission,
		);

		// Cannot add oversized metadata
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 100], vec![0u8; 10], 12),
			Error::<Test>::BadMetadata,
		);
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 10], vec![0u8; 100], 12),
			Error::<Test>::BadMetadata,
		);

		// Successfully add metadata and take deposit
		Balances::make_free_balance_be(&1, 30);
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 10],
			12
		));
		assert_eq!(Balances::free_balance(&1), 9);

		// Update deposit
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 5],
			12
		));
		assert_eq!(Balances::free_balance(&1), 14);
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(1),
			0,
			vec![0u8; 10],
			vec![0u8; 15],
			12
		));
		assert_eq!(Balances::free_balance(&1), 4);

		// Cannot over-reserve
		assert_noop!(
			Assets::set_metadata(RuntimeOrigin::signed(1), 0, vec![0u8; 20], vec![0u8; 20], 12),
			BalancesError::<Test, _>::InsufficientBalance,
		);

		// Clear Metadata
		assert!(Metadata::<Test>::contains_key(0));
		assert_noop!(
			Assets::clear_metadata(RuntimeOrigin::signed(2), 0),
			Error::<Test>::NoPermission
		);
		assert_noop!(Assets::clear_metadata(RuntimeOrigin::signed(1), 1), Error::<Test>::Unknown);
		assert_ok!(Assets::clear_metadata(RuntimeOrigin::signed(1), 0));
		assert!(!Metadata::<Test>::contains_key(0));
	});
}

/// Destroying an asset works
#[test]
fn finish_destroy_asset_destroys_asset() {
	new_test_ext().execute_with(|| {
		assert_ok!(Assets::do_force_create(0, 1, true, 50));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));

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
fn balance_conversion_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::BalanceConversion;

		let id = 42;
		assert_ok!(Assets::do_force_create(id, 1, true, 10));
		let not_sufficient = 23;
		assert_ok!(Assets::do_force_create(not_sufficient, 1, false, 10));
		assert_eq!(asset_ids(), vec![23, 42, 999]);
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, 1234),
			Err(ConversionError::AssetMissing)
		);
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(
				100,
				not_sufficient
			),
			Err(ConversionError::AssetNotSufficient)
		);
		// 10 / 1 == 10 -> the conversion should 10x the value
		assert_eq!(
			BalanceToAssetBalance::<Balances, Test, ConvertInto>::to_asset_balance(100, id),
			Ok(100 * 10)
		);
	});
}

#[test]
fn assets_from_genesis_should_exist() {
	new_test_ext().execute_with(|| {
		assert_eq!(asset_ids(), vec![999]);
		assert!(Metadata::<Test>::contains_key(999));
		assert_eq!(Assets::balance(999, 1), 100);
		assert_eq!(Assets::total_supply(999), 100);
	});
}

#[test]
fn querying_allowance_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::approvals::{Inspect, Mutate};
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, 100, Some(1)));
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
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::do_mint(0, &1, amount, Some(1)));
		assert_ok!(Assets::transfer(RuntimeOrigin::signed(1), 0, 2, amount - 1));
	})
}

#[test]
fn querying_roles_should_work() {
	new_test_ext().execute_with(|| {
		use frame_support::traits::tokens::fungibles::roles::Inspect;
		assert_ok!(Assets::do_force_create(0, 1, true, 1));
		assert_ok!(Assets::set_team(
			RuntimeOrigin::signed(1),
			0,
			// Issuer
			2,
			// Admin
			3,
		));
		assert_eq!(Assets::owner(0), Some(1));
		assert_eq!(Assets::issuer(0), Some(2));
		assert_eq!(Assets::admin(0), Some(3));
	});
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
fn supply_history_query_historic_blocks_should_work() {
	new_test_ext().execute_with(|| {
		let asset_id = 95;
		let account_id = 32;
		let account_id2 = 974;
		let amount = 345;
		let burn_amount = 127;
		let start_block = System::block_number();
		let mut block = start_block;
		assert_eq!(Assets::total_historical_supply(asset_id, block), Some(0));
		assert_ok!(Assets::do_force_create(asset_id, account_id, true, 1));
		assert_ok!(Assets::do_mint(asset_id, &account_id, amount, None));
		assert_eq!(Assets::total_historical_supply(asset_id, block), Some(amount));
		assert_ok!(Assets::do_mint(asset_id, &account_id2, amount, None));
		assert_eq!(Assets::total_historical_supply(asset_id, block), Some(2 * amount));
		block += 5;
		run_to_block(block);
		assert_eq!(Assets::total_historical_supply(asset_id, block - 3), Some(2 * amount));
		assert_eq!(Assets::total_historical_supply(asset_id, start_block), Some(2 * amount));
		let flags = DebitFlags { keep_alive: false, best_effort: false};
		assert_ok!(Assets::do_burn(asset_id, &account_id2, burn_amount, None, flags));
		assert_eq!(Assets::total_historical_supply(asset_id, block - 3), Some(2 * amount));
		assert_eq!(Assets::total_historical_supply(asset_id, block), Some(2 * amount - burn_amount));
		assert_eq!(Assets::total_historical_supply(asset_id, start_block), Some(2 * amount));
		block += 19;
		run_to_block(block);
		assert_eq!(Assets::total_historical_supply(asset_id, block - 3), Some(2 * amount - burn_amount));
		assert_eq!(Assets::total_historical_supply(asset_id, start_block), Some(2 * amount));
		assert_ok!(Assets::do_burn(asset_id, &account_id2, burn_amount, None, flags));
		assert_eq!(Assets::total_historical_supply(asset_id, block - 3), Some(2 * amount - burn_amount));
		assert_eq!(Assets::total_historical_supply(asset_id, block), Some(2 * amount - 2 * burn_amount));
		assert_eq!(Assets::total_historical_supply(asset_id, start_block), Some(2 * amount));
	})
}
