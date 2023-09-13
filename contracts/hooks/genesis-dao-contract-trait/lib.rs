#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink_primitives::AccountId;

type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

#[ink::trait_definition]
pub trait GenesisDao {
	/// hook point for `on_vote` pallet
	#[ink(message)]
	fn on_vote(&self, voter: AccountId, voting_power: Balance) -> Balance;
}
