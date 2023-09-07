use crate::Config;
use pallet_dao_core::BalanceOf;
use pallet_hookpoints::Pallet as HP;

pub fn on_vote<T: Config>(
	owner: T::AccountId,
	signer: T::AccountId,
	voter: T::AccountId,
	voting_power: BalanceOf<T>,
) -> BalanceOf<T> {
	let hp = HP::<T>::create("GenesisDao::on_vote", owner, signer)
		.add_arg::<T::AccountId>(voter)
		.add_arg::<BalanceOf<T>>(voting_power);

	HP::<T>::execute::<BalanceOf<T>>(hp).unwrap_or(voting_power)
}
