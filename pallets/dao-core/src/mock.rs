use crate as pallet_dao_core;
use crate::*;
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64, ConstU8},
};
use sp_core::H256;

use commons::traits::ActiveProposalsMock;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

pub(crate) type Balance = u128;
/// Index of a transaction in the chain.
pub type Nonce = u32;
// Account ID
pub type AccountId = u64;

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system,
		Balances: pallet_balances,
		DaoCore: pallet_dao_core,
		Assets: pallet_dao_assets,
		CoreX: pallet_dao_assets::dao_core,
	}
);

impl frame_system::Config for Test {
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
	type BlockHashCount = ConstU64<250>;
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

parameter_types! {
	// we're not really using this, as reservation is via DAO, but whatever
	pub const ApprovalDeposit: Balance = 1;
	pub const AssetsStringLimit: u32 = 50;
}

impl pallet_dao_assets::dao_core::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = u32;
	type ExposeAsset = Assets;
	type CoreWeightInfo = ();
	type DaoDeposit = ConstU128<10>;
	type MinLength = ConstU32<3>;
	type MaxLengthId = ConstU32<8>;
	type MaxLengthName = ConstU32<16>;
	type MaxLengthMetadata = ConstU32<256>;
	type TokenUnits = ConstU8<10>;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type AssetId = u32;
	type ExposeAsset = Assets;
	type CoreWeightInfo = ();
	type DaoDeposit = ConstU128<10>;
	type MinLength = ConstU32<3>;
	type MaxLengthId = ConstU32<8>;
	type MaxLengthName = ConstU32<16>;
	type MaxLengthMetadata = ConstU32<256>;
	type TokenUnits = ConstU8<10>;
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

// Build genesis storage according to the mock runtime.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(1, 1000)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
