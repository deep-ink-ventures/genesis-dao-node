#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod base_dao_contracts {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct BaseDaoContracts {
		dao_id: Vec<u8>,
    }

    impl dao_contract_traits::GenesisDAO for BaseDaoContracts {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(dao_id: Vec<u8>) -> Self {
            Self ( dao_id }
        }

		#[ink(message)]
		fn calculate_voting_power(&self, _voter: AccountId, voting_power: u128) -> u128 {
			2 * voting_power
		}
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut base_dao_contracts = BaseDaoContracts::new(b"TEST");
            assert_eq!(base_dao_contracts.calculate_voting_power(3), 6);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = BaseDaoContractsRef::new(b"TEST");

            // When
            let contract_account_id = client
                .instantiate("base_dao_contracts", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate succeeded")
                .account_id;

            // Then
            let get = build_message::<BaseDaoContractsRef>(contract_account_id.clone())
                .call(|base_dao_contracts| base_dao_contracts.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }
    }
}
