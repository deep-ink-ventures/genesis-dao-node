#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod types;
pub use types::*;

use ink::prelude::vec::Vec;
use scale::Encode;

#[ink::chain_extension]
pub trait AssetExtension {
	type ErrorCode = AssetError;

	#[ink(extension = 100)]
	fn transfer(asset_id: AssetId, target: AccountId, amount: Balance) -> Result<(), AssetError>;

	#[ink(extension = 101)]
	fn transfer_keep_alive(
		asset_id: AssetId,
		target: AccountId,
		amount: Balance,
	) -> Result<(), AssetError>;

	#[ink(extension = 102)]
	fn approve(asset_id: AssetId, delegate: AccountId, amount: Balance) -> Result<(), AssetError>;

	#[ink(extension = 103)]
	fn cancel_approval(asset_id: AssetId, delegate: AccountId) -> Result<(), AssetError>;

	#[ink(extension = 104)]
	fn transfer_from(
		asset_id: AssetId,
		owner: AccountId,
		destination: AccountId,
		amount: Balance,
	) -> Result<(), AssetError>;

	#[ink(extension = 105)]
	fn balance_of(asset_id: AssetId, account: AccountId) -> Result<u128, AssetError>;

	#[ink(extension = 106)]
	fn total_supply(asset_id: AssetId) -> Result<u128, AssetError>;

	#[ink(extension = 107)]
	fn allowance(
		asset_id: AssetId,
		owner: AccountId,
		spender: AccountId,
	) -> Result<u128, AssetError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AssetError {
	/// Account balance must be greater than or equal to the transfer amount.
	BalanceLow,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
	/// The asset ID is already taken.
	InUse,
	/// Invalid witness data given.
	BadWitness,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// Invalid metadata given.
	BadMetadata,
	/// No approval exists that would allow the transfer.
	Unapproved,
	/// The source account would not survive the transfer and it needs to stay alive.
	WouldDie,
	/// The asset-account already exists.
	AlreadyExists,
	/// The operation would result in funds being burned.
	WouldBurn,
	/// The asset is not live, and likely being destroyed.
	AssetNotLive,
	/// Unknown error
	RuntimeError,
	/// Encoding error
	EncodingError,
}

impl ink::env::chain_extension::FromStatusCode for AssetError {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			1 => Err(Self::BalanceLow),
			2 => Err(Self::NoAccount),
			3 => Err(Self::NoPermission),
			4 => Err(Self::Unknown),
			5 => Err(Self::InUse),
			6 => Err(Self::BadWitness),
			7 => Err(Self::MinBalanceZero),
			8 => Err(Self::BadMetadata),
			9 => Err(Self::Unapproved),
			10 => Err(Self::WouldDie),
			11 => Err(Self::AlreadyExists),
			12 => Err(Self::WouldBurn),
			13 => Err(Self::AssetNotLive),
			14 => Err(Self::RuntimeError),

			_ => panic!("encountered unknown status code"),
		}
	}
}

impl From<scale::Error> for AssetError {
	fn from(_: scale::Error) -> Self {
		Self::EncodingError
	}
}

impl AssetError {
	pub fn to_bytes(&self) -> Vec<u8> {
		match self {
			AssetError::BalanceLow => "BalanceLow".encode(),
			AssetError::NoAccount => "NoAccount".encode(),
			AssetError::NoPermission => "NoPermission".encode(),
			AssetError::Unknown => "Unknown".encode(),
			AssetError::InUse => "InUse".encode(),
			AssetError::BadWitness => "BadWitness".encode(),
			AssetError::MinBalanceZero => "MinBalanceZero".encode(),
			AssetError::BadMetadata => "BadMetadata".encode(),
			AssetError::Unapproved => "Unapproved".encode(),
			AssetError::WouldDie => "WouldDie".encode(),
			AssetError::AlreadyExists => "AlreadyExists".encode(),
			AssetError::WouldBurn => "WouldBurn".encode(),
			AssetError::AssetNotLive => "AssetNotLive".encode(),
			AssetError::RuntimeError => "RuntimeError".encode(),
			AssetError::EncodingError => "EncodingError".encode(),
		}
	}
}
