#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod genesis_dao {
    #[ink(storage)]
    pub struct GenesisDao {}

    impl GenesisDao {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
    }

    impl genesis_dao_contract_trait::GenesisDao for GenesisDao {
        /// hook point for `on_vote` pallet
        #[ink(message)]
        fn on_vote(&self, _voter: AccountId, voting_power: Balance) -> Balance {
            voting_power * 2
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use genesis_dao_contract_trait::GenesisDao as Trait;
        
        #[ink::test]
        fn test_on_vote_hookpoint() {
            let genesis_dao = GenesisDao::new();
            assert_eq!(genesis_dao.on_vote(AccountId::from([0x01; 32]), 50), 100);
        }
    }

}