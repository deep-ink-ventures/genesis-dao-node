use crate::Config;
use pallet_hookpoints::Pallet as HP;


pub fn on_vote_callback<T: Config>(dao_owner: T::AccountId, voter: T::AccountId, balance: T::Balance) -> T::Balance {
	let hp = HP::<T>::create(
		"GenesisDAO::calculate_voting_power",
		dao_owner,
		voter.clone()
	)
		.add_arg::<T::AccountId>(voter)
		.add_arg::<T::Balance>(balance);

	HP::<T>::execute::<T::Balance>(hp).unwrap_or(balance)
}
