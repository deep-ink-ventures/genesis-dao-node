#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod genesis_dao {
    use ink::prelude::vec::Vec;
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {}

    #[ink(storage)]
    pub struct GenesisDao {
        vote_plugins: Vec<AccountId>,
    }

    impl GenesisDao {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                vote_plugins: Vec::new(),
            }
        }

        #[ink(message)]
        pub fn register_vote_plugin(&mut self, vote_plugin: AccountId) {
            if self.vote_plugins.contains(&vote_plugin) {
                return;
            }
            self.vote_plugins.push(vote_plugin);
        }

        #[ink(message)]
        pub fn remove_vote_plugin(&mut self, vote_plugin: AccountId) {
            self.vote_plugins.retain(|&x| x != vote_plugin);
        }

        #[ink(message)]
        pub fn get_vote_plugins(&self) -> Vec<AccountId> {
            self.vote_plugins.clone()
        }
    }

    impl genesis_dao_contract_trait::GenesisDao for GenesisDao {

        /// Hook point for `on_vote` pallet
        ///
        /// Iterates over all registered vote plugins and calls their `get_voting_power` function.
        ///
        /// # Arguments
        ///
        /// - `voter`: The AccountId of the voter.
        /// - `voting_power`: The voting power of the voter.
        ///
        /// - Returns the new voting power.
        #[ink(message)]
        fn on_vote(&self, voter: AccountId, voting_power: Balance) -> Balance {
            let mut voting_power = voting_power.clone();

            for contract_id in self.vote_plugins.iter() {
                voting_power = match build_call::<DefaultEnvironment>()
                    .call(contract_id.clone())
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!("Vote::get_voting_power")))
                            .push_arg(&voter)
                            .push_arg(&voting_power)
                    )
                    .returns::<Balance>()
                    .try_invoke() {
                    Ok(new_voting_power) => new_voting_power.unwrap_or(voting_power),
                    Err(_) => voting_power
                };
            }
            voting_power
        }
    }
}