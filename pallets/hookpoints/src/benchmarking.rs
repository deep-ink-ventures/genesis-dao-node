#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

use crate::Pallet as HookPoints;

// Helper function to set up a whitelisted caller for interaction
fn setup_caller<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	caller
}

benchmarks! {

	  register_global_callback {
		let caller = setup_caller::<T>();
		let contract: T::AccountId = account("contract", 0, 0);
	}: _(RawOrigin::Signed(caller.clone()), contract)
	verify {
		assert_last_event::<T>(Event::GlobalCallbackRegistered { who: caller }.into());
	}

	register_specific_callback {
		let caller = setup_caller::<T>();
		let contract: T::AccountId = account("contract_specific", 0, 0);
		let id = b"HOOK".to_vec();
	}: _(RawOrigin::Signed(caller.clone()), contract, id)
	verify {
		assert_last_event::<T>(Event::SpecificCallbackRegistered { who: caller }.into());
	}

}

// Helper function to validate the benchmark flow by the last event
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

impl_benchmark_test_suite!(HookPoints, crate::mock::new_test_ext(), crate::mock::Test)
