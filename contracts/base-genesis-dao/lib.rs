#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod base_genesis_dao {

	use ink_prelude::vec::Vec;

	/// Defines the storage of your contract.
	/// Add new fields to the below struct in order
	/// to add new static storage fields to your contract.
	#[ink(storage)]
	pub struct BaseGenesisDao {
		dao_id: Vec<u8>,
	}

	impl BaseGenesisDao {
		/// Constructor that initializes the `bool` value to the given `init_value`.
		#[ink(constructor)]
		pub fn new(dao_id: Vec<u8>) -> Self {
			Self { dao_id }
		}
	}

	impl genesis_dao_traits::GenesisDAO for BaseGenesisDao {
		#[ink(message)]
		fn calculate_voting_power(&self, _voter: AccountId, voting_power: u128) -> u128 {
			2 * voting_power
		}
	}

	#[cfg(test)]
	mod tests {
		/// Imports all the definitions from the outer scope so we can use them here.
		use super::*;

		use genesis_dao_traits::GenesisDAO;

		/// We test a simple use case of our contract.
		#[ink::test]
		fn it_works() {
			let base_dao_contracts = BaseGenesisDao::new(b"TEST".to_vec());
			assert_eq!(base_dao_contracts.calculate_voting_power([0; 32].into(), 3), 6);
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

		use genesis_dao_traits::GenesisDAO;

		/// A helper function used for calling contract messages.
		use ink_e2e::build_message;

		/// The End-to-End test `Result` type.
		type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

		/// We test that we can upload and instantiate the contract
		#[ink_e2e::test]
		async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
			// Given
			let constructor = BaseGenesisDaoRef::new(b"TEST".to_vec());

			// When
			let contract_account_id = client
				.instantiate("base_dao_contracts", &ink_e2e::alice(), constructor, 0, None)
				.await
				.expect("instantiate succeeded")
				.account_id;

			// Then
			let calculate_voting_power =
				build_message::<BaseGenesisDaoRef>(contract_account_id.clone())
					.call(|contract| contract.calculate_voting_power([0; 32].into(), 3));
			let get_result =
				client.call_dry_run(&ink_e2e::alice(), &calculate_voting_power, 0, None).await;
			assert!(matches!(get_result.return_value(), 6));

			Ok(())
		}
	}
}
