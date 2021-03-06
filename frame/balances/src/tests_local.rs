// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Hyperspace Network
// SPDX-License-Identifier: GPL-3.0
//
// Hyperspace is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hyperspace is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

//! Test utilities

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{
	assert_ok, parameter_types,
	traits::StorageMapShim,
	weights::{DispatchInfo, IdentityFee, Weight},
};
use frame_system::{mocking::*, RawOrigin};
use pallet_transaction_payment::CurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeDebug,
};
// --- hyperspace ---
use crate::{self as hyperspace_balances, *};

type Balance = u64;

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

hyperspace_support::impl_test_account_data! {}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Balance;
	type BlockNumber = Balance;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = Balance;
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

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}
impl pallet_transaction_payment::Config for Test {
	type OnChargeTransaction = CurrencyAdapter<Etp, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<u64>;
	type FeeMultiplierUpdate = ();
}

parameter_types! {
	pub static ExistentialDeposit: u64 = 0;
}
impl Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = StorageMapShim<
		Account<Test, EtpInstance>,
		frame_system::Provider<Test>,
		Balance,
		AccountData<Balance>,
	>;
	type MaxLocks = ();
	type OtherCurrencies = (Dna,);
	type WeightInfo = ();
}
impl Config<DnaInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = StorageMapShim<
		Account<Test, DnaInstance>,
		frame_system::Provider<Test>,
		Balance,
		AccountData<Balance>,
	>;
	type MaxLocks = ();
	type OtherCurrencies = (Etp,);
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		Etp: hyperspace_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		Dna: hyperspace_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},
	}
}

pub struct ExtBuilder {
	existential_deposit: Balance,
	monied: bool,
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: 1,
			monied: false,
		}
	}
}
impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: Balance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn monied(mut self, monied: bool) -> Self {
		self.monied = monied;
		if self.existential_deposit == 0 {
			self.existential_deposit = 1;
		}
		self
	}
	pub fn set_associated_constants(&self) {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
	}
	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_constants();
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		hyperspace_balances::GenesisConfig::<Test, EtpInstance> {
			balances: if self.monied {
				vec![
					(1, 10 * self.existential_deposit),
					(2, 20 * self.existential_deposit),
					(3, 30 * self.existential_deposit),
					(4, 40 * self.existential_deposit),
					(12, 10 * self.existential_deposit),
				]
			} else {
				vec![]
			},
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

decl_tests! { Test, ExtBuilder, EXISTENTIAL_DEPOSIT }

#[test]
fn emit_events_with_no_existential_deposit_suicide_with_dust() {
	<ExtBuilder>::default()
		.existential_deposit(2)
		.build()
		.execute_with(|| {
			assert_ok!(Etp::set_balance(RawOrigin::Root.into(), 1, 100, 0));

			assert_eq!(
				events(),
				[
					Event::frame_system(frame_system::Event::NewAccount(1)),
					Event::hyperspace_balances_Instance0(RawEvent::Endowed(1, 100)),
					Event::hyperspace_balances_Instance0(RawEvent::BalanceSet(1, 100, 0)),
				]
			);

			let _ = Etp::slash(&1, 98);

			// no events
			assert_eq!(events(), []);

			let _ = Etp::slash(&1, 1);

			assert_eq!(
				events(),
				[
					Event::hyperspace_balances_Instance0(RawEvent::DustLost(1, 1)),
					Event::frame_system(frame_system::Event::KilledAccount(1))
				]
			);
		});
}

#[test]
fn dust_collector_should_work() {
	type AnotherBalance = Module<Test, Instance1>;

	<ExtBuilder>::default()
		.existential_deposit(100)
		.build()
		.execute_with(|| {
			assert_ok!(Etp::set_balance(RawOrigin::Root.into(), 1, 100, 0));

			assert_eq!(
				events(),
				[
					Event::frame_system(frame_system::Event::NewAccount(1)),
					Event::hyperspace_balances_Instance0(RawEvent::Endowed(1, 100)),
					Event::hyperspace_balances_Instance0(RawEvent::BalanceSet(1, 100, 0)),
				]
			);

			let _ = Etp::slash(&1, 1);

			assert_eq!(
				events(),
				[
					Event::hyperspace_balances_Instance0(RawEvent::DustLost(1, 99)),
					Event::frame_system(frame_system::Event::KilledAccount(1))
				]
			);

			// ---

			assert_ok!(Etp::set_balance(RawOrigin::Root.into(), 1, 100, 0));
			assert_ok!(AnotherBalance::set_balance(
				RawOrigin::Root.into(),
				1,
				100,
				0
			));

			assert_eq!(
				events(),
				[
					Event::frame_system(frame_system::Event::NewAccount(1)),
					Event::hyperspace_balances_Instance0(RawEvent::Endowed(1, 100)),
					Event::hyperspace_balances_Instance0(RawEvent::BalanceSet(1, 100, 0)),
					Event::hyperspace_balances_Instance1(RawEvent::Endowed(1, 100)),
					Event::hyperspace_balances_Instance1(RawEvent::BalanceSet(1, 100, 0)),
				]
			);

			let _ = Etp::slash(&1, 1);

			assert_eq!(events(), []);

			let _ = AnotherBalance::slash(&1, 1);

			assert_eq!(
				events(),
				[
					Event::hyperspace_balances_Instance0(RawEvent::DustLost(1, 99)),
					Event::hyperspace_balances_Instance1(RawEvent::DustLost(1, 99)),
					Event::frame_system(frame_system::Event::KilledAccount(1)),
				]
			);
		});
}
