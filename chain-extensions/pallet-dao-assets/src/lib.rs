#![cfg_attr(not(feature = "std"), no_std)]
use frame_system::RawOrigin;
use pallet_dao_assets::WeightInfo;
use pallet_contracts::chain_extension::{
    ChainExtension, Environment, Ext, InitState, RetVal, SysConfig,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::StaticLookup;
use sp_runtime::{DispatchError, ModuleError};
use sp_std::marker::PhantomData;

enum AssetsFunc {
    Transfer = 100
}

impl TryFrom<u16> for AssetsFunc {
    type Error = DispatchError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(AssetsFunc::Transfer),
            _ => Err(DispatchError::Other(
                "PalletDaoAssetsExtension: Unimplemented func_id",
            )),
        }
    }
}


#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Outcome {
    /// Success
    Success = 0,
    /// Account balance must be greater than or equal to the transfer amount.
    BalanceLow = 1,
    /// The account to alter does not exist.
    NoAccount = 2,
    /// The signing account has no permission to do the operation.
    NoPermission = 3,
    /// The given asset ID is unknown.
    Unknown = 4,
    /// The asset ID is already taken.
    InUse = 5,
    /// Invalid witness data given.
    BadWitness = 6,
    /// Minimum balance should be non-zero.
    MinBalanceZero = 7,
    /// Invalid metadata given.
    BadMetadata = 8,
    /// No approval exists that would allow the transfer.
    Unapproved = 9,
    /// The source account would not survive the transfer and it needs to stay alive.
    WouldDie = 10,
    /// The asset-account already exists.
    AlreadyExists = 11,
    /// The operation would result in funds being burned.
    WouldBurn = 12,
    /// The asset is not live, and likely being destroyed.
    AssetNotLive = 13,
    /// The asset status is not the expected status.
    /// Unknown error
    RuntimeError = 99,
}


impl From<DispatchError> for Outcome {
    fn from(input: DispatchError) -> Self {
        let error_text = match input {
            DispatchError::Module(ModuleError { message, .. }) => message,
            _ => Some("No module error Info"),
        };
        return match error_text {
            Some("BalanceLow") => Outcome::BalanceLow,
            Some("NoAccount") => Outcome::NoAccount,
            Some("NoPermission") => Outcome::NoPermission,
            Some("Unknown") => Outcome::Unknown,
            Some("InUse") => Outcome::InUse,
            Some("BadWitness") => Outcome::BadWitness,
            Some("MinBalanceZero") => Outcome::MinBalanceZero,
            Some("BadMetadata") => Outcome::BadMetadata,
            Some("Unapproved") => Outcome::Unapproved,
            Some("WouldDie") => Outcome::WouldDie,
            Some("AlreadyExists") => Outcome::AlreadyExists,
            Some("WouldBurn") => Outcome::WouldBurn,
            Some("AssetNotLive") => Outcome::AssetNotLive,
            _ => Outcome::RuntimeError,
        };
    }
}

pub struct AssetsExtension<T>(PhantomData<T>);

impl<T> Default for AssetsExtension<T> {
    fn default() -> Self {
        AssetsExtension(PhantomData)
    }
}

impl<T> ChainExtension<T> for AssetsExtension<T>
    where
        T: pallet_dao_assets::Config + pallet_contracts::Config,
        <T as pallet_dao_assets::Config>::AssetId: Copy,
        <<T as SysConfig>::Lookup as StaticLookup>::Source: From<<T as SysConfig>::AccountId>,
        <T as SysConfig>::AccountId: From<[u8; 32]>,
{
    fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
        where
            E: Ext<T=T>,
    {
        let func_id = env.func_id().try_into()?;
        let mut env = env.buf_in_buf_out();

        return match func_id {
            AssetsFunc::Transfer => {
                let (origin, id, target, amount): (
                    T::AccountId,
                    <T as pallet_dao_assets::Config>::AssetId,
                    T::AccountId,
                    T::Balance,
                ) = env.read_as()?;

                let base_weight = <T as pallet_dao_assets::Config>::WeightInfo::transfer();
                env.charge_weight(base_weight)?;


                let raw_origin = RawOrigin::Signed(origin);
                let call_result = pallet_dao_assets::Pallet::<T>::transfer(
                    raw_origin.into(),
                    id.into(),
                    target.into(),
                    amount,
                );
                match call_result {
                    Err(e) => {
                        let mapped_error = Outcome::from(e);
                        Ok(RetVal::Converging(mapped_error as u32))
                    }
                    Ok(_) => Ok(RetVal::Converging(Outcome::Success as u32)),
                }
            }
        }
    }

    fn enabled() -> bool {
        true
    }
}