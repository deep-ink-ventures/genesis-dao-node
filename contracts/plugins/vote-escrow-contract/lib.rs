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
        /// The unlock time is too far in the future, it may not exceed the maximum lock time
        UnlockTimeToFarInTheFuture,
        /// Could not transfer_from the PSP22 tokens, most likely
        /// because the user has not approved the contract
        UnableToLockTokens,
        /// The user has no locked balance
        NoLockedBalance,
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

        /// Create a new token lock
        #[ink(message)]
        pub fn create_lock(&mut self, amount: u128, unlock_time: u32) -> Result<(), Error> {
            if unlock_time > self.max_time {
                return Err(Error::UnlockTimeToFarInTheFuture);
            }
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
                Err(_) => Err(Error::UnableToLockTokens),
                Ok(_) => {
                    let start_time = self.env().block_timestamp();
                    self.locked_balances.insert(self.env().caller(), &(
                        amount,
                        self.env().block_number(),
                        unlock_time
                    ));
                    Ok(())
                }
            }
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
            // TODO: Implement the logic for withdrawing unlocked tokens
            // - Check that the unlock time has passed
            // - Update the locked_balances mapping
            // - Interact with the PSP22 token contract to transfer tokens back to the user
            Ok(())
        }

        /// Query voting power
        #[ink(message)]
        pub fn voting_power(&self, account: AccountId, at_time: u32) -> u128 {
            // TODO: Implement the logic for querying voting power
            // - Use the decay function to calculate the voting power
            0
        }
    }
}
