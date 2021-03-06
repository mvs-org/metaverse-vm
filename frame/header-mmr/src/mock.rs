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

#![cfg(test)]

// --- crates ---
use codec::Encode;
// --- substrate ---
use frame_system::mocking::*;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DigestItem,
};
// --- hyperspace ---
use crate::{self as hyperspace_header_mmr, *};

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl Config for Test {}

frame_support::construct_runtime! {
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config},
		HeaderMMR: hyperspace_header_mmr::{Module, Call, Storage},
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn header_mmr_log(hash: H256) -> DigestItem<H256> {
	let mmr_root_log = MerkleMountainRangeRootLog::<H256> {
		prefix: PARENT_MMR_ROOT_LOG_ID,
		parent_mmr_root: hash,
	};

	DigestItem::Other(mmr_root_log.encode())
}

pub fn initialize_block(number: u64, parent_hash: H256) {
	System::initialize(
		&number,
		&parent_hash,
		&Default::default(),
		Default::default(),
	);
}

// -- helpers ---
pub const HEADERS_N_ROOTS: [(&str, &str); 10] = [
	(
		"34f61bfda344b3fad3c3e38832a91448b3c613b199eb23e5110a635d71c13c65",
		"34f61bfda344b3fad3c3e38832a91448b3c613b199eb23e5110a635d71c13c65",
	),
	(
		"70d641860d40937920de1eae29530cdc956be830f145128ebb2b496f151c1afb",
		"3aafcc7fe12cb8fad62c261458f1c19dba0a3756647fa4e8bff6e248883938be",
	),
	(
		"12e69454d992b9b1e00ea79a7fa1227c889c84d04b7cd47e37938d6f69ece45d",
		"7ddf10d67045173e3a59efafb304495d9a7c84b84f0bc0235470a5345e32535d",
	),
	(
		"3733bd06905e128d38b9b336207f301133ba1d0a4be8eaaff6810941f0ad3b1a",
		"488e9565547fec8bd36911dc805a7ed9f3d8d1eacabe429c67c6456933c8e0a6",
	),
	(
		"3d7572be1599b488862a1b35051c3ef081ba334d1686f9957dbc2afd52bd2028",
		"6e0c4ab56e0919a7d45867fcd1216e2891e06994699eb838386189e9abda55f1",
	),
	(
		"2a04add3ecc3979741afad967dfedf807e07b136e05f9c670a274334d74892cf",
		"293b49420345b185a1180e165c76f76d8cc28fe46c1f6eb4a96959253b571ccd",
	),
	(
		"c58e247ea35c51586de2ea40ac6daf90eac7ac7b2f5c88bbc7829280db7890f1",
		"2dee5b87a481a9105cb4b2db212a1d8031d65e9e6e68dc5859bef5e0fdd934b2",
	),
	(
		"2cf0262f0a8b00cad22afa04d70fb0c1dbb2eb4a783beb7c5e27bd89015ff573",
		"54be644b5b3291dd9ae9598b49d1f986e4ebd8171d5e89561b2a921764c7b17c",
	),
	(
		"05370d06def89f11486c994c459721b4bd023ff8c2347f3187e9f42ef39bddab",
		"620dbc3a28888da8b17ebf5b18dba53794621463e2bbabcf88b8cbc97508ab38",
	),
	(
		"c0c8c3f7dc9cdfa87d2433bcd72a744d634524a5ff76e019e44ea450476bac99",
		"a94bf2a4e0437c236c68675403d980697cf7c9b0f818a622cb40199db5e12cf8",
	),
];
