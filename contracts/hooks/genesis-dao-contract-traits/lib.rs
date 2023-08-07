#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink_primitives::AccountId;

#[ink::trait_definition]
pub trait GenesisDAO {

	/// Called upon voting
	#[ink(message)]
	fn calculate_voting_power(&self, voter: AccountId, voting_power: u128) -> u128;
}