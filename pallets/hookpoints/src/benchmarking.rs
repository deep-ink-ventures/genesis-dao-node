//! Benchmarking setup for pallet-hookpoints

use super::*;
use crate::Pallet as HookPoints;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller};
use frame_system::{Pallet as System, RawOrigin};

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
	register_global_callback {
		let who: T::AccountId = whitelisted_caller();
		let contract: T::AccountId = account("contract", 0, SEED);
	}: _(RawOrigin::Signed(who.clone()), contract.clone())
	verify {
		assert_eq!(HookPoints::<T>::callbacks(who.clone()), Some(contract));
		assert_last_event::<T>(Event::GlobalCallbackRegistered { who }.into());
	}

	impl_benchmark_test_suite!(HookPoints, crate::mock::new_test_ext(), crate::mock::Test);
}
