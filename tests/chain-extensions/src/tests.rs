use crate::mock::*;
use frame_support::assert_ok;
use frame_support::pallet_prelude::DispatchError;
use frame_support::weights::Weight;
use frame_support::sp_io::hashing::blake2_256;
use pallet_contracts::{CollectEvents, DebugInfo, Determinism};

use codec::{Decode, Encode};
use pallet_contracts_primitives::Code;


fn selector_from_str(label: &str) -> Vec<u8> {
    let hash = blake2_256(label.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]].to_vec()
}

fn install(signer: AccountId, contract_path: &str, data: Vec<u8>) -> Result<AccountId, DispatchError>
{
    let contract_instantiate_result = Contracts::bare_instantiate(
        signer,
        0_u32.into(),
        Weight::MAX,
        Some(100_u32.into()),
        Code::Upload(std::fs::read(contract_path).unwrap()),
        data,
        vec![],
        pallet_contracts::DebugInfo::UnsafeDebug,
        pallet_contracts::CollectEvents::UnsafeCollect,
    );
    Ok(contract_instantiate_result.result?.account_id)
}

fn call<R>(
    signer: AccountId,
    contract_address: AccountId,
    data: Vec<u8>,
) -> Result<R, DispatchError> where R: Decode {
    let call_result = Contracts::bare_call(
        signer,
        contract_address,
        0_u32.into(),
        Weight::MAX,
        Some(100_u32.into()),
        data,
        DebugInfo::Skip,
        CollectEvents::Skip,
        Determinism::Enforced,
    ).result?.data;
    <Result<R, DispatchError>>::decode(&mut &call_result[..])
            .map_err(|_| DispatchError::Other("decoding error"))
            .unwrap()
}

#[test]
fn test_transfer_extension() {
    new_test_ext().execute_with(|| {
        let sender = ALICE;
        let origin = RuntimeOrigin::signed(sender.clone());

        let dao_id: Vec<u8> = b"GDAO".to_vec();
        let dao_name = b"Genesis DAO".to_vec();
        assert_ok!(DaoCore::create_dao(origin.clone().into(), dao_id.clone(), dao_name));
        assert_ok!(DaoCore::issue_token(origin.clone().into(), dao_id.clone(), 1000_u32.into()));
        let dao = DaoCore::load_dao(dao_id).unwrap();
        let asset_id = dao.asset_id.unwrap();

        assert_eq!(Assets::balance(asset_id.clone(), ALICE), 1000);
        assert_eq!(Assets::balance(asset_id.clone(), BOB), 0);

        let mut data = selector_from_str("new");
        data.append(&mut asset_id.clone().encode());

        let contract_path = "../../contracts/extensions/dao-assets-contract/target/ink/dao_assets_contract.wasm";
        let contract_address = install(
            sender.clone(),
            contract_path,
            data,
        ).expect("code deployed");

        let mut data = selector_from_str("transfer");
        data.append(&mut BOB.clone().encode());
        data.append(&mut 100_u128.encode());

        call::<()>(
            sender.clone(),
            contract_address,
            data,
        ).expect("call success");

        assert_eq!(Assets::balance(asset_id.clone(), ALICE), 900);
        assert_eq!(Assets::balance(asset_id.clone(), BOB), 100);
    });
}
