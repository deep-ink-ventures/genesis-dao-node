#![cfg_attr(not(feature = "std"), no_std, no_main)]
use ink::prelude::vec::Vec;
use scale::Encode;

pub type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
pub type Balance = u128;

#[ink::chain_extension]
pub trait VoteEscrowExtension {
	type ErrorCode = VoteEscrowError;

	// #[ink(extension = 100)]
	// fn transfer(asset_id: AssetId, target: AccountId, amount: Balance) -> Result<(),
	// VoteEscrowError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum VoteEscrowError {
	/// Unknown error
	RuntimeError,
	/// Encoding error
	EncodingError,
}

impl ink::env::chain_extension::FromStatusCode for VoteEscrowError {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			14 => Err(Self::RuntimeError),

			_ => panic!("encountered unknown status code"),
		}
	}
}

impl From<scale::Error> for VoteEscrowError {
	fn from(_: scale::Error) -> Self {
		Self::EncodingError
	}
}

impl VoteEscrowError {
	pub fn to_bytes(&self) -> Vec<u8> {
		match self {
			VoteEscrowError::RuntimeError => "RuntimeError".encode(),
			VoteEscrowError::EncodingError => "EncodingError".encode(),
		}
	}
}
