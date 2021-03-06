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

//! # Mock file for relay authorities

// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{parameter_types, traits::OnInitialize};
use frame_system::{mocking::*, EnsureRoot};
use sp_core::H256;
use sp_io::{hashing, TestExternalities};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	RuntimeDebug,
};
// --- hyperspace ---
use crate::{self as hyperspace_relay_authorities, *};
use hyperspace_relay_primitives::relay_authorities::Sign as SignT;

pub type BlockNumber = u64;
pub type AccountId = u64;
pub type Index = u64;
pub type Balance = u128;

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

pub type RelayAuthoritiesError = Error<Test, DefaultInstance>;

pub const DEFAULT_MMR_ROOT: H256 = H256([0; 32]);
pub const DEFAULT_SIGNATURE: [u8; 65] = [0; 65];

hyperspace_support::impl_test_account_data! {}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = Index;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
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
	pub const MaxLocks: u32 = 1024;
}
impl hyperspace_balances::Config<EtpInstance> for Test {
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
impl Config for Test {
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

frame_support::construct_runtime! {
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		Etp: hyperspace_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		RelayAuthorities: hyperspace_relay_authorities::{Module, Call, Storage, Config<T>, Event<T>}
	}
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
	hyperspace_relay_authorities::GenesisConfig::<Test, DefaultInstance> {
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
		.filter(|e| matches!(e, Event::hyperspace_relay_authorities(_)))
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
