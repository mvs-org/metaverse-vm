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

//! Mock file for ethereum-relay.

// --- substrate ---
use frame_support::{
	impl_outer_dispatch, impl_outer_origin, parameter_types, traits::OnInitialize,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, RuntimeDebug};
// --- hyperspace ---
use crate::*;

pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u128;

pub type System = frame_system::Module<Test>;
pub type EthereumRelay = Module<Test>;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		frame_system::System,
		hyperspace_ethereum_relay::EthereumRelay,
	}
}

hyperspace_support::impl_test_account_data! { deprecated }

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;
pub struct UnusedTechnicalMembership;
impl Contains<AccountId> for UnusedTechnicalMembership {
	fn sorted_members() -> Vec<AccountId> {
		vec![1, 2, 3]
	}
}
parameter_types! {
	pub const EthereumRelayModuleId: ModuleId = ModuleId(*b"da/ethrl");
	pub const EthereumNetwork: EthereumNetworkType = EthereumNetworkType::Mainnet;
	pub static BestConfirmedBlockNumber: EthereumBlockNumber = 0;
	pub static ConfirmPeriod: BlockNumber = 0;
}
impl Config for Test {
	type ModuleId = EthereumRelayModuleId;
	type Event = ();
	type EthereumNetwork = EthereumNetwork;
	type Call = Call;
	type Currency = Etp;
	type RelayerGame = UnusedRelayerGame;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type ConfirmPeriod = ConfirmPeriod;
	type TechnicalMembership = UnusedTechnicalMembership;
	type ApproveThreshold = ();
	type RejectThreshold = ();
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl hyperspace_balances::Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ();
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
}

pub struct ExtBuilder {
	best_confirmed_block_number: EthereumBlockNumber,
	confirm_period: BlockNumber,
}
impl ExtBuilder {
	pub fn best_confirmed_block_number(
		mut self,
		best_confirmed_block_number: EthereumBlockNumber,
	) -> Self {
		self.best_confirmed_block_number = best_confirmed_block_number;

		self
	}

	pub fn confirm_period(mut self, confirm_period: BlockNumber) -> Self {
		self.confirm_period = confirm_period;

		self
	}

	pub fn set_associated_constants(&self) {
		BEST_CONFIRMED_BLOCK_NUMBER.with(|v| v.replace(self.best_confirmed_block_number));
		CONFIRM_PERIOD.with(|v| v.replace(self.confirm_period));
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_constants();

		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		GenesisConfig::<Test> {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 17, 187, 232, 219, 78, 52, 123, 78, 140, 147, 124, 28, 131, 112, 228, 181, 237, 51, 173, 179, 219, 105, 203, 219, 122, 56, 225, 229, 11, 27, 130, 250, 215, 248, 151, 79, 181, 172, 120, 217, 172, 9, 155, 154, 213, 1, 139, 237, 194, 206, 10, 114, 218, 209, 130, 122, 23, 9, 218, 48, 88, 15, 5, 68, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 136, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 212, 229, 103, 64, 248, 118, 174, 248, 192, 16, 184, 106, 64, 213, 245, 103, 69, 161, 24, 208, 144, 106, 52, 230, 154, 236, 140, 13, 177, 203, 143, 163],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoader::from_file(
				"../../../../bin/res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		storage.into()
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			best_confirmed_block_number: BEST_CONFIRMED_BLOCK_NUMBER.with(|v| *v.borrow()),
			confirm_period: CONFIRM_PERIOD.with(|v| *v.borrow()),
		}
	}
}

pub struct UnusedRelayerGame;
impl RelayerGameProtocol for UnusedRelayerGame {
	type Relayer = AccountId;
	type RelayHeaderId = EthereumBlockNumber;
	type RelayHeaderParcel = EthereumRelayHeaderParcel;
	type RelayProofs = EthereumRelayProofs;

	fn get_proposed_relay_header_parcels(
		_: &RelayAffirmationId<Self::RelayHeaderId>,
	) -> Option<Vec<Self::RelayHeaderParcel>> {
		unimplemented!()
	}
	fn best_confirmed_header_id_of(_: &Self::RelayHeaderId) -> Self::RelayHeaderId {
		BEST_CONFIRMED_BLOCK_NUMBER.with(|v| *v.borrow())
	}
	fn affirm(
		_: &Self::Relayer,
		_: Self::RelayHeaderParcel,
		_: Option<Self::RelayProofs>,
	) -> Result<Self::RelayHeaderId, DispatchError> {
		unimplemented!()
	}
	fn dispute_and_affirm(
		_: &Self::Relayer,
		_: Self::RelayHeaderParcel,
		_: Option<Self::RelayProofs>,
	) -> Result<(Self::RelayHeaderId, u32), DispatchError> {
		unimplemented!()
	}
	fn complete_relay_proofs(
		_: RelayAffirmationId<Self::RelayHeaderId>,
		_: Vec<Self::RelayProofs>,
	) -> DispatchResult {
		unimplemented!()
	}
	fn extend_affirmation(
		_: &Self::Relayer,
		_: RelayAffirmationId<Self::RelayHeaderId>,
		_: Vec<Self::RelayHeaderParcel>,
		_: Option<Vec<Self::RelayProofs>>,
	) -> Result<(Self::RelayHeaderId, u32, u32), DispatchError> {
		unimplemented!()
	}
}

pub fn run_to_block(n: BlockNumber) {
	// EthereumRelay::on_finalize(System::block_number());

	for b in System::block_number() + 1..=n {
		System::set_block_number(b);
		EthereumRelay::on_initialize(b);

		// if b != n {
		// 	EthereumRelay::on_finalize(System::block_number());
		// }
	}
}
