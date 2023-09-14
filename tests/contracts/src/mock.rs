use codec::{Decode, Encode};
use frame_support::{
	assert_ok,
	pallet_prelude::DispatchError,
	parameter_types,
	sp_io::hashing::blake2_256,
	traits::{AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU32, ConstU64, ConstU8},
	weights::Weight,
};
use frame_system as system;
use frame_system::mocking::MockUncheckedExtrinsic;

use pallet_contracts::{CollectEvents, DebugInfo, Determinism};
use pallet_contracts_primitives::{Code, ReturnFlags};
use sp_core::H256;

use commons::traits::pallets::ActiveProposalsMock;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

// type Block = frame_system::mocking::MockBlock<Test>;

pub type Block = generic::Block<generic::Header<u32, BlakeTwo256>, MockUncheckedExtrinsic<Test>>;

pub(crate) type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;
// Account ID
pub type AccountId = AccountId32;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system,
		Timestamp: pallet_timestamp,
		Balances: pallet_balances,
		Contracts: pallet_contracts,
		Assets: pallet_dao_assets,
		DaoCore: pallet_dao_core,
		HookPoints: pallet_hookpoints,
		DaoVotes: pallet_dao_votes,
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type Block = Block;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeCall = RuntimeCall;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Nonce = Nonce;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type BlockHashCount = ConstU32<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<2>;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<0>;
	type WeightInfo = ();
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

fn schedule<T: pallet_contracts::Config>() -> pallet_contracts::Schedule<T> {
	pallet_contracts::Schedule {
		limits: pallet_contracts::Limits {
			runtime_memory: 1024 * 1024 * 1024,
			..Default::default()
		},
		..Default::default()
	}
}

parameter_types! {
	pub const DepositPerItem: Balance = 0;
	pub const DepositPerByte: Balance = 0;
	pub Schedule: pallet_contracts::Schedule<Test> = schedule::<Test>();
	pub const DefaultDepositLimit: Balance = 0;
}

// Pallet contracts promises to never use this, but needs this type anyway
// Therefore we provide it, but panic when called.
pub struct FakeRandom;
impl<Output, BlockNumber> frame_support::traits::Randomness<Output, BlockNumber> for FakeRandom {
	fn random(_: &[u8]) -> (Output, BlockNumber) {
		panic!("Pallet contracts promised not to call me");
	}

	fn random_seed() -> (Output, BlockNumber) {
		panic!("Pallet contracts promised not to call me");
	}
}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = FakeRandom;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// The safest default is to allow no calls at all.
	///
	/// Runtimes should whitelist dispatchables that are allowed to be called from contracts
	/// and make sure they are stable. Dispatchables exposed to contracts are not allowed to
	/// change because that would break already deployed contracts. The `RuntimeCall` structure
	/// itself is not allowed to change the indices of existing pallets, too.
	type CallFilter = frame_support::traits::Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type CallStack = [pallet_contracts::Frame<Self>; 31];
	type WeightPrice = ();
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = pallet_dao_assets_extensions::AssetsExtension<Self>;
	type Schedule = Schedule;
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 128 * 1024 }>;
	type DefaultDepositLimit = DefaultDepositLimit;
	type MaxStorageKeyLen = ConstU32<128>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type UnsafeUnstableInterface = ConstBool<false>;
	type Migrations = ();
}

parameter_types! {
	pub const ApprovalDeposit: Balance = 1;
	pub const AssetsStringLimit: u32 = 50;
}

impl pallet_dao_assets::Config for Test {
	type ActiveProposals = ActiveProposalsMock<Self>;
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetIdParameter = u32;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<Self::AccountId>>;
	type ApprovalDeposit = ApprovalDeposit;
	type RemoveItemsLimit = ConstU32<1000>;
	type StringLimit = AssetsStringLimit;
	type HistoryHorizon = ConstU32<4200>;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_dao_core::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MinLength = ConstU32<3>;
	type MaxLengthId = ConstU32<8>;
	type MaxLengthName = ConstU32<16>;
	type MaxLengthMetadata = ConstU32<256>;
	type Currency = Balances;
	type DaoDeposit = ConstU128<10>;
	type TokenUnits = ConstU8<10>;
	type AssetId = u32;
	type ExposeAsset = Assets;
	type CoreWeightInfo = ();
}

impl pallet_hookpoints::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MaxLengthId = ConstU32<64>;
	type WeightInfo = ();
}

impl pallet_dao_votes::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ProposalDeposit = ConstU128<10>;
	type ProposalId = u32;
    type MaxProposals = ConstU32<25>;
	type WeightInfo = ();
}

