#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! # Vote Escrow Contract
//!
//! ## Voting Power Function
//!
//! The core functionality of this contract revolves around the decay function for calculating
//! voting power, which is represented by the following formula:
//!
//! $$w(t) = \frac{{\text{{amount}} \cdot (\text{{end}} - t)}}{{\text{{MAXTIME}}}}$$
//!
//! Where:
//! - $w(t)$: Voting weight at time \( t \).
//! - *amount*: The amount of tokens locked.
//! - *end*: The unlock time (in seconds).
//! - *MAXTIME* : The maximum lock time (e.g., 4 years in seconds).
//!
//! ### Representation of Decay
//!
//! The decay of voting power is linear over time. The x-axis in a hypothetical plot would represent
//! the time elapsed in years, starting from the time the tokens are locked until they are unlocked
//! (e.g., 2 years). The y-axis would represent the voting weight, which starts from the full amount
//! of locked tokens and decays linearly to zero as time progresses to the unlock time.
//!
//! This decay function ensures that the voting weight decreases uniformly over time, reflecting
//! the user's decreasing commitment to the future of whatever they are voting for.
//!
//! The decay is additive, meaning that it adds to the amount of tokens that are unlocked at the
//! unlock time.
//!
//! ### Boosting
//! The voting power can be boosted by a constant value given on `new`.

#[ink::contract]
pub mod vote_escrow {
    use ink::env::call::{build_call, ExecutionInput, Selector};
    use ink::env::DefaultEnvironment;
    use ink::storage::Mapping;

    type LockedBalance = (u128, BlockNumber, u32); // (amount, created_time, unlock_time)

    /// Event emitted when tokens are successfully locked.
    #[ink(event)]
    pub struct Locked {
        #[ink(topic)]
        from: AccountId,
        amount: u128,
        unlock_time: u32,
    }

    /// Event emitted when the lock amount is increased.
    #[ink(event)]
    pub struct LockAmountIncreased {
        #[ink(topic)]
        account: AccountId,
        added_amount: u128,
        total_amount: u128,
    }

    /// Event emitted when the lock time is increased.
    #[ink(event)]
    pub struct LockTimeIncreased {
        #[ink(topic)]
        account: AccountId,
        new_unlock_time: u32,
    }

    /// Event emitted when tokens are successfully withdrawn.
    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        account: AccountId,
        amount: u128,
    }

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
        WithdrawFailed,
        /// The lock time can only be increased
        OnlyIncreaseOfLockPossible
    }

    /// Contract storage
    #[ink(storage)]
    pub struct VoteEscrow {
        /// The PSP22 token used for voting
        token: AccountId,
        /// The maximum lock time for tokens
        max_time: u32,
        /// The boost to apply to the voting power
        boost: u8,
        /// Mapping of user accounts to their locked balances
        locked_balances: Mapping<AccountId, LockedBalance>,
    }

    impl VoteEscrow {
        /// Initializes a new VoteEscrow contract.
        ///
        /// - `token`: The token contract to use for locking and voting.
        /// - `max_time`: The maximum time tokens can be locked in blocks.
        /// - `boost`: The boost factor to be applied to voting power.
        #[ink(constructor)]
        pub fn new(token: AccountId, max_time: u32, boost: u8) -> Self {
            Self {
                token,
                max_time,
                boost,
                locked_balances: Mapping::new(),
            }
        }

        /// Returns the PSP22 token contract used in this contract.
        #[ink(message)]
        pub fn get_token(&self) -> AccountId {
            self.token
        }

        /// Returns the maximum lock time for tokens in blcoks
        #[ink(message)]
        pub fn get_max_time(&self) -> u32 {
            self.max_time
        }

        /// Returns the locked balance for a given user.
        ///
        /// Returns a tuple `(amount, created_time, unlock_time)`.
        #[ink(message)]
        pub fn get_lock(&self, account: AccountId) -> (u128, u32, u32) {
            match self.locked_balances.get(account) {
                None => (0, 0, 0),
                Some((amount, start_time, lock_time)) => (amount, start_time.into(), lock_time)
            }
        }

        /// Locks a given amount of tokens until a specified unlock time.
        ///
        /// - `amount`: The amount of tokens to lock.
        /// - `unlock_time`: The time in blocks until which the tokens will be locked.
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

            self.env().emit_event(Locked {
                from: self.env().caller(),
                amount,
                unlock_time,
            });
            Ok(())
        }

        /// Increases the amount of tokens locked in the contract.
        ///
        /// - `additional_amount`: The additional amount of tokens to lock.
        #[ink(message)]
        pub fn increase_amount(&mut self, additional_amount: u128) -> Result<(), Error> {
            match self.locked_balances.get(self.env().caller()) {
                None => return Err(Error::NoLockedBalance),
                Some((amount, start_time, lock_time)) => {
                    let result = build_call::<DefaultEnvironment>()
                        .call(self.token)
                        .exec_input(
                            ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer_from")))
                                .push_arg(&self.env().caller())
                                .push_arg(&self.env().account_id())
                                .push_arg(&additional_amount)
                                .push_arg(&"")
                        )
                        .returns::<Result<(), Error>>()
                        .invoke();

                    if result.is_err() {
                        return Err(Error::UnableToLockTokens);
                    }

                    self.locked_balances.insert(
                        self.env().caller(),
                        &(amount + additional_amount, start_time, lock_time),
                    );
                    self.env().emit_event(LockAmountIncreased {
                        account: self.env().caller(),
                        added_amount: additional_amount,
                        total_amount: amount + additional_amount
                    });
                    Ok(())
                }
            }
        }

        /// Increases the amount of tokens locked in the contract.
        ///
        /// - `additional_amount`: The additional amount of tokens to lock.
        #[ink(message)]
        pub fn increase_unlock_time(&mut self, new_unlock_time: u32) -> Result<(), Error> {
            if new_unlock_time > self.max_time {
                return Err(Error::UnlockTimeToFarInTheFuture);
            }
            match self.locked_balances.get(self.env().caller()) {
                None => return Err(Error::NoLockedBalance),
                Some((amount, start_time, lock_time)) => {
                    let elapsed_time = self.env().block_number().saturating_sub(start_time);
                    let remaining_time = lock_time.saturating_sub(elapsed_time);
                    if remaining_time > new_unlock_time {
                        return Err(Error::OnlyIncreaseOfLockPossible);
                    }
                    self.locked_balances.insert(
                        self.env().caller(),
                        &(amount, self.env().block_number(), new_unlock_time),
                    );
                    self.env().emit_event(LockTimeIncreased {
                        account: self.env().caller(),
                        new_unlock_time,
                    });
                    Ok(())
                }
            }
        }

        /// Withdraw all unlocked tokens for the caller
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
                    self.env().emit_event(Withdrawn {
                        account: self.env().caller(),
                        amount,
                    });
                    Ok(())
                }
            }
        }

        /// Increases the amount of tokens locked in the contract.
        ///
        /// - `additional_amount`: The additional amount of tokens to lock.
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

            voting_power * self.boost as u128 + amount
        }
    }
}
