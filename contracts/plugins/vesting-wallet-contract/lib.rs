#![cfg_attr(not(feature = "std"), no_std, no_main)]

type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;

/// Storage for the main contract.
#[ink::contract]
pub mod vesting_wallet {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::LangError;
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned when we are unable to fund the vesting wallet from the sender
        CannotFund,
    }

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct VestingWalletInfo {
        start_time: u64,
        duration: u64,
        total_tokens: u128,
        withdrawn_tokens: u128,
    }

    #[ink(storage)]
    pub struct VestingWallets {
        token: AccountId,
        wallets: Mapping<AccountId, VestingWalletInfo>,
    }

    impl VestingWallets {
        /// Constructor initializes the token for the contract.
        #[ink(constructor)]
        pub fn new(token: AccountId) -> Self {
            Self {
                token,
                wallets: Mapping::new(),
            }
        }

        /// Get the token for the contract.
        #[ink(message)]
        pub fn get_token(&self) -> AccountId {
            self.token
        }

        /// Create a vesting wallet for a user.
        #[ink(message)]
        pub fn create_vesting_wallet_for(&mut self, account: AccountId, amount: u128, duration: u64) -> Result<(), Error> {
            match build_call::<DefaultEnvironment>()
                .call(self.token)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer_from")))
                        .push_arg(&self.env().caller())
                        .push_arg(&self.env().account_id())
                        .push_arg(&amount)
                        .push_arg(&"")
                )
                .returns::<Result<(), Error>>()
                .try_invoke()
            {
                Err(_) => Err(Error::CannotFund),
                Ok(_) => {
                    let start_time = Self::env().block_timestamp();
                    let wallet_info = VestingWalletInfo {
                        start_time,
                        duration,
                        total_tokens: amount,
                        withdrawn_tokens: 0,
                    };
                    self.wallets.insert(account, &wallet_info);
                    Ok(())
                }
            }
        }

        /// Get unvested tokens for a user.
        #[ink(message)]
        pub fn get_unvested(&self, account: AccountId) -> u128 {
            // Default value for compilation
            0
        }

        /// Get vested but not yet withdrawn tokens for a user.
        #[ink(message)]
        pub fn get_available_for_withdraw(&self, account: AccountId) -> u128 {
            // Default value for compilation
            0
        }

        /// Get total tokens for a user.
        #[ink(message)]
        pub fn get_total(&self, account: AccountId) -> u128 {
            // Default value for compilation
            0
        }

        /// Withdraw vested tokens.
        #[ink(message)]
        pub fn withdraw(&mut self, account: AccountId) {
            // Implementation here
        }
    }
}
