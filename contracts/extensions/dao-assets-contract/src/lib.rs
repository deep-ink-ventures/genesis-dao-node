#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod psp22;

use dao_assets_extension::AssetExtension;
use ink::env::Environment;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = dao_assets_extension::AccountId;
	type Balance = dao_assets_extension::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

	type ChainExtension = AssetExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod dao_assets_contract {
	use dao_assets_extension::AssetId;
    use crate::psp22::{PSP22, PSP22Error};

    #[ink(storage)]
	pub struct AssetContract {
		asset_id: AssetId,
	}

	impl AssetContract {
		#[ink(constructor)]
		pub fn new(asset_id: AssetId) -> Self {
			Self { asset_id }
		}
	}

    impl PSP22 for AssetContract {

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            // TODO: Implement total_supply retrieval
            0
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            // TODO: Implement balance retrieval
            0
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // TODO: Implement allowance retrieval
            0
        }

        #[ink(message)]
        fn transfer(&self, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            self.env().extension().transfer(sender, self.asset_id, to, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn transfer_from(&self, from: AccountId, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            self.env().extension().transfer_from(sender, self.asset_id, from, to, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn approve(&self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            self.env().extension().approve(sender, self.asset_id, spender, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn increase_allowance(&self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            let current_allowance = self.allowance(sender.clone(), spender.clone());
            let new_allowance = current_allowance + delta_value;
            self.approve(spender, new_allowance).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn decrease_allowance(&self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            let current_allowance = self.allowance(sender.clone(), spender.clone());
            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);  // This might not be the exact error you want to return, adjust as needed
            }
            let new_allowance = current_allowance - delta_value;
            self.approve(spender, new_allowance).map_err(PSP22Error::from)
        }
    }

    // Separate impl block for transfer_keep_alive
    impl AssetContract {
        #[ink(message)]
        pub fn transfer_keep_alive(&self, target: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            self.env().extension().transfer_keep_alive(sender, self.asset_id, target, amount).map_err(PSP22Error::from)
        }
    }
}
