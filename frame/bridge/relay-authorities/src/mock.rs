// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Metaverse
// SPDX-License-Identifier: GPL-3.0
//
// Hyperspace is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hyperspace is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

//! # Mock file for relay authorities

pub mod relay_authorities {
	// --- hyperspace ---
	pub use crate::Event;
}

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types, traits::OnInitialize, weights::Weight,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_io::{hashing, TestExternalities};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill, RuntimeDebug,
};
// --- hyperspace ---
use crate::*;
use hyperspace_relay_primitives::relay_authorities::Sign as SignT;

pub type BlockNumber = u64;
pub type AccountId = u64;
pub type Index = u64;
pub type Balance = u128;

pub type System = frame_system::Module<Test>;
pub type Etp = hyperspace_balances::Module<Test, EtpInstance>;
pub type RelayAuthorities = Module<Test>;

pub type RelayAuthoritiesError = Error<Test, DefaultInstance>;

pub const DEFAULT_MMR_ROOT: H256 = H256([0; 32]);
pub const DEFAULT_SIGNATURE: [u8; 65] = [0; 65];

impl_outer_origin! {
	pub enum Origin for Test {}
}

impl_outer_event! {
	pub enum Event for Test {
		frame_system <T>,
		hyperspace_balances Instance0<T>,
		relay_authorities <T>,
	}
}

hyperspace_support::impl_test_account_data! {}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
pub struct HyperspaceMMR;
impl MMR<BlockNumber, H256> for HyperspaceMMR {
	fn get_root(_: BlockNumber) -> Option<H256> {
		Some(Default::default())
	}
}
pub struct Sign;
impl SignT<BlockNumber> for Sign {
	type Signature = [u8; 65];
	type Message = [u8; 32];
	type Signer = [u8; 20];

	fn hash(raw_message: impl AsRef<[u8]>) -> Self::Message {
		hashing::blake2_256(raw_message.as_ref())
	}

	fn verify_signature(_: &Self::Signature, _: &Self::Message, _: &Self::Signer) -> bool {
		true
	}
}
parameter_types! {
	pub const LockId: LockIdentifier = *b"lockidts";
	pub const TermDuration: BlockNumber = 10;
	pub const MaxCandidates: usize = 7;
	pub const SignThreshold: Perbill = Perbill::from_percent(60);
	pub const SubmitDuration: BlockNumber = 3;
}
impl Trait for Test {
	type Event = Event;
	type EtpCurrency = Etp;
	type LockId = LockId;
	type TermDuration = TermDuration;
	type MaxCandidates = MaxCandidates;
	type AddOrigin = EnsureRoot<Self::AccountId>;
	type RemoveOrigin = EnsureRoot<Self::AccountId>;
	type ResetOrigin = EnsureRoot<Self::AccountId>;
	type HyperspaceMMR = HyperspaceMMR;
	type Sign = Sign;
	type OpCodes = ();
	type SignThreshold = SignThreshold;
	type SubmitDuration = SubmitDuration;
	type WeightInfo = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}
impl frame_system::Trait for Test {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 1024;
}
impl hyperspace_balances::Trait<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ();
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type OtherCurrencies = ();
	type WeightInfo = ();
}

pub fn new_test_ext() -> TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	hyperspace_balances::GenesisConfig::<Test, EtpInstance> {
		balances: (1..10)
			.map(|i: AccountId| vec![(i, 100 * i as Balance), (10 * i, 1000 * i as Balance)])
			.flatten()
			.collect(),
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	GenesisConfig::<Test> {
		authorities: vec![(9, signer_of(9), 1)],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	storage.into()
}

pub fn run_to_block(n: BlockNumber) {
	for b in System::block_number() + 1..=n {
		System::set_block_number(b);
		RelayAuthorities::on_initialize(b);
	}
}

pub fn events() -> Vec<Event> {
	let events = System::events()
		.into_iter()
		.map(|evt| evt.event)
		.collect::<Vec<_>>();

	System::reset_events();

	events
}

pub fn relay_authorities_events() -> Vec<Event> {
	events()
		.into_iter()
		.filter(|e| matches!(e, Event::relay_authorities(_)))
		.collect()
}

pub fn request_authority(account_id: AccountId) -> DispatchResult {
	RelayAuthorities::request_authority(Origin::signed(account_id), 1, signer_of(account_id))
}

pub fn request_authority_with_stake(account_id: AccountId, stake: Balance) -> DispatchResult {
	RelayAuthorities::request_authority(Origin::signed(account_id), stake, signer_of(account_id))
}

pub fn signer_of(account_id: AccountId) -> [u8; 20] {
	[account_id as _; 20]
}
