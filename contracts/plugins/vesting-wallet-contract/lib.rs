#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! ### Vesting Wallet
//!
//! This contract implements a vesting wallet for tokens.
//! The vesting schedule is defined by a start time, total duration, and the total tokens to be
//! vested.
//!
//! Follows this function:
//!
//! $$U(t) = V_{t} - \frac{t - t_0}{T}V_{t}$$
//!
//! Where:
//!
//! - $U(t)$ is the number of unvested tokens at time t,
//! - $t$ is the time elapsed since the beginning of the vesting period,
//! - $t_0$ is the starting time of the vesting period,
//! - $T$ is the total duration of the vesting period,
//! - $V_{t}$ is the total number of tokens to be vested.

#[ink::contract]
pub mod vesting_wallet {
	use ink::{
		env::{
			call::{build_call, ExecutionInput, Selector},
			DefaultEnvironment,
		},
		storage::Mapping,
	};

	/// Emitted when a new vesting wallet is initialized.
	#[ink(event)]
	pub struct VestingWalletInitialized {
		#[ink(topic)]
		token: AccountId
	}

	/// Event emitted when a new vesting wallet is created.
	#[ink(event)]
	pub struct VestingWalletCreated {
		#[ink(topic)]
		account: AccountId,
		amount: Balance,
		duration: u32,
	}

	/// Event emitted when tokens are successfully withdrawn from a vesting wallet.
	#[ink(event)]
	pub struct TokensWithdrawn {
		#[ink(topic)]
		account: AccountId,
		amount: Balance,
	}

	#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum Error {
		/// Raised when we are unable to fund the vesting wallet from the sender,
		/// most likely because the sender has not approved the contract
		CannotFund,
		/// Raised when we are unable to withdraw from the vesting wallet
		WithdrawFailed,
	}

	#[derive(scale::Decode, scale::Encode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
	pub struct VestingWalletInfo {
		start_time: BlockNumber,
		duration: u32,
		total_tokens: Balance,
		withdrawn_tokens: Balance,
	}

	#[ink(storage)]
	pub struct VestingWallets {
		token: AccountId,
		wallets: Mapping<AccountId, VestingWalletInfo>,
	}

	impl VestingWallets {
		/// Initializes a new VestingWallets contract.
		///
		/// # Arguments
		///
		/// - `token`: The AccountId of the token contract that will be vested.
		#[ink(constructor)]
		pub fn new(token: AccountId) -> Self {
			let contract = Self { token, wallets: Mapping::new() };
			Self::env().emit_event(VestingWalletInitialized { token });
			contract
		}

		/// Returns the AccountId of the token contract.
		#[ink(message)]
		pub fn get_token(&self) -> AccountId {
			self.token
		}

		/// Creates a vesting wallet for a given account.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId for whom the vesting wallet will be created.
		/// - `amount`: The total amount of tokens to be vested.
		/// - `duration`: The total duration over which the tokens will be vested.
		#[ink(message)]
		pub fn create_vesting_wallet_for(
			&mut self,
			account: AccountId,
			amount: Balance,
			duration: u32,
		) -> Result<(), Error> {
			match build_call::<DefaultEnvironment>()
				.call(self.token)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!(
						"PSP22::transfer_from"
					)))
					.push_arg(&self.env().caller())
					.push_arg(&self.env().account_id())
					.push_arg(&amount)
					.push_arg(&""),
				)
				.returns::<Result<(), Error>>()
				.try_invoke()
			{
				Err(_) => Err(Error::CannotFund),
				Ok(_) => {
					let wallet_info = VestingWalletInfo {
						start_time: self.env().block_number(),
						duration,
						total_tokens: amount,
						withdrawn_tokens: 0,
					};
					self.wallets.insert(account, &wallet_info);
					self.env().emit_event(VestingWalletCreated { account, amount, duration });
					Ok(())
				},
			}
		}

		/// Creates a vesting wallet for a given account.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId for whom the vesting wallet will be created.
		/// - `amount`: The total amount of tokens to be vested.
		/// - `duration`: The total duration over which the tokens will be vested.
		#[ink(message)]
		pub fn get_unvested(&self, account: AccountId) -> Balance {
			match self.wallets.get(account) {
				None => 0,
				Some(wallet) => {
					let now: Balance = self.env().block_number().into();
					let start: Balance = wallet.start_time.into();
					let duration: Balance = wallet.duration.into();

					if duration == 0 {
						// a bit of a pointless vesting wallet but we'll allow it
						wallet.total_tokens
					} else if duration < now - start {
						// vesting period is over
						0
					} else {
						let vested = (now - start) * wallet.total_tokens / duration;
						wallet.total_tokens - vested
					}
				},
			}
		}

		/// Returns the number of tokens that have been withdrawn so far for a given account.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId to check for withdrawn tokens.
		#[ink(message)]
		pub fn get_withdrawn(&self, account: AccountId) -> Balance {
			match self.wallets.get(account) {
				None => 0,
				Some(wallet) => wallet.withdrawn_tokens,
			}
		}

		/// Returns the number of tokens that have been withdrawn so far for a given account.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId to check for withdrawn tokens.
		#[ink(message)]
		pub fn get_available_for_withdraw(&self, account: AccountId) -> Balance {
			match self.wallets.get(account) {
				None => 0,
				Some(wallet) =>
					wallet.total_tokens - wallet.withdrawn_tokens - self.get_unvested(account),
			}
		}

		/// Returns the total number of tokens held by the vesting wallet for a given account.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId to check for total tokens.
		#[ink(message)]
		pub fn get_total(&self, account: AccountId) -> Balance {
			self.get_unvested(account) + self.get_available_for_withdraw(account)
		}

		/// Allows an account to withdraw vested tokens.
		///
		/// # Arguments
		///
		/// - `account`: The AccountId from which tokens will be withdrawn.
		#[ink(message)]
		pub fn withdraw(&mut self, account: AccountId) -> Result<(), Error> {
			let mut wallet = match self.wallets.get(account) {
				None => return Ok(()),
				Some(wallet) => wallet,
			};
			let amount = self.get_available_for_withdraw(account);
			if amount == 0 {
				return Ok(())
			}

			match build_call::<DefaultEnvironment>()
				.call(self.token)
				.exec_input(
					ExecutionInput::new(Selector::new(ink::selector_bytes!("PSP22::transfer")))
						.push_arg(&account)
						.push_arg(&amount)
						.push_arg(&""),
				)
				.returns::<Result<(), Error>>()
				.try_invoke()
			{
				Err(_) => Err(Error::WithdrawFailed),
				Ok(_) => {
					// this is fine as reentrancy is disabled by default
					wallet.withdrawn_tokens += &amount;
					self.wallets.insert(account, &wallet);
					self.env().emit_event(TokensWithdrawn { account, amount });
					Ok(())
				},
			}
		}
	}

	impl plugins::Vote for VestingWallets {
		#[ink(message)]
		fn get_id(&self) -> u32 {
			1
		}

		#[ink(message)]
		fn get_voting_power(&self, voter: AccountId, voting_power: Balance) -> Balance {
			voting_power + self.get_total(voter)
		}
	}
}
