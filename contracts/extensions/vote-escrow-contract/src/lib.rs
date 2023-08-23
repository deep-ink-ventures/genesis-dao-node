#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::env::Environment;
use vote_escrow_extension::VoteEscrowExtension;

macro_rules! ensure {
	( $x:expr, $y:expr $(,)? ) => {{
		if !$x {
			return Err($y)
		}
	}};
}

/// A custom environment for the VoteEscrow contract, using the default Ink environment but
/// with a specific chain extension for asset operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = vote_escrow_extension::AccountId;
	type Balance = vote_escrow_extension::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

	type ChainExtension = VoteEscrowExtension;
}

#[derive(scale::Encode, scale::Decode, Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	NotAllowed,
	AlreadyCreated,
	LockPeriodTooLong,
	BalanceLow,
	TokenDepositFailure,
	TokenWithdrawFailure,
	NoLockCreated,
	NotExpired,
}

#[ink::contract(env = crate::CustomEnvironment)]
mod vote_escrow_contract {
	use ink::prelude::vec::Vec;

	use super::*;
	use ink::{
		env::{
			call::{build_call, ExecutionInput, Selector},
			DefaultEnvironment,
		},
		storage::{traits::StorageLayout, Mapping},
	};

	pub const DAY: Timestamp = 24 * 60 * 60;
	pub const YEAR: Timestamp = 365 * DAY;

	#[derive(scale::Encode, scale::Decode, Debug, Default, PartialEq)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
	pub struct LockedBalance {
		value: Balance,
		created_at: Timestamp,
		end_time: Timestamp,
	}

	/// Defines the storage of your contract.
	/// Add new fields to the below struct in order
	/// to add new static storage fields to your contract.
	#[ink(storage)]
	pub struct VoteEscrowContract {
		/// PSP token that is defined as the VoteEscrow token
		token: AccountId,
		/// The maximum lock time (e.g., 4 years in seconds)
		max_lock_time: u64,
		/// The user's locked balances
		locked_balances: Mapping<AccountId, LockedBalance>,
		/// The owner of the contract
		owner: AccountId,
	}

	#[ink(event)]
	pub struct OwnershipTransferred {
		#[ink(topic)]
		pub block: BlockNumber,
		pub new_owner: AccountId,
	}

	#[ink(event)]
	pub struct MaxTimeUpdated {
		#[ink(topic)]
		pub block: BlockNumber,
		pub new_max_lock_time: Timestamp,
	}

	#[ink(event)]
	pub struct LockCreated {
		#[ink(topic)]
		pub owner: AccountId,
		pub value: Balance,
		pub created_at: Timestamp,
		pub lock_period: Timestamp,
	}

	#[ink(event)]
	pub struct LockValueIncreased {
		#[ink(topic)]
		pub owner: AccountId,
		pub increase: Balance,
		pub new_value: Balance,
	}

	#[ink(event)]
	pub struct LockPeriodExtended {
		#[ink(topic)]
		pub owner: AccountId,
		pub extend: Timestamp,
		pub new_end_time: Timestamp,
	}

	#[ink(event)]
	pub struct Withdrawal {
		#[ink(topic)]
		pub owner: AccountId,
		pub timestamp: Timestamp,
		pub value: Balance,
	}

	impl VoteEscrowContract {
		/// Constructor
		/// Initialize the contract with the given token address
		/// 	`token`: Address of the PSP22 contract
		#[ink(constructor)]
		pub fn new(token: AccountId) -> Self {
			Self {
				token,
				max_lock_time: 4 * YEAR, // 4 years
				locked_balances: Default::default(),
				owner: Self::env().caller(),
			}
		}

		/// Transfer ownership
		/// Transfer the ownership of the contract to a new account
		/// 	`new_owner`: new owner of the contract
		#[ink(message)]
		pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<(), Error> {
			let caller = self.env().caller();

			ensure!(caller == self.owner, Error::NotAllowed);

			self.owner = new_owner;
			let block = self.env().block_number();
			self.env().emit_event(OwnershipTransferred { block, new_owner });

			Ok(())
		}

		/// Set max lock time
		/// 	`new_max_lock_time`: new max_lock_time
		#[ink(message)]
		pub fn set_max_lock_time(&mut self, new_max_lock_time: Timestamp) -> Result<(), Error> {
			let caller = self.env().caller();

			ensure!(caller == self.owner, Error::NotAllowed);

			self.max_lock_time = new_max_lock_time;

			let block = self.env().block_number();

			self.env().emit_event(MaxTimeUpdated { block, new_max_lock_time });

			Ok(())
		}

