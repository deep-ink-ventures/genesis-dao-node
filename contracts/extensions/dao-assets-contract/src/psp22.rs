use dao_assets_extension::{AccountId, AssetError, Balance};
use ink::prelude::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PSP22Error {
	/// Custom error type for cases in which an implementation adds its own restrictions.
	Custom(Vec<u8>),
	/// Returned if not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// Returned if not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Returned if recipient's address is zero.
	ZeroRecipientAddress,
	/// Returned if sender's address is zero.
	ZeroSenderAddress,
	/// Returned if a safe transfer check fails (e.g. if the receiving contract does not accept
	/// tokens).
	SafeTransferCheckFailed(Vec<u8>),
}

#[ink::trait_definition]
pub trait PSP22 {
	/// Returns the total token supply.
	#[ink(message)]
	fn total_supply(&self) -> Balance;

	/// Returns the account Balance for the specified `owner`.
	///
	/// Returns `0` if the account is non-existent.
	#[ink(message)]
	fn balance_of(&self, owner: AccountId) -> Balance;

	/// Returns the amount which `spender` is still allowed to withdraw from `owner`.
	///
	/// Returns `0` if no allowance has been set `0`.
	#[ink(message)]
	fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

    /// Transfers `value` amount of tokens from the caller's account to account `to`
    /// with additional `data` in unspecified format.
    ///
    /// On success a `Transfer` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientBalance` error if there are not enough tokens on
    /// the caller's account Balance.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`
    /// with additional `data` in unspecified format.
    ///
    /// This can be used to allow a contract to transfer tokens on ones behalf and/or
    /// to charge fees in sub-currencies, for example.
    ///
    /// On success a `Transfer` and `Approval` events are emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
    /// for the caller to withdraw from `from`.
    ///
    /// Returns `InsufficientBalance` error if there are not enough tokens on
    /// the the account Balance of `from`.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[ink(message)]
    fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        value: Balance,
        data: Vec<u8>,
    ) -> Result<(), PSP22Error>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the `value` amount.
    ///
    /// If this function is called again it overwrites the current allowance with `value`.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[ink(message)]
    fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

    /// Atomically increases the allowance granted to `spender` by the caller.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[ink(message)]
    fn increase_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error>;

    /// Atomically decreases the allowance granted to `spender` by the caller.
    ///
    /// An `Approval` event is emitted.
    ///
    /// # Errors
    ///
    /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
    /// by owner for `spender`.
    ///
    /// Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    /// Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[ink(message)]
    fn decrease_allowance(&mut self, spender: AccountId, delta_value: Balance) -> Result<(), PSP22Error>;
}

impl From<AssetError> for PSP22Error {
	fn from(value: AssetError) -> Self {
		match value {
			AssetError::BalanceLow => PSP22Error::InsufficientBalance,
			AssetError::Unapproved => PSP22Error::InsufficientAllowance,
			_ => PSP22Error::Custom(value.to_bytes()),
		}
	}
}
