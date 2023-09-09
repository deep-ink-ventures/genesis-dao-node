#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// ### Vesting Wallet
///
/// Follows this function:
///
/// $$U(t) = V_{t} - \frac{t - t_0}{T}V_{t}$$
///
/// Where:
///
/// - $U(t)$ is the number of unvested tokens at time t,
/// - $t$ is the time elapsed since the beginning of the vesting period,
/// - $t_0$ is the starting time of the vesting period,
/// - $T$ is the total duration of the vesting period,
/// - $V_{t}$ is the total number of tokens to be vested.
///
#[ink::contract]
pub mod vesting_wallet {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::storage::Mapping;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Raised when we are unable to fund the vesting wallet from the sender
        CannotFund,
        /// Raised when we are unable to withdraw from the vesting wallet
        WithdrawFailed
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
                    let start_time = self.env().block_timestamp();
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
            match self.wallets.get(&account) {
                None => 0,
                Some(wallet) => {
                    let now: u128 = self::env().block_timestamp().into();
                    let start: u128 = wallet.start_time.into();
                    let duration: u128 = wallet.duration.into();

                    if duration > 0 {
                        wallet.total_tokens - (now - start) * wallet.total_tokens / duration
                    } else {
                        wallet.total_tokens
                    }
                }
            }
        }

        /// Get vested but not yet withdrawn tokens for a user.
        #[ink(message)]
        pub fn get_available_for_withdraw(&self, account: AccountId) -> u128 {
            match self.wallets.get(&account) {
                None => 0,
                Some(wallet) => {
                    wallet.total_tokens - wallet.withdrawn_tokens - self.get_unvested(account)
                }
            }
        }

        /// Get total tokens for a user held by this wallet
        #[ink(message)]
        pub fn get_total(&self, account: AccountId) -> u128 {
            self.get_unvested(account.clone()) + self.get_available_for_withdraw(account)
        }

        /// Withdraw vested tokens.
        #[ink(message)]
        pub fn withdraw(&mut self, account: AccountId) -> Result<u128, Error>{
            let mut wallet = match self.wallets.get(&account) {
                None => return Ok(0),
                Some(wallet) => wallet
            };
            let amount = self.get_available_for_withdraw(account);
            if amount == 0 {
                return Ok(0);
            }

            match build_call::<DefaultEnvironment>()
                .call(self.token)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer")))
                        .push_arg(&account)
                        .push_arg(&amount)
                        .push_arg(&"")
                )
                .returns::<Result<(), Error>>()
                .try_invoke()
            {
                Err(_) => Err(Error::WithdrawFailed),
                Ok(_) => {
                    // this is fine as reentrancy is disbaled by default
                    wallet.withdrawn_tokens += &amount;
                    self.wallets.insert(account, &wallet);
                    Ok(amount)
                }
            }
        }
    }
}
