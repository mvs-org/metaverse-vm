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

#![allow(dead_code)]

pub mod oldna_issuing {
	// --- hyperspace ---
	pub use crate::Event;
}

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::{Header, H256},
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeDebug,
};
// --- hyperspace ---
use crate::*;

pub type AccountId = u64;
pub type Balance = u128;

pub type System = frame_system::Module<Test>;
pub type OldnaIssuing = Module<Test>;

pub type OldnaIssuingError = Error<Test>;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

impl_outer_event! {
	pub enum Event for Test {
		frame_system <T>,
		hyperspace_balances Instance0<T>,
		oldna_issuing <T>,
	}
}

hyperspace_support::impl_test_account_data! { deprecated }

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const OldnaIssuingModuleId: ModuleId = ModuleId(*b"da/oldnai");
}
impl Config for Test {
	type Event = Event;
	type ModuleId = OldnaIssuingModuleId;
	type EtpCurrency = Etp;
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
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

pub fn new_test_ext() -> TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	EtpConfig {
		balances: (1..10)
			.map(|i: AccountId| vec![(i, 100 * i as Balance), (10 * i, 1000 * i as Balance)])
			.flatten()
			.collect(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisConfig {
		total_mapped_etp: 4_000,
	}
	.assimilate_storage::<Test>(&mut t)
	.unwrap();

	t.into()
}

pub fn events() -> Vec<Event> {
	let events = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();

	System::reset_events();

	events
}

pub fn oldna_issuing_events() -> Vec<Event> {
	events()
		.into_iter()
		.filter(|e| matches!(e, Event::oldna_issuing(_)))
		.collect()
}
