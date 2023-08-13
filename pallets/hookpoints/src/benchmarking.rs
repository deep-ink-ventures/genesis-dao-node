// Copyright (C) Deep Ink Ventures GmbH
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


//! Build Instructions:
//! > cargo build --release --features runtime-benchmarks
//!
//! Weight Creation:
//! > ./target/release/genesis-dao-solochain benchmark pallet --chain dev --pallet pallet_hookpoints --extrinsic '*' --steps 20 --repeat 10 --output pallets/dao-core/src/weights.rs --template ./benchmarking/frame-weight-template.hbs

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

// Helper function to set up a whitelisted caller for interaction
fn setup_caller<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	caller
}

// Helper function to validate the benchmark flow by the last event
fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
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

	impl_benchmark_test_suite!(HookPoints, crate::mock::new_test_ext(), crate::mock::Test);
}