#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod vote_escrow {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::storage::Mapping;

    type LockedBalance = (u128, BlockNumber, u32); // (amount, created_time, unlock_time)

    /// Error types
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Could not transfer_from the PSP22 tokens, most likely
        /// because the user has not approved the contract
        UnableToLockTokens,
        /// The unlock time is too far in the future, it may not exceed the maximum lock time
        UnlockTimeToFarInTheFuture,
        /// The user has no locked balance
        NoLockedBalance,
        /// The lock up period is still running
        TokensStillLocked,
        /// Could not transfer the PSP22 tokens
        WithdrawFailed
    }

    /// Contract storage
    #[ink(storage)]
    pub struct VoteEscrow {
        /// The PSP22 token used for voting
        token: AccountId,
        /// The maximum lock time for tokens
        max_time: u32,
        /// Mapping of user accounts to their locked balances
        locked_balances: Mapping<AccountId, LockedBalance>,
    }

    impl VoteEscrow {
        /// Constructor to initialize the contract storage
        #[ink(constructor)]
        pub fn new(token: AccountId, max_time: u32) -> Self {
            Self {
                token,
                max_time,
                locked_balances: Mapping::new(),
            }
        }

        /// Returns the AccountId of the token contract.
        #[ink(message)]
        pub fn get_token(&self) -> AccountId {
            self.token
        }

        /// Returns the maximum lock time for tokens
        #[ink(message)]
        pub fn get_max_time(&self) -> u32 {
            self.max_time
        }

        /// Returns the locked balance for an account
        #[ink(message)]
        pub fn get_lock(&self, account: AccountId) -> (u128, u32, u32) {
            match self.locked_balances.get(account) {
                None => (0, 0, 0),
                Some((amount, start_time, lock_time)) => (amount, start_time.into(), lock_time)
            }
        }

        /// Create a new token lock
        #[ink(message)]
        pub fn create_lock(&mut self, amount: u128, unlock_time: u32) -> Result<(), Error> {
            if unlock_time > self.max_time {
                return Err(Error::UnlockTimeToFarInTheFuture);
            }
            let result = build_call::<DefaultEnvironment>()
                .call(self.token)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer_from")))
                        .push_arg(&self.env().caller())
                        .push_arg(&self.env().account_id())
                        .push_arg(&amount)
                        .push_arg(&"")
                )
                .returns::<Result<(), Error>>()
                .invoke();

            if result.is_err() {
                return Err(Error::UnableToLockTokens);
            }
            self.locked_balances.insert(self.env().caller(), &(
                amount,
                self.env().block_number(),
                unlock_time
            ));
            Ok(())
        }

        /// Increase the amount of locked tokens
        #[ink(message)]
        pub fn increase_amount(&mut self, additional_amount: u128) -> Result<(), Error> {
            match self.locked_balances.get(self.env().caller()) {
                None => return Err(Error::NoLockedBalance),
                Some((amount, start_time, lock_time)) => {
                    self.locked_balances.insert(
                        self.env().caller(),
                        &(amount + additional_amount, start_time, lock_time)
                    );
                }
            }
            Ok(())
        }

        /// Increase the unlock time of the locked tokens
        #[ink(message)]
        pub fn increase_unlock_time(&mut self, new_unlock_time: u32) -> Result<(), Error> {
            match self.locked_balances.get(self.env().caller()) {
                None => return Err(Error::NoLockedBalance),
                Some((amount, start_time, lock_time)) => {
                    let new_lock_time = lock_time + new_unlock_time;
                    if new_lock_time > self.max_time {
                        return Err(Error::UnlockTimeToFarInTheFuture);
                    }
                    self.locked_balances.insert(
                        self.env().caller(),
                        &(amount, start_time, new_lock_time)
                    );
                }
            }
            Ok(())
        }

        /// Withdraw unlocked tokens
        #[ink(message)]
        pub fn withdraw(&mut self) -> Result<(), Error> {
            let maybe_locked = self.locked_balances.get(self.env().caller());
            if maybe_locked.is_none() {
                return Err(Error::NoLockedBalance);
            }
            let (amount, start_time, lock_time) = maybe_locked.unwrap();
            if lock_time > self.env().block_number() - start_time {
                return Err(Error::TokensStillLocked);
            }

            match build_call::<DefaultEnvironment>()
                .call(self.token)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer")))
                        .push_arg(&self.env().caller())
                        .push_arg(&amount)
                        .push_arg(&"")
                )
                .returns::<Result<(), Error>>()
                .try_invoke()
            {
                Err(_) => Err(Error::WithdrawFailed),
                Ok(_) => {
                    // this is fine as reentrancy is disabled by default
                    self.locked_balances.remove(&self.env().caller());
                    Ok(())
                }
            }
        }
        #[ink(message)]
        pub fn voting_power(&self, account: AccountId) -> u128 {
            // Retrieve the locked balance information for the account
            let maybe_locked = self.locked_balances.get(account);
            if maybe_locked.is_none() {
                return 0;
            }
            let (amount, start_time, lock_time) = maybe_locked.unwrap();

            // Calculate the time elapsed since the tokens were locked
            let elapsed_time = self.env().block_number().saturating_sub(start_time);

            // If the elapsed time is greater than or equal to the lock time, the voting power is
            // just whatever remains in the wallet
            if elapsed_time >= lock_time {
                return amount;
            }

            // Calculate the remaining lock time
            let remaining_lock_time: u128 = lock_time.saturating_sub(elapsed_time).into();

            // Apply a decay function to the voting power if needed
            // For this example, we'll assume the voting power is proportional to the remaining lock time
            // decay_factor = remaining_lock_time / lock_time
            // voting_power = amount * decay_factor
            // To avoid floating-point arithmetic, we'll multiply before dividing
            let voting_power = amount.saturating_mul(remaining_lock_time) / lock_time as u128;

             if voting_power < amount {
                return amount;
            }
            voting_power
        }
    }
}
