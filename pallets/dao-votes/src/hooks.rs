use codec::{Encode};
use crate::Config;
use pallet_hookpoints::Pallet as HookPoints;

pub fn on_vote_callback<T: Config>(dao_owner: T::AccountId, voter: T::AccountId, balance: T::Balance) -> T::Balance {
	// the selector for "GenesisDAO::calculate_voting_power"
	let mut data = 0xa68e4cba_u32.to_be_bytes().to_vec();
	data.append(&mut voter.encode());
	data.append(&mut balance.encode());

	HookPoints::<T>::exec_hook_point::<T::Balance>(
		&dao_owner,
		voter,
		"ON_VOTING_CALC",
		data
	).unwrap_or(balance)
}
