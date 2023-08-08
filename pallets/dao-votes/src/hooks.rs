use crate::Config;
use pallet_hookpoints::Pallet as HP;

pub fn on_vote<T: Config>(owner: T::AccountId, signer: T::AccountId, voter: T::AccountId, balance: T::Balance) -> T::Balance
{ 
   let hp = HP::<T>::create(
		"GenesisDAO::on_vote",
		owner,
		signer
	) 
		.add_arg::<T::AccountId>(voter)
		.add_arg::<T::Balance>(balance);

	HP::<T>::execute::<T::Balance>(hp).unwrap_or(balance)
}