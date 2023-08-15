#![cfg_attr(not(feature = "std"), no_std, no_main)]

use dao_assets_extension::AssetExtension;
use ink::env::Environment;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = dao_assets_extension::AccountId;
	type Balance = dao_assets_extension::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

	type ChainExtension = AssetExtension;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod dao_assets_contract {
	use dao_assets_extension::{AssetError, AssetId};

	/// Defines the storage of our contract.
	///
	/// Here we store the random seed fetched from the chain.
	#[ink(storage)]
	pub struct AssetContract {
		asset_id: AssetId,
	}

	impl AssetContract {
		#[ink(constructor)]
		pub fn new(asset_id: AssetId) -> Self {
			Self { asset_id }
		}

		#[ink(message)]
		pub fn transfer(&self, target: AccountId, amount: Balance) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env().extension().transfer(sender, asset_id, target, amount)
		}

		#[ink(message)]
		pub fn transfer_keep_alive(
			&self,
			target: AccountId,
			amount: Balance,
		) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env().extension().transfer_keep_alive(sender, asset_id, target, amount)
		}

		#[ink(message)]
		pub fn approve_transfer(
			&self,
			delegate: AccountId,
			amount: Balance,
		) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env().extension().approve_transfer(sender, asset_id, delegate, amount)
		}

		#[ink(message)]
		pub fn cancel_approval(&self, delegate: AccountId) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env().extension().cancel_approval(sender, asset_id, delegate)
		}

		#[ink(message)]
		pub fn transfer_approved(
			&self,
			owner: AccountId,
			destination: AccountId,
			amount: Balance,
		) -> Result<(), AssetError> {
			let sender = self.env().caller();
			let asset_id = self.asset_id.clone();

			self.env()
				.extension()
				.transfer_approved(sender, asset_id, owner, destination, amount)
		}
	}
}
