use crate::Config;
use pallet_hookpoints::Pallet as HP;


pub fn on_vote_callback<T: Config>(dao_owner: T::AccountId, voter: T::AccountId, balance: T::Balance) -> T::Balance {
	HP::<T>::execute::<T::Balance>(
		HP::<T>::create(
			dao_owner,
			voter.clone(),
			"GenesisDAO",
			HP::<T>::callback("calculate_voting_power")
				.add_arg::<T::AccountId>(voter)
				.add_arg::<T::Balance>(balance)
		)
	).unwrap_or(balance)
}
