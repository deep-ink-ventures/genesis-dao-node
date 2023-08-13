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
            voting_power
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use genesis_dao_contract_trait::GenesisDao as Trait;
        
        #[ink::test]
        fn test_on_vote_hookpoint() {
            let genesis_dao = GenesisDao::new();
            assert_eq!(genesis_dao.on_vote(AccountId::from([0x01; 32]), 0), 0);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use genesis_dao_contract_trait::GenesisDao as Trait;
        use ink_e2e::build_message;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_test_on_vote(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = GenesisDaoRef::new();

            let contract_account_id = client
                .instantiate("genesis_dao", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let on_vote = build_message::<GenesisDaoRef>(contract_account_id.clone())
                .call(|genesis_dao| genesis_dao.on_vote(
                    ink::env::test::default_accounts::<ink::env::DefaultEnvironment>().alice,
                    0
                ));

            let result = client
                .call_dry_run(&ink_e2e::alice(), &on_vote, 0, None)
                .await;

            assert_eq!(result.return_value(), 0);
            Ok(())
        }
    }
}