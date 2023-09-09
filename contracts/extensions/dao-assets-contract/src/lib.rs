#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// `dao_assets_contract` module provides a contract for assets, implementing the PSP22 standard.
/// The contract primarily focuses on asset transfers, approvals, and allowances.;
use dao_assets_extension::AssetExtension;
use ink::env::Environment;
pub mod psp22;

/// A custom environment for the AssetContract, using the default Ink environment but
/// with a specific chain extension for asset operations.
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

/// The primary contract module for DAO assets.
#[ink::contract(env = crate::CustomEnvironment)]
mod dao_assets_contract {
    use ink::prelude::vec::Vec;
    use dao_assets_extension::AssetId;
    use crate::psp22::{traits::PSP22, errors::PSP22Error};

    /// The `AssetContract` struct represents the main storage for the contract,
    /// containing the `asset_id` which is unique for each asset.
    #[ink(storage)]
    pub struct AssetContract {
        asset_id: AssetId,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    impl AssetContract {
        /// Constructs a new `AssetContract` with a given `asset_id`.
        #[ink(constructor)]
        pub fn new(asset_id: AssetId) -> Self {
            Self { asset_id }
        }

        /// Returns the asset id
        #[ink(message)]
        pub fn get_asset_id(&self) -> AssetId {
            self.asset_id
        }

        /// Transfers an `amount` of assets to the `target` account, ensuring the sender stays alive.
        #[ink(message)]
        pub fn transfer_keep_alive(&mut self, to: AccountId, value: Balance) -> Result<(), PSP22Error> {
            let result = self.env().extension().transfer_keep_alive(self.asset_id, to.clone(), value).map_err(PSP22Error::from);
            if result.is_ok() {
                self.env().emit_event(Transfer {
                    from: Some(self.env().caller()),
                    to: Some(to),
                    value,
                });
            }
            result
        }
    }

    /// Implementation of the PSP22 standard for the `AssetContract`.
    impl PSP22 for AssetContract {
        /// Returns the total supply of the asset.
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.env().extension().total_supply(self.asset_id).unwrap_or(0)
        }

        /// Returns the balance of the `owner` account.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.env().extension().balance_of(self.asset_id, owner).unwrap_or(0)
        }

        /// Returns the allowance provided to `spender` by the `owner`.
        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.env().extension().allowance(self.asset_id, owner, spender).unwrap_or(0)
        }

        /// Transfer tokens from the caller to the `to` account.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let result = self.env().extension().transfer(self.asset_id, to.clone(), value).map_err(PSP22Error::from);
            if result.is_ok() {
                self.env().emit_event(Transfer {
                    from: Some(self.env().caller()),
                    to: Some(to),
                    value,
                });
            }
            result
        }

        /// Transfer tokens from the `from` account to the `to` account on behalf of the caller.
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance, _data: Vec<u8>) -> Result<(), PSP22Error> {
            let result = self.env().extension().transfer_from(self.asset_id, from.clone(), to.clone(), value).map_err(PSP22Error::from);
            if result.is_ok() {
                self.env().emit_event(Transfer {
                    from: Some(from),
                    to: Some(to),
                    value,
                });
            }
            result
        }

        /// Approve the `spender` to transfer tokens on behalf of the caller.
        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            let result = self.env().extension().approve(self.asset_id, spender.clone(), value).map_err(PSP22Error::from);
            if result.is_ok() {
                self.env().emit_event(Approval {
                    owner: self.env().caller(),
                    spender,
                    value,
                });
            }
            result
        }

        /// Increase the allowance provided to `spender` by `delta_value`.
        #[ink(message)]
        fn increase_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error> {
            let sender = self.env().caller();
            let current_allowance = self.allowance(sender.clone(), spender.clone());
            let new_allowance = current_allowance + delta_value;
            self.env().extension().cancel_approval(self.asset_id, spender.clone()).map_err(PSP22Error::from)?;
            self.approve(spender, new_allowance).map_err(PSP22Error::from)
        }

        /// Decrease the allowance provided to `spender` by `delta_value`.
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