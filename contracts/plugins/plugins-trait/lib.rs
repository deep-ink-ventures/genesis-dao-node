#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink_primitives::AccountId;

type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

#[ink::trait_definition]
pub trait Vote {
	/// Retrieves the id of the vote plugin.
	/// The id should be unique.
	#[ink(message)]
	fn get_id(&self) -> u32;

	/// Retrieves the voting power from the underlying contract.
	/// The plugin should return the new total, not the additional total.
	///
	/// # Arguments
	///
	/// - `voter`: The AccountId of the voter.
	/// - `voting_power`: The voting power of the voter.
	#[ink(message)]
	fn get_voting_power(&self, voter: AccountId, voting_power: Balance) -> Balance;
}
