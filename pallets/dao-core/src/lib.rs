#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod functions;

mod types;
pub use types::Dao;

pub use frame_support::{
	sp_runtime::traits::{One, Saturating},
	storage::bounded_vec::BoundedVec,
	traits::{
		tokens::fungibles::{metadata::Mutate as MetadataMutate, Mutate},
		Currency,
	},
	weights::Weight,
};
use pallet_dao_assets::Pallet as Assets;

pub mod weights;
use weights::WeightInfo;
pub use crate::types::{DaoNameOf, DaoIdOf, AssetIdOf, CurrencyOf, AccountIdOf, DepositBalanceOf, MetadataOf, DaoOf};

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{pallet_prelude::*, traits::ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use crate::types::DepositBalanceOf;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_dao_assets::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: ReservableCurrency<Self::AccountId>;

		type AssetId: IsType<<Self as pallet_dao_assets::Config>::AssetId>
			+ Member
			+ Parameter
			+ Copy
			+ Default
			+ MaxEncodedLen
			+ One
			+ Saturating;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type DaoDeposit: Get<DepositBalanceOf<Self>>;

		#[pallet::constant]
		type MinLength: Get<u32>;

		#[pallet::constant]
		type MaxLengthId: Get<u32>;

		#[pallet::constant]
		type MaxLengthName: Get<u32>;

		#[pallet::constant]
		type MaxLengthMetadata: Get<u32>;

		#[pallet::constant]
		type TokenUnits: Get<u8>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		DaoCreated {
			dao_id: DaoIdOf<T>,
			owner: T::AccountId,
		},
		DaoDestroyed {
			dao_id: DaoIdOf<T>,
		},
		DaoTokenIssued {
			dao_id: DaoIdOf<T>,
			supply: <T as pallet_dao_assets::Config>::Balance,
			asset_id: <T as Config>::AssetId,
		},
		DaoMetadataSet {
			dao_id: DaoIdOf<T>,
		},
		DaoOwnerChanged {
			dao_id: DaoIdOf<T>,
			new_owner: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		DaoIdInvalidLengthTooShort,
		DaoIdInvalidLengthTooLong,
		DaoIdInvalidChar,
		DaoNameInvalidLengthTooShort,
		DaoNameInvalidLengthTooLong,
		DaoAlreadyExists,
		DaoDoesNotExist,
		DaoSignerNotOwner,
		DaoTokenAlreadyIssued,
		MetadataInvalidLengthTooLong,
		MetadataInvalid,
		HashInvalidWrongLength,
	}

	/// Key-Value Store of all _DAOs_, with the key being the `dao_id`.
	#[pallet::storage]
	#[pallet::getter(fn get_dao)]
	pub type Daos<T: Config> = StorageMap<_, Blake2_128Concat, DaoIdOf<T>, DaoOf<T>>;

	/// Internal incrementor of all assets issued by this module.
	/// The first asset starts with _1_ (sic!, not 0) and then the id is assigned by order of
	/// creation.
	#[pallet::storage]
	#[pallet::getter(fn get_current_asset_id)]
	pub type CurrentAssetId<T> = StorageValue<_, AssetIdOf<T>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a fresh DAO.
		///
		/// - `dao_id`: Unique identifier for the DAO, bounded by _MinLength_ & _MaxLengthId_
		/// - `dao_name`: Name of the to-be-created DAO, bounded by _MinLength_ & _MaxLengthName_
		///
		/// A DAO must reserve the _DaoDeposit_ fee.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_dao())]
		pub fn create_dao(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			dao_name: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let id: BoundedVec<_, _> =
				dao_id.try_into().map_err(|_| Error::<T>::DaoIdInvalidLengthTooLong)?;
			ensure!(
				id.len() >= T::MinLength::get() as usize,
				Error::<T>::DaoIdInvalidLengthTooShort
			);
			ensure!(
				id.iter().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()),
				Error::<T>::DaoIdInvalidChar
			);
			ensure!(!<Daos<T>>::contains_key(&id), Error::<T>::DaoAlreadyExists);

			let name: BoundedVec<_, _> =
				dao_name.try_into().map_err(|_| Error::<T>::DaoNameInvalidLengthTooLong)?;
			ensure!(
				name.len() >= T::MinLength::get() as usize,
				Error::<T>::DaoNameInvalidLengthTooShort
			);

			<T as Config>::Currency::reserve(&sender, <T as Config>::DaoDeposit::get())?;

			Self::deposit_event(Event::DaoCreated { owner: sender.clone(), dao_id: id.clone() });
			let dao = Dao {
				id: id.clone(),
				name,
				owner: sender,
				asset_id: None,
				meta: Default::default(),
				meta_hash: Default::default(),
			};
			<Daos<T>>::insert(id, dao);
			Ok(())
		}

		/// Destroy a DAO.
		///
		/// - `dao_id`: The DAO to destroy
		///
		/// Signer of this TX needs to be the owner of the DAO.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::destroy_dao())]
		pub fn destroy_dao(origin: OriginFor<T>, dao_id: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = Self::load_dao(dao_id)?;
			ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);

			if let Some(asset_id) = dao.asset_id {
				if let Some(asset) = pallet_dao_assets::Asset::<T>::get(asset_id.into()) {
					if pallet_dao_assets::AssetStatus::Destroyed != asset.status {
						Err(Error::<T>::DaoTokenAlreadyIssued)?;
					}
				}
			}

			<T as Config>::Currency::unreserve(&sender, <T as Config>::DaoDeposit::get());
			Self::deposit_event(Event::DaoDestroyed { dao_id: dao.id.clone() });
			<Daos<T>>::remove(&dao.id);
			Ok(())
		}

		/// Issue the DAO token
		///
		/// - `dao_id`: The DAO for which to issue a token
		/// - `supply`: The total supply of the token to be issued
		///
		/// Tokens can only be issued once and the signer of this TX needs to be the owner
		/// of the DAO.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::issue_token())]
		pub fn issue_token(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			supply: <T as pallet_dao_assets::Config>::Balance,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = Self::load_dao(dao_id)?;
			ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);
			ensure!(dao.asset_id.is_none(), Error::<T>::DaoTokenAlreadyIssued);

			// create a fresh asset
			<CurrentAssetId<T>>::mutate(|asset_id| asset_id.saturating_inc());

			<pallet_dao_assets::pallet::Pallet<T>>::do_force_create(
				<CurrentAssetId<T>>::get().into(),
				dao.owner.clone(),
				One::one(),
			)?;

			// and distribute it to the owner
			<pallet_dao_assets::pallet::Pallet<T>>::do_mint(
				<CurrentAssetId<T>>::get().into(),
				&dao.owner,
				supply,
			)?;

			// set the token metadata to the dao metadata
			<pallet_dao_assets::pallet::Pallet<T>>::do_set_metadata(
				<CurrentAssetId<T>>::get().into(),
				&dao.owner,
				dao.name.into(),
				dao.id.clone().into(),
				<T as Config>::TokenUnits::get(),
			)?;

			Self::deposit_event(Event::DaoTokenIssued {
				dao_id: dao.id.clone(),
				supply,
				asset_id: <CurrentAssetId<T>>::get(),
			});
			// ... and link the dao to the asset
			<Daos<T>>::try_mutate(dao.id, |maybe_dao| {
				let d = maybe_dao.as_mut().ok_or(Error::<T>::DaoDoesNotExist)?;
				d.asset_id = Some(<CurrentAssetId<T>>::get());
				Ok(())
			})
		}

		/// Set metadata
		///
		/// - `dao_id`: The DAO for which to set metadata
		/// - `meta`: HTTP or IPFS address for the metadata about this DAO (description, logo)
		/// - `hash`: SHA3 hash of the metadata to be found via `meta`
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_metadata())]
		pub fn set_metadata(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			meta: Vec<u8>,
			hash: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao = Self::load_dao(dao_id)?;
			ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);

			let meta: BoundedVec<_, _> =
				meta.try_into().map_err(|_| Error::<T>::MetadataInvalidLengthTooLong)?;
			let hash: BoundedVec<_, _> =
				hash.try_into().map_err(|_| Error::<T>::HashInvalidWrongLength)?;
			ensure!(
				meta.is_empty() && hash.is_empty() || Self::metadata_is_valid(&meta),
				Error::<T>::MetadataInvalid
			);

			Self::deposit_event(Event::DaoMetadataSet { dao_id: dao.id.clone() });

			<Daos<T>>::try_mutate(dao.id, |maybe_dao| {
				let dao = maybe_dao.as_mut().ok_or(Error::<T>::DaoDoesNotExist)?;
				dao.meta = meta;
				dao.meta_hash = hash;
				Ok(())
			})
		}

		/// Change owner
		///
		/// - `dao_id`: the DAO to transfer ownership of
		/// - `new_owner`: the new owner
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::set_metadata())]
		pub fn change_owner(
			origin: OriginFor<T>,
			dao_id: Vec<u8>,
			new_owner: T::AccountId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let dao_id: BoundedVec<_, _> =
				dao_id.try_into().map_err(|_| Error::<T>::DaoIdInvalidLengthTooLong)?;
			Daos::<T>::try_mutate(dao_id.clone(), |maybe_dao| -> DispatchResult {
				let dao = maybe_dao.as_mut().ok_or(Error::<T>::DaoDoesNotExist)?;
				ensure!(dao.owner == sender, Error::<T>::DaoSignerNotOwner);
				if dao.owner == new_owner {
					return Ok(())
				}
				// also change asset owner if token was issued
				if let Some(asset_id) = dao.asset_id {
					Assets::<T>::change_owner(asset_id.into(), new_owner.clone())?;
				}

				dao.owner = new_owner.clone();
				Ok(())
			})?;
			Self::deposit_event(Event::DaoOwnerChanged { dao_id, new_owner });
			Ok(())
		}
	}
}
