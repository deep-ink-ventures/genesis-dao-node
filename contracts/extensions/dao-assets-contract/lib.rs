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
}