		/// Create lock
		/// Allow users to lock tokens for a specific period
		/// 	`value`: token amount to lcok
		/// 	`lock_period`: lock period
		#[ink(message)]
		pub fn create_lock(&mut self, value: Balance, lock_period: Timestamp) -> Result<(), Error> {
			let caller = self.env().caller();

			// Check if the caller has already created a lock
			ensure!(self.locked_balances.get(caller).is_none(), Error::AlreadyCreated);

			// Ensure that the lock_period is no longer than the max_time
			ensure!(lock_period <= self.max_lock_time, Error::LockPeriodTooLong);

			// Ensure that the user has enough token balance to lock
			let balance = build_call::<DefaultEnvironment>()
				.call(self.token)
				.gas_limit(0)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("balance_of")))
						.push_arg(caller),
				)
				.returns::<Balance>()
				.invoke();

			ensure!(balance >= value, Error::BalanceLow);

			// Check if the deposit into the contract account is successful

			let deposit = build_call::<DefaultEnvironment>()
				.call(self.token)
				.gas_limit(0)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer_from")))
						.push_arg(caller)
						.push_arg(self.env().account_id())
						.push_arg(value)
						.push_arg(Vec::<u8>::default()),
				)
				.returns::<Result<(), ()>>()
				.invoke();

			ensure!(deposit.is_ok(), Error::TokenDepositFailure);

			// Okay, we can create a lock record now

			let timestamp: Timestamp = self.env().block_timestamp();
			let end_time: Timestamp = timestamp + lock_period;
			let lock = LockedBalance { value, created_at: timestamp, end_time };

			self.locked_balances.insert(caller, &lock);

			self.env().emit_event(LockCreated {
				owner: caller,
				value,
				created_at: timestamp,
				lock_period,
			});

			Ok(())
		}

		/// Increase Amount
		/// Allow users to increase the amount of locked tokens without updating the lock period
		/// 	`value`: value to increase
		#[ink(message)]
		pub fn increase_amount(&mut self, value: Balance) -> Result<(), Error> {
			let caller = self.env().caller();

			let lock = self.locked_balances.get(caller);

			ensure!(lock.is_some(), Error::NoLockCreated);

			let mut lock = lock.unwrap();

			lock.value += value;

			self.locked_balances.insert(caller, &lock);

			self.env().emit_event(LockValueIncreased {
				owner: caller,
				increase: value,
				new_value: lock.value,
			});

			Ok(())
		}

		// Increase lock period
		// Extends the unlock time
		//		`extend_time`: Time to extend
		#[ink(message)]
		pub fn increase_unlock_time(&mut self, extend_time: Timestamp) -> Result<(), Error> {
			let caller = self.env().caller();

			// Ensure that the user has created a lock
			let lock = self.locked_balances.get(caller);
			ensure!(lock.is_some(), Error::NoLockCreated);

			// Ensure that the extended lock period does not exceed the max allowed time
			let mut lock = lock.unwrap();
			let new_lock_period = lock.end_time - lock.created_at + extend_time;

			ensure!(new_lock_period <= self.max_lock_time, Error::LockPeriodTooLong);

			// Update the record
			lock.end_time += extend_time;

			self.locked_balances.insert(caller, &lock);

			self.env().emit_event(LockPeriodExtended {
				owner: caller,
				extend: extend_time,
				new_end_time: lock.end_time,
			});

			Ok(())
		}

		// Withdraw
		// Allow users to withdraw their tokens after the lock has expired
		#[ink(message)]
		pub fn withdraw(&mut self) -> Result<(), Error> {
			let caller = self.env().caller();

			// Ensure that the user has created a lock
			let lock = self.locked_balances.get(caller);
			ensure!(lock.is_some(), Error::NotAllowed);

			let lock = lock.unwrap();

			// Ensure that the lock has expired
			let timestamp = self.env().block_timestamp();
			ensure!(timestamp > lock.end_time, Error::NotExpired);

			let value = lock.value;

			let withdraw = build_call::<DefaultEnvironment>()
				.call(self.token)
				.gas_limit(0)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("transfer")))
						.push_arg(caller)
						.push_arg(value)
						.push_arg(Vec::<u8>::default()),
				)
				.returns::<Result<(), ()>>()
				.invoke();

			ensure!(withdraw.is_ok(), Error::TokenWithdrawFailure);

			// Now remove the record to allow the user to create another lock in the future
			self.locked_balances.remove(caller);

			self.env().emit_event(Withdrawal { owner: caller, timestamp, value });

			Ok(())
		}
	}
}
