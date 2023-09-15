#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Genesis DAO Contract
///
/// This contract manages vote plugins that can affect the voting power
/// of DAO members in various ways. It acts as a registry and a hook
/// point for voting in the DAO.
#[ink::contract]
mod genesis_dao {
	use ink::{
		env::{
			call::{build_call, ExecutionInput, Selector},
			DefaultEnvironment,
		},
		prelude::vec::Vec,
	};

	/// Error types that can be emitted by this contract.
	#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum Error {}

	/// Event emitted when a new vote plugin is registered.
	#[ink(event)]
	pub struct VotePluginRegistered {
		#[ink(topic)]
		plugin: AccountId,
	}

	/// Event emitted when a vote plugin is removed.
	#[ink(event)]
	pub struct VotePluginRemoved {
		#[ink(topic)]
		plugin: AccountId,
	}

	/// Contract storage.
	#[ink(storage)]
	pub struct GenesisDao {
		/// Registered vote plugins.
		vote_plugins: Vec<AccountId>,
	}

	impl GenesisDao {
		/// Constructor initializes the contract.
		#[ink(constructor)]
		pub fn new() -> Self {
			Self { vote_plugins: Vec::new() }
		}

		/// Registers a new vote plugin.
		///
		/// Adds the given vote plugin to the list of registered vote plugins
		/// if it is not already present.
		///
		/// # Arguments
		///
		/// - `vote_plugin`: AccountId of the vote plugin contract.
		#[ink(message)]
		pub fn register_vote_plugin(&mut self, vote_plugin: AccountId) {
			if self.vote_plugins.contains(&vote_plugin) {
				return
			}
			self.vote_plugins.push(vote_plugin);
			self.env().emit_event(VotePluginRegistered { plugin: vote_plugin });
		}

		/// Removes a vote plugin.
		///
		/// Removes the given vote plugin from the list of registered vote plugins.
		///
		/// # Arguments
		///
		/// - `vote_plugin`: AccountId of the vote plugin contract.
		#[ink(message)]
		pub fn remove_vote_plugin(&mut self, vote_plugin: AccountId) {
			self.vote_plugins.retain(|&x| x != vote_plugin);
			self.env().emit_event(VotePluginRemoved { plugin: vote_plugin });
		}

		/// Gets all registered vote plugins.
		///
		/// Returns a list of AccountIds for all registered vote plugins.
		///
		/// # Returns
		///
		/// - `Vec<AccountId>`: List of registered vote plugins.
		#[ink(message)]
		pub fn get_vote_plugins(&self) -> Vec<AccountId> {
			self.vote_plugins.clone()
		}
	}

	impl genesis_dao_contract_trait::GenesisDao for GenesisDao {
		/// `on_vote` Hook Point
		///
		/// This function gets called when a vote is made. It iterates through
		/// all registered vote plugins and updates the voting power of the voter.
		///
		/// # Arguments
		///
		/// - `voter`: AccountId of the voter.
		/// - `voting_power`: Initial voting power of the voter.
		///
		/// # Returns
		///
		/// - `Balance`: Updated voting power after considering all vote plugins.
		#[ink(message)]
		fn on_vote(&self, voter: AccountId, voting_power: Balance) -> Balance {
			let mut voting_power = voting_power;

			for contract_id in self.vote_plugins.iter() {
				voting_power = match build_call::<DefaultEnvironment>()
					.call(*contract_id)
					.exec_input(
						ExecutionInput::new(Selector::new(ink::selector_bytes!(
							"Vote::get_voting_power"
						)))
						.push_arg(&voter)
						.push_arg(&voting_power),
					)
					.returns::<Balance>()
					.try_invoke()
				{
					Ok(new_voting_power) => new_voting_power.unwrap_or(voting_power),
					Err(_) => voting_power,
				};
			}
			voting_power
		}
	}
}
