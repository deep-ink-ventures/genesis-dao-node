#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod genesis_dao {

    #[ink(storage)]
    pub struct GenesisDao {}

    impl genesis_dao_contract_traits::GenesisDao for GenesisDao {
        /// hook point for `t` pallet
        #[ink(message)]
        fn t(whatever: AccountId) {
            // do nothing
        }

        /// hook point for `on_vote` pallet
        #[ink(message)]
        fn on_vote(voter: AccountId, foo: u128) -> u128 {
            foo
        }

        /// hook point for `empty_func` pallet
        #[ink(message)]
        fn empty_func() {
            // do nothing
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_t_hookpoint() {
            let genesis_dao = GenesisDao::default();
            assert_eq!(genesis_dao.t(), ());
        }

        #[ink::test]
        fn test_on_vote_hookpoint() {
            let genesis_dao = GenesisDao::default();
            assert_eq!(genesis_dao.on_vote(&ink_e2e::alice(), 42), 42);
        }

        #[ink::test]
        fn test_empty_func_hookpoint() {
            let genesis_dao = GenesisDao::default();
            assert_eq!(genesis_dao.empty_func(), ());
        }

    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
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
