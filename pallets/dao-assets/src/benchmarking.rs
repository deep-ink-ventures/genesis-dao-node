//! Assets pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin as SystemOrigin;

use crate::Pallet as Assets;

const SEED: u32 = 0;

fn default_asset_id<T: Config>() -> T::AssetIdParameter {
	T::BenchmarkHelper::create_asset_id_parameter(0)
}

fn create_default_asset<T: Config>() -> (T::AssetIdParameter, T::AccountId) {
	let asset_id = default_asset_id::<T>();
	let caller: T::AccountId = whitelisted_caller();
	assert!(Assets::<T>::do_force_create(asset_id.into(), caller.clone(), 1u32.into(),).is_ok());
	(asset_id, caller)
}

fn create_default_minted_asset<T: Config>(
	amount: T::Balance,
) -> (T::AssetIdParameter, T::AccountId) {
	let (asset_id, caller) = create_default_asset::<T>();
	assert!(Assets::<T>::do_mint(asset_id.into(), &caller, amount).is_ok());
	(asset_id, caller)
}

fn add_sufficients<T: Config>(n: u32) {
	let asset_id = default_asset_id::<T>();
	for i in 0..n {
		let target = account("sufficient", i, SEED);
		assert!(Assets::<T>::do_mint(asset_id.into(), &target, 100u32.into()).is_ok());
	}
}

fn add_approvals<T: Config>(minter: T::AccountId, n: u32) {
	let asset_id = default_asset_id::<T>();
	T::Currency::deposit_creating(&minter, T::ApprovalDeposit::get() * n.into());
	Assets::<T>::do_mint(asset_id.into(), &minter, (100 * (n + 1)).into()).unwrap();
	let origin = SystemOrigin::Signed(minter);
	for i in 0..n {
		let target = account("approval", i, SEED);
		T::Currency::make_free_balance_be(&target, T::Currency::minimum_balance());
		let target_lookup = T::Lookup::unlookup(target);
		Assets::<T>::approve_transfer(
			origin.clone().into(),
			asset_id,
			target_lookup,
			100u32.into(),
		)
		.unwrap();
	}
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn assert_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {
	start_destroy {
		let (asset_id, caller) = create_default_minted_asset::<T>(100u32.into());
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::DestructionStarted { asset_id: asset_id.into() }.into());
	}

	destroy_accounts {
		let c in 0 .. T::RemoveItemsLimit::get();
		let (asset_id, caller) = create_default_asset::<T>();
		add_sufficients::<T>(c);
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::AccountsDestroyed {
			asset_id: asset_id.into(),
			accounts_destroyed: c,
			accounts_remaining: 0,
		}.into());
	}

	destroy_approvals {
		let a in 0 .. T::RemoveItemsLimit::get();
		let (asset_id, caller) = create_default_minted_asset::<T>(100u32.into());
		add_approvals::<T>(caller.clone(), a);
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::ApprovalsDestroyed {
			asset_id: asset_id.into(),
			approvals_destroyed: a,
			approvals_remaining: 0,
		}.into());
	}

	finish_destroy {
		let (asset_id, caller) = create_default_asset::<T>();
		Assets::<T>::start_destroy(SystemOrigin::Signed(caller.clone()).into(), asset_id)?;
	}:_(SystemOrigin::Signed(caller), asset_id)
	verify {
		assert_last_event::<T>(Event::Destroyed {
			asset_id: asset_id.into(),
		}.into()
		);
	}

	transfer {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(amount);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target_lookup, amount)
	verify {
		assert_last_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: caller, to: target, amount }.into());
	}

	transfer_keep_alive {
		let mint_amount = T::Balance::from(200u32);
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(mint_amount);
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup = T::Lookup::unlookup(target.clone());
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target_lookup, amount)
	verify {
		assert!(frame_system::Pallet::<T>::account_exists(&caller));
		assert_last_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: caller, to: target, amount }.into());
	}

	approve_transfer {
		let (asset_id, caller) = create_default_minted_asset::<T>(100u32.into());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());

		let delegated_to: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegated_to.clone());
		let amount = 100u32.into();
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, delegate_lookup, amount)
	verify {
		assert_last_event::<T>(Event::ApprovedTransfer { asset_id: asset_id.into(), source: caller, delegate: delegated_to, amount }.into());
	}

	transfer_approved {
		let (asset_id, owner) = create_default_minted_asset::<T>(100u32.into());
		let owner_lookup = T::Lookup::unlookup(owner.clone());
		T::Currency::make_free_balance_be(&owner, DepositBalanceOf::<T>::max_value());

		let delegated_to: T::AccountId = account("delegate", 0, SEED);
		whitelist_account!(delegated_to);
		let delegate_lookup = T::Lookup::unlookup(delegated_to.clone());
		let amount = 100u32.into();
		let origin = SystemOrigin::Signed(owner.clone()).into();
		Assets::<T>::approve_transfer(origin, asset_id, delegate_lookup, amount)?;

		let dest: T::AccountId = account("dest", 0, SEED);
		let dest_lookup = T::Lookup::unlookup(dest.clone());
	}: _(SystemOrigin::Signed(delegated_to.clone()), asset_id, owner_lookup, dest_lookup, amount)
	verify {
		assert!(T::Currency::reserved_balance(&owner).is_zero());
		assert_event::<T>(Event::Transferred { asset_id: asset_id.into(), from: owner, to: dest, amount }.into());
	}

	cancel_approval {
		let (asset_id, caller) = create_default_minted_asset::<T>(100u32.into());
		T::Currency::make_free_balance_be(&caller, DepositBalanceOf::<T>::max_value());

		let delegated_to: T::AccountId = account("delegate", 0, SEED);
		let delegate_lookup = T::Lookup::unlookup(delegated_to.clone());
		let amount = 100u32.into();
		let origin = SystemOrigin::Signed(caller.clone()).into();
		Assets::<T>::approve_transfer(origin, asset_id, delegate_lookup.clone(), amount)?;
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, delegate_lookup)
	verify {
		assert_last_event::<T>(Event::ApprovalCancelled { asset_id: asset_id.into(), owner: caller, delegate: delegated_to }.into());
	}

	delegate {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(amount);
		let target: T::AccountId = account("target", 0, SEED);
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target.clone())
	verify {
		assert_last_event::<T>(Event::Delegated { from: caller, to: target }.into());
	}

	revoke_delegation {
		let amount = T::Balance::from(100u32);
		let (asset_id, caller) = create_default_minted_asset::<T>(amount);
		let target: T::AccountId = account("target", 0, SEED);
		Assets::<T>::do_delegate(&asset_id.into(), &caller.clone(), &target.clone(), false)?;
	}: _(SystemOrigin::Signed(caller.clone()), asset_id, target.clone())
	verify {
		assert_last_event::<T>(Event::DelegationRevoked { delegated_by: caller, revoked_from: target }.into());
	}

	impl_benchmark_test_suite!(Assets, crate::mock::new_test_ext(), crate::mock::Test)
}
