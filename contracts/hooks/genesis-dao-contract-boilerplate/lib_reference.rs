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
        /// hook point for `empty_func` pallet
        #[ink(message)]
        fn empty_func(&self) -> u32 {
            0
        }

        /// hook point for `t` pallet
        #[ink(message)]
        fn t(&self, _whatever: AccountId) {
            // do nothing
        }

        /// hook point for `on_vote` pallet
        #[ink(message)]
        fn on_vote(&self, _voter: AccountId, foo: u128) -> u128 {
            foo
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use genesis_dao_contract_trait::GenesisDao as Trait;
        use ink_primitives::AccountId;

        #[ink::test]
        fn test_t_hookpoint() {
            let genesis_dao = GenesisDao::new();
            assert_eq!(genesis_dao.t(AccountId::from([0x01; 32])), ());
        }

        #[ink::test]
        fn test_on_vote_hookpoint() {
            let genesis_dao = GenesisDao::new();
            assert_eq!(genesis_dao.on_vote(AccountId::from([0x01; 32]), 42), 42);
        }

        #[ink::test]
        fn test_empty_func_hookpoint() {
            let genesis_dao = GenesisDao::new();
            assert_eq!(genesis_dao.empty_func(), ());
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
         use super::*;
         use ink_e2e::build_message;

         type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;


        #[ink_e2e::test]
        async fn test_t_hookpoint(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = GenesisDao::default();

            // When
            let contract_account_id = client
                .instantiate("genesis_dao", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let t = build_message::<GenesisDao>(contract_account_id.clone())
                .call(|genesis_dao| genesis_dao.t());
            let t_result = client.call_dry_run(&ink_e2e::alice(), &t, 0, None).await;
            assert!(matches!(t_result.return_value(), ()));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_on_vote_hookpoint(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = GenesisDao::default();

            // When
            let contract_account_id = client
                .instantiate("genesis_dao", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let on_vote = build_message::<GenesisDao>(contract_account_id.clone())
                .call(|genesis_dao| genesis_dao.on_vote(&ink_e2e::alice(), 42));
            let on_vote_result = client.call_dry_run(&ink_e2e::alice(), &on_vote, 0, None).await;
            assert!(matches!(on_vote_result.return_value(), 42));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_empty_func_hookpoint(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = GenesisDao::default();

            // When
            let contract_account_id = client
                .instantiate("genesis_dao", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let empty_func = build_message::<GenesisDao>(contract_account_id.clone())
                .call(|genesis_dao| genesis_dao.empty_func());
            let empty_func_result = client.call_dry_run(&ink_e2e::alice(), &empty_func, 0, None).await;
            assert!(matches!(empty_func_result.return_value(), ()));

            Ok(())
        }
     }
}
