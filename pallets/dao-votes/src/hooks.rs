use crate::Config;
use pallet_hookpoints::Pallet as HP;

pub fn on_vote<T: Config>(owner: T::AccountId, signer: T::AccountId, voter: T::AccountId, voting_power: T::Balance) -> T::Balance
{
   let hp = HP::<T>::create(
		"GenesisDao::on_vote",
		owner,
		signer
	)
		.add_arg::<T::AccountId>(voter)
		.add_arg::<T::Balance>(voting_power);

	HP::<T>::execute::<T::Balance>(hp).unwrap_or(voting_power)
}