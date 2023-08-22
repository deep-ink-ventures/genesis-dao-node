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
    use ink::prelude::vec::Vec;

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

        #[ink(message)]
        pub fn transfer_keep_alive(&mut self, target: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self.env().extension().transfer_keep_alive(self.asset_id, target, amount).map_err(PSP22Error::from)
        }
	}

    impl PSP22 for AssetContract {

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.env().extension().total_supply(self.asset_id).unwrap_or(0)
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.env().extension().balance_of(self.asset_id, owner).unwrap_or(0)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.env().extension().allowance(self.asset_id, owner, spender).unwrap_or(0)
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            self.env().extension().transfer(self.asset_id, to, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            self.env().extension().transfer_from(self.asset_id, from, to, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            self.env().extension().approve(self.asset_id, spender, value).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn increase_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            let current_allowance = self.allowance(sender.clone(), spender.clone());
            let new_allowance = current_allowance + delta_value;
            self.env().extension().cancel_approval(self.asset_id, spender.clone()).map_err(PSP22Error::from)?;
            self.approve(spender, new_allowance).map_err(PSP22Error::from)
        }

        #[ink(message)]
        fn decrease_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            let current_allowance = self.allowance(sender.clone(), spender.clone());
            if current_allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance);
            }
            let new_allowance = current_allowance - delta_value;
            self.env().extension().cancel_approval(self.asset_id, spender.clone()).map_err(PSP22Error::from)?;
            self.approve(spender, new_allowance).map_err(PSP22Error::from)
        }
    }
}
