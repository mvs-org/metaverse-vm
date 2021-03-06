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

//! Mock file for treasury.

mod treasury {
	// --- hyperspace ---
	// Re-export needed for `impl_outer_event!`.
	pub use super::super::*;
}

// --- substrate ---
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	ModuleId,
};
// --- hyperspace ---
use crate::*;

type Balance = u64;

pub type System = frame_system::Module<Test>;
pub type Treasury = Module<Test>;

impl_outer_event! {
	pub enum Event for Test {
		frame_system <T>,
		hyperspace_balances Instance0<T>,
		hyperspace_balances Instance1<T>,
		treasury <T>,
	}
}

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

hyperspace_support::impl_test_account_data! { deprecated }

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
	pub static TenToFourteen: Vec<u128> = vec![10, 11, 12, 13, 14];
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
	type AccountId = u128;
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

pub struct Tippers;
impl Contains<u128> for Tippers {
	fn sorted_members() -> Vec<u128> {
		TEN_TO_FOURTEEN.with(|v| v.borrow().clone())
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(new: &u128) {
		TEN_TO_FOURTEEN.with(|v| {
			let mut members = v.borrow_mut();
			members.push(*new);
			members.sort();
		})
	}
}
impl ContainsLengthBound for Tippers {
	fn min_len() -> usize {
		0
	}
	fn max_len() -> usize {
		TEN_TO_FOURTEEN.with(|v| v.borrow().len())
	}
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
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

parameter_types! {
	pub const TreasuryModuleId: ModuleId = ModuleId(*b"da/trsry");
	pub const TipCountdown: u64 = 1;
	pub const TipFindersFee: Percent = Percent::from_percent(20);
	pub const TipReportDepositBase: Balance = 1;
	pub const DataDepositPerByte: Balance = 1;
	pub const BountyDepositBase: u64 = 80;
	pub const BountyDepositPayoutDelay: u64 = 3;
	pub const BountyUpdatePeriod: u32 = 20;
	pub const MaximumReasonLength: u32 = 16384;
	pub const BountyCuratorDeposit: Permill = Permill::from_percent(50);
	pub const BountyValueMinimum: u64 = 1;
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const EtpProposalBondMinimum: Balance = 1;
	pub const DnaProposalBondMinimum: Balance = 1;
	pub const SpendPeriod: u64 = 2;
	pub const Burn: Permill = Permill::from_percent(50);
}
impl Config for Test {
	type ModuleId = TreasuryModuleId;
	type EtpCurrency = Etp;
	type DnaCurrency = Dna;
	type ApproveOrigin = frame_system::EnsureRoot<u128>;
	type RejectOrigin = frame_system::EnsureRoot<u128>;
	type Tippers = Tippers;
	type TipCountdown = TipCountdown;
	type TipFindersFee = TipFindersFee;
	type TipReportDepositBase = TipReportDepositBase;
	type DataDepositPerByte = DataDepositPerByte;
	type Event = Event;
	type OnSlashEtp = ();
	type OnSlashDna = ();
	type ProposalBond = ProposalBond;
	type EtpProposalBondMinimum = EtpProposalBondMinimum;
	type DnaProposalBondMinimum = DnaProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BountyDepositBase = BountyDepositBase;
	type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
	type BountyUpdatePeriod = BountyUpdatePeriod;
	type BountyCuratorDeposit = BountyCuratorDeposit;
	type BountyValueMinimum = BountyValueMinimum;
	type MaximumReasonLength = MaximumReasonLength;
	type EtpBurnDestination = (); // Just gets burned.
	type DnaBurnDestination = (); // Just gets burned.
	type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	EtpConfig {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	DnaConfig {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisConfig::default()
		.assimilate_storage::<Test, _>(&mut t)
		.unwrap();

	t.into()
}

pub fn last_event() -> RawEvent<u128, H256, u64, u64, DefaultInstance> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::treasury(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.last()
		.unwrap()
}

pub fn tip_hash() -> H256 {
	BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.hyperspace"), 3u128))
}
