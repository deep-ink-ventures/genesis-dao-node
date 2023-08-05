use codec::{Decode, Encode};
use frame_support::pallet_prelude::DispatchError;
use pallet_dao_assets::AssetBalanceOf;
use crate::Config;
use pallet_hookpoints::Pallet as HookPoints;

pub fn on_vote_callback<T: Config>(dao_owner: T::AccountId, voter: T::AccountId, balance: T::Balance) -> T::Balance {
	// the selector for "GenesisDAO::calculate_voting_power"
	let mut data = 0xa68e4cba_u32.to_be_bytes().to_vec();
	data.append(&mut voter.encode());
	data.append(&mut balance.encode());

	// whenever something goes wrong here we do not alter behaviour and return the original balance
	match HookPoints::<T>::exec_hookpoint(&dao_owner, voter,"ON_VOTING_CALC", data) {
		// We got a result, let's decode it
		Ok(result) =>  <Result<AssetBalanceOf<T>, DispatchError>>::decode(&mut &result.data[..])
						.map_err(|_| DispatchError::Other("decoding error")).unwrap().unwrap_or(balance),
		// we couldn't exec the hookpoint for various reasons, among them that there is no contract installed
		Err(_) =>  balance
	}
}
