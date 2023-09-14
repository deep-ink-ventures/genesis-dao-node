use dao_assets_extension::AssetError;
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

impl From<AssetError> for PSP22Error {
	fn from(value: AssetError) -> Self {
		match value {
			AssetError::BalanceLow => PSP22Error::InsufficientBalance,
			AssetError::Unapproved => PSP22Error::InsufficientAllowance,
			_ => PSP22Error::Custom(value.to_bytes()),
		}
	}
}
