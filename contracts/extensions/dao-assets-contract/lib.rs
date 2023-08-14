#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;
use dao_assets_extension::AssetExtension;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = dao_assets_extension::AccountId;
    type Balance = dao_assets_extension::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = AssetExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod dao_assets_contract {
    use dao_assets_extension::{AssetId, AssetError};

    /// Defines the storage of our contract.
    ///
    /// Here we store the random seed fetched from the chain.
    #[ink(storage)]
    pub struct AssetContract {
        asset_id: AssetId
    }

    impl AssetContract {
        #[ink(constructor)]
        pub fn new(asset_id: AssetId) -> Self {
            Self { asset_id }
        }

        #[ink(message)]
		pub fn transfer(
			&self,
			target: AccountId,
			amount: Balance,
		) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.transfer(sender, asset_id, target, amount)
		}
    }

    // /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    // #[cfg(test)]
    // mod tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;
    //
    //     /// We test if the default constructor does its job.
    //     #[ink::test]
    //     fn default_works() {
    //         let rand_extension = RandExtension::new_default();
    //         assert_eq!(rand_extension.get(), [0; 32]);
    //     }
    //
    //     #[ink::test]
    //     fn chain_extension_works() {
    //         // given
    //         struct MockedExtension;
    //         impl ink::env::test::ChainExtension for MockedExtension {
    //             /// The static function id of the chain extension.
    //             fn func_id(&self) -> u32 {
    //                 1101
    //             }
    //
    //             /// The chain extension is called with the given input.
    //             ///
    //             /// Returns an error code and may fill the `output` buffer with a
    //             /// SCALE encoded result. The error code is taken from the
    //             /// `ink::env::chain_extension::FromStatusCode` implementation for
    //             /// `RandomReadErr`.
    //             fn call(&mut self, _input: &[u8], output: &mut Vec<u8>) -> u32 {
    //                 let ret: [u8; 32] = [1; 32];
    //                 scale::Encode::encode_to(&ret, output);
    //                 0
    //             }
    //         }
    //         ink::env::test::register_chain_extension(MockedExtension);
    //         let mut rand_extension = RandExtension::new_default();
    //         assert_eq!(rand_extension.get(), [0; 32]);
    //
    //         // when
    //         rand_extension.update([0_u8; 32]).expect("update must work");
    //
    //         // then
    //         assert_eq!(rand_extension.get(), [1; 32]);
    //     }
    // 1}
}