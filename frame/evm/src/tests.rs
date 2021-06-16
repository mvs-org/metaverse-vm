#![cfg(test)]

use crate::{self as hyperspace_evm, *};
use frame_support::{assert_ok, traits::GenesisBuild};
use frame_system::mocking::*;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeDebug,
};
use std::{collections::BTreeMap, str::FromStr};

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

type Balance = u64;

hyperspace_support::impl_test_account_data! {}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = Call;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

frame_support::parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl hyperspace_balances::Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
}
impl hyperspace_balances::Config<DnaInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

/// Fixed gas price of `0`.
pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		// Gas price is always one token per gas.
		0.into()
	}
}

/// EmptyIssuingHandler
pub struct EmptyIssuingHandler;
impl IssuingHandler for EmptyIssuingHandler {
	fn handle(_address: H160, _caller: H160, _input: &[u8]) -> DispatchResult {
		Ok(())
	}
}

pub struct RawAccountBasic<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> AccountBasic for RawAccountBasic<T> {
	/// Get the account basic in EVM format.
	fn account_basic(address: &H160) -> Account {
		let account_id = T::AddressMapping::into_account_id(*address);

		let nonce = <frame_system::Pallet<T>>::account_nonce(&account_id);
		let balance = T::EtpCurrency::free_balance(&account_id);

		Account {
			nonce: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(nonce)),
			balance: U256::from(UniqueSaturatedInto::<u128>::unique_saturated_into(balance)),
		}
	}

	fn mutate_account_basic(address: &H160, new: Account) {
		let account_id = T::AddressMapping::into_account_id(*address);
		let current = T::EtpAccountBasic::account_basic(address);

		if current.nonce < new.nonce {
			// ASSUME: in one single EVM transaction, the nonce will not increase more than
			// `u128::max_value()`.
			for _ in 0..(new.nonce - current.nonce).low_u128() {
				<frame_system::Pallet<T>>::inc_account_nonce(&account_id);
			}
		}

		if current.balance > new.balance {
			let diff = current.balance - new.balance;
			T::EtpCurrency::slash(&account_id, diff.low_u128().unique_saturated_into());
		} else if current.balance < new.balance {
			let diff = new.balance - current.balance;
			T::EtpCurrency::deposit_creating(&account_id, diff.low_u128().unique_saturated_into());
		}
	}

	fn transfer(_source: &H160, _target: &H160, _value: U256) -> Result<(), ExitError> {
		Ok(())
	}
}

/// Ensure that the origin is root.
pub struct EnsureAddressRoot<AccountId>(sp_std::marker::PhantomData<AccountId>);

impl<OuterOrigin, AccountId> EnsureAddressOrigin<OuterOrigin> for EnsureAddressRoot<AccountId>
where
	OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>> + From<RawOrigin<AccountId>>,
{
	type Success = ();

	fn try_address_origin(_address: &H160, origin: OuterOrigin) -> Result<(), OuterOrigin> {
		origin.into().and_then(|o| match o {
			RawOrigin::Root => Ok(()),
			r => Err(OuterOrigin::from(r)),
		})
	}
}

impl Config for Test {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<Self::AccountId>;

	type AddressMapping = ConcatAddressMapping;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;

	type Event = Event;
	type Precompiles = ();
	type ChainId = ();
	type BlockGasLimit = ();
	type Runner = crate::runner::stack::Runner<Self>;
	type IssuingHandler = EmptyIssuingHandler;
	type EtpAccountBasic = RawAccountBasic<Test>;
	type DnaAccountBasic = RawAccountBasic<Test>;
}

frame_support::construct_runtime! {
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		Etp: hyperspace_balances::<Instance0>::{Pallet, Call, Storage, Config<T>, Event<T>},
		Dna: hyperspace_balances::<Instance1>::{Pallet, Call, Storage, Config<T>, Event<T>},
		EVM: hyperspace_evm::{Pallet, Call, Storage, Config, Event<T>},
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	let mut accounts = BTreeMap::new();
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000001").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::from(1000000),
			storage: Default::default(),
			code: vec![
				0x00, // STOP
			],
		},
	);
	accounts.insert(
		H160::from_str("1000000000000000000000000000000000000002").unwrap(),
		GenesisAccount {
			nonce: U256::from(1),
			balance: U256::from(1000000),
			storage: Default::default(),
			code: vec![
				0xff, // INVALID
			],
		},
	);

	<hyperspace_balances::GenesisConfig<Test, EtpInstance>>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	<hyperspace_balances::GenesisConfig<Test, DnaInstance>>::default()
		.assimilate_storage(&mut t)
		.unwrap();
	hyperspace_evm::GenesisConfig { accounts }
		.assimilate_storage::<Test>(&mut t)
		.unwrap();
	t.into()
}

#[test]
fn fail_call_return_ok() {
	new_test_ext().execute_with(|| {
		assert_ok!(EVM::call(
			Origin::root(),
			H160::default(),
			H160::from_str("1000000000000000000000000000000000000001").unwrap(),
			Vec::new(),
			U256::default(),
			1000000,
			U256::default(),
			None,
		));

		assert_ok!(EVM::call(
			Origin::root(),
			H160::default(),
			H160::from_str("1000000000000000000000000000000000000002").unwrap(),
			Vec::new(),
			U256::default(),
			1000000,
			U256::default(),
			None,
		));
	});
}