// Helper functions and constants used in tests

pub const ALICE: AccountId32 = AccountId32::new([1u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([2u8; 32]);
pub const CHARLIE: AccountId32 = AccountId32::new([3u8; 32]);
pub const ASSET_CONTRACT_PATH: &str = "wasm/test_dao_assets_contract.wasm";
pub const VESTING_WALLET_CONTRACT_PATH: &str = "wasm/test_vesting_wallet_contract.wasm";
pub const VOTE_ESCROW_CONTRACT_PATH: &str = "wasm/test_vote_escrow_contract.wasm";
pub const DAO_CONTRACT_PATH: &str = "wasm/test_genesis_dao_contract.wasm";

pub fn create_dao() -> Vec<u8> {
	let origin = RuntimeOrigin::signed(ALICE);
	let dao_id: Vec<u8> = b"GDAO".to_vec();
	let dao_name = b"Genesis DAO".to_vec();
	assert_ok!(DaoCore::create_dao(origin.clone().into(), dao_id.clone(), dao_name));
	assert_ok!(DaoCore::issue_token(origin.clone().into(), dao_id.clone(), 1000_u32.into()));
	dao_id
}

pub fn create_assets_contract() -> AccountId {
	let dao = DaoCore::load_dao(create_dao()).unwrap();
	let asset_id = dao.asset_id.unwrap();

	let mut data = selector_from_str("new");
	data.append(&mut asset_id.clone().encode());
	install(ALICE, ASSET_CONTRACT_PATH, data).expect("code deployed")
}

pub fn create_vesting_wallet_contract() -> (AccountId, AccountId) {
	let asset_contract = create_assets_contract();
	let mut data = selector_from_str("new");
	data.append(&mut asset_contract.clone().encode());
	(install(ALICE, VESTING_WALLET_CONTRACT_PATH, data).expect("code deployed"), asset_contract)
}

pub fn create_vote_escrow_contract() -> (AccountId, AccountId) {
	let asset_contract = create_assets_contract();
	let mut data = selector_from_str("new");
	data.append(&mut asset_contract.clone().encode());
	data.append(&mut 1000_u32.encode());
	data.append(&mut 4_u8.encode());
	(install(ALICE, VOTE_ESCROW_CONTRACT_PATH, data).expect("code deployed"), asset_contract)
}

pub fn get_asset_id_from_contract(contract: AccountId) -> u32 {
	let account_id = call::<AccountId>(ALICE, contract.clone(), selector_from_str("get_token"))
		.expect("call success");

	call::<u32>(ALICE, account_id.clone(), selector_from_str("get_asset_id")).expect("call success")
}

pub fn install(
	signer: AccountId,
	contract_path: &str,
	data: Vec<u8>,
) -> Result<AccountId, DispatchError> {
	let contract_instantiate_result = Contracts::bare_instantiate(
		signer,
		0_u32.into(),
		Weight::MAX,
		Some(100_u32.into()),
		Code::Upload(std::fs::read(contract_path).unwrap()),
		data,
		vec![],
		DebugInfo::Skip,
		CollectEvents::Skip,
	);
	Ok(contract_instantiate_result.result?.account_id)
}

pub fn call<R>(
	signer: AccountId,
	contract_address: AccountId,
	data: Vec<u8>,
) -> Result<R, DispatchError>
where
	R: Decode,
{
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
	)
	.result
	.unwrap();

	match call_result.flags {
		ReturnFlags::REVERT => Err(DispatchError::Other("failed")),
		_ => <Result<R, DispatchError>>::decode(&mut &call_result.data[..])
			.map_err(|_| DispatchError::Other("decoding error"))
			.unwrap(),
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, 1_000_000_000_000),
			(BOB, 1_000_000_000_000),
			(CHARLIE, 1_000_000_000_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn selector_from_str(label: &str) -> Vec<u8> {
	let hash = blake2_256(label.as_bytes());
	[hash[0], hash[1], hash[2], hash[3]].to_vec()
}

pub fn forward_by_blocks(n: u32) {
	use frame_support::traits::{OnFinalize, OnInitialize};
	let current = System::block_number();
	let target = current + n;
	while System::block_number() < target {
		let mut block = System::block_number();
		Assets::on_finalize(block);
		System::on_finalize(block);
		System::reset_events();

		block += 1_u32;

		System::set_block_number(block);
		System::on_initialize(block);
		Assets::on_initialize(block);
	}
}
