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

pub mod mock_relay {
	pub mod types {
		pub type MockRelayBlockNumber = u32;
		pub type MockRelayHeaderHash = u128;
	}

	pub use types::*;

	// --- crates ---
	use serde::{Deserialize, Serialize};
	// --- substrate ---
	use sp_runtime::DispatchResult;
	// --- hyperspace ---
	use crate::{mock::*, *};

	decl_storage! {
		trait Store for Module<T: Config> as HyperspaceRelay {
			pub ConfirmedBlockNumbers get(fn best_confirmed_block_number): MockRelayBlockNumber;

			pub ConfirmedHeaders
				get(fn confirmed_header_of)
				: map hasher(identity) MockRelayBlockNumber
				=> Option<MockRelayHeader>;
		}

		add_extra_genesis {
			config(headers): Vec<MockRelayHeader>;
			build(|config: &GenesisConfig| {
				let mut best_confirmed_block_number = 0;

				ConfirmedHeaders::insert(
					best_confirmed_block_number,
					MockRelayHeader {
						number: 0,
						hash: 0,
						parent_hash: 0,
						valid: true,
					}
				);

				for header in &config.headers {
					ConfirmedHeaders::insert(header.number, header.clone());

					best_confirmed_block_number = best_confirmed_block_number.max(header.number);
				}

				ConfirmedBlockNumbers::put(best_confirmed_block_number);
			});
		}
	}

	decl_module! {
		pub struct Module<T: Config> for enum Call
		where
			origin: T::Origin
		{}
	}

	impl<T: Config> Relayable for Module<T> {
		type RelayHeaderId = MockRelayBlockNumber;
		type RelayHeaderParcel = MockRelayHeader;
		type RelayProofs = ();

		fn best_confirmed_relay_header_id() -> Self::RelayHeaderId {
			Self::best_confirmed_block_number()
		}

		fn preverify_game_sample_points(
			_: &RelayAffirmationId<Self::RelayHeaderId>,
			_: &[Self::RelayHeaderParcel],
		) -> DispatchResult {
			Ok(())
		}

		fn verify_relay_proofs(
			_: &Self::RelayHeaderId,
			relay_header_parcel: &Self::RelayHeaderParcel,
			_: &Self::RelayProofs,
			_: Option<&Self::RelayHeaderId>,
		) -> DispatchResult {
			ensure!(relay_header_parcel.valid, "Parcel - INVALID");

			Ok(())
		}

		fn verify_relay_chain(mut relay_chain: Vec<&Self::RelayHeaderParcel>) -> DispatchResult {
			let verify_continuous =
				|previous: &MockRelayHeader, next: &MockRelayHeader| -> DispatchResult {
					ensure!(previous.hash == next.parent_hash, "Continuous - INVALID");

					Ok(())
				};

			relay_chain.sort_by_key(|relay_header_parcel| relay_header_parcel.number);

			for window in relay_chain.windows(2) {
				let previous = window[0];
				let next = window[1];

				verify_continuous(previous, next)?;
			}

			verify_continuous(
				&Self::confirmed_header_of(RelayerGame::best_confirmed_header_id_of(
					&relay_chain[0].number,
				))
				.unwrap(),
				relay_chain[0],
			)?;

			Ok(())
		}

		fn distance_between(
			relay_header_id: &Self::RelayHeaderId,
			best_confirmed_relay_header_id: Self::RelayHeaderId,
		) -> u32 {
			relay_header_id - best_confirmed_relay_header_id
		}

		// FIXME
		fn try_confirm_relay_header_parcel(
			relay_header_parcel: Self::RelayHeaderParcel,
		) -> DispatchResult {
			ConfirmedBlockNumbers::mutate(|best_confirmed_block_number| {
				if relay_header_parcel.number > *best_confirmed_block_number {
					*best_confirmed_block_number = relay_header_parcel.number;

					ConfirmedHeaders::insert(relay_header_parcel.number, relay_header_parcel);
				}
			});

			Ok(())
		}

		fn new_round(_: &Self::RelayHeaderId, _: Vec<Self::RelayHeaderId>) {}

		fn game_over(_: &Self::RelayHeaderId) {}
	}

	#[derive(
		Clone, Debug, Default, PartialEq, PartialOrd, Encode, Decode, Serialize, Deserialize,
	)]
	pub struct MockRelayHeader {
		pub number: MockRelayBlockNumber,
		pub hash: MockRelayHeaderHash,
		pub parent_hash: MockRelayHeaderHash,
		pub valid: bool,
	}
	impl MockRelayHeader {
		pub fn gen(
			number: MockRelayBlockNumber,
			parent_hash: MockRelayHeaderHash,
			valid: u8,
		) -> Self {
			let valid = match valid {
				0 => false,
				_ => true,
			};

			Self {
				number,
				hash: GENESIS_TIME
					.with(|v| v.to_owned())
					.borrow()
					.elapsed()
					.as_nanos(),
				parent_hash,
				valid,
			}
		}

		pub fn gen_continous(
			start: u32,
			mut validations: Vec<u8>,
			continous_valid: bool,
		) -> Vec<Self> {
			if validations.is_empty() {
				return vec![];
			}

			let mut parent_hash = if continous_valid {
				0
			} else {
				GENESIS_TIME
					.with(|v| v.to_owned())
					.borrow()
					.elapsed()
					.as_nanos()
			};
			let mut chain = vec![Self::gen(start, parent_hash, validations[0])];

			parent_hash = chain[0].hash;

			if validations.len() > 1 {
				validations.remove(0);

				for (valid, number) in validations.into_iter().zip(start + 1..) {
					let header = Self::gen(number, parent_hash, valid);

					parent_hash = header.hash;

					chain.push(header);
				}
			}

			chain.reverse();

			chain
		}
	}
	impl RelayHeaderParcelInfo for MockRelayHeader {
		type HeaderId = u32;

		fn header_id(&self) -> Self::HeaderId {
			self.number
		}
	}
}

// --- std ---
use std::time::Instant;
// --- crates ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::{impl_outer_origin, parameter_types, traits::OnFinalize};
use sp_runtime::RuntimeDebug;
// --- hyperspace ---
use crate::*;
use hyperspace_relay_primitives::relayer_game::*;
use mock_relay::{MockRelayBlockNumber, MockRelayHeader};

pub type AccountId = u64;
pub type BlockNumber = u64;
pub type Balance = u128;

pub type System = frame_system::Module<Test>;
pub type Relay = mock_relay::Module<Test>;

pub type RelayerGameError = Error<Test, DefaultInstance>;
pub type RelayerGame = Module<Test, DefaultInstance>;

impl_outer_origin! {
	pub enum Origin for Test
	where
		system = frame_system
	{}
}

hyperspace_support::impl_test_account_data! { deprecated }

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const RelayerGameLockId: LockIdentifier = *b"da/rgame";
	pub static GenesisTime: Instant = Instant::now();
	pub static ChallengeTime: BlockNumber = 6;
	pub static EstimateBond: Balance = 1;
}
impl Config for Test {
	type EtpCurrency = Etp;
	type LockId = RelayerGameLockId;
	type EtpSlash = ();
	type RelayerGameAdjustor = RelayerGameAdjustor;
	type RelayableChain = Relay;
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
	type BlockNumber = BlockNumber;
	type Hash = sp_core::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::testing::Header;
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

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}
impl hyperspace_balances::Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = ();
	type OtherCurrencies = ();
	type WeightInfo = ();
}

pub struct RelayerGameAdjustor;
impl AdjustableRelayerGame for RelayerGameAdjustor {
	type Moment = BlockNumber;
	type Balance = Balance;
	type RelayHeaderId = MockRelayBlockNumber;

	fn max_active_games() -> u8 {
		32
	}

	fn affirm_time(_round: u32) -> Self::Moment {
		CHALLENGE_TIME.with(|v| v.borrow().to_owned()) / 2
	}

	fn complete_proofs_time(_round: u32) -> Self::Moment {
		CHALLENGE_TIME.with(|v| v.borrow().to_owned()) / 2
	}

	fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>) {
		sample_points.push(vec![sample_points.last().unwrap().last().unwrap() - 1]);
	}

	fn estimate_stake(_: u32, _: u32) -> Self::Balance {
		ESTIMATE_BOND.with(|v| v.borrow().to_owned())
	}
}

pub struct ExtBuilder {
	headers: Vec<MockRelayHeader>,
	challenge_time: BlockNumber,
	estimate_stake: Balance,
}
impl ExtBuilder {
	pub fn headers(mut self, headers: Vec<MockRelayHeader>) -> Self {
		self.headers = headers;

		self
	}
	pub fn challenge_time(mut self, challenge_time: BlockNumber) -> Self {
		self.challenge_time = challenge_time;

		self
	}
	pub fn estimate_stake(mut self, estimate_stake: Balance) -> Self {
		self.estimate_stake = estimate_stake;

		self
	}

	pub fn set_associated_constants(&self) {
		CHALLENGE_TIME.with(|v| v.replace(self.challenge_time));
		ESTIMATE_BOND.with(|v| v.replace(self.estimate_stake));
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.set_associated_constants();

		let _ = env_logger::try_init();
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		EtpConfig {
			balances: (1..10)
				.map(|i: AccountId| vec![(i, 100 * i as Balance), (10 * i, 1000 * i as Balance)])
				.flatten()
				.collect(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();
		mock_relay::GenesisConfig {
			headers: self.headers.clone(),
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		storage.into()
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			headers: vec![],
			challenge_time: RelayerGameAdjustor::affirm_time(0)
				+ RelayerGameAdjustor::complete_proofs_time(0),
			estimate_stake: RelayerGameAdjustor::estimate_stake(0, 0),
		}
	}
}

pub fn challenge_time() -> u64 {
	CHALLENGE_TIME.with(|v| v.borrow().to_owned())
}

pub fn run_to_block(n: BlockNumber) {
	RelayerGame::on_finalize(System::block_number());

	for b in System::block_number() + 1..=n {
		System::set_block_number(b);
		// RelayerGame::on_initialize(b);

		if b != n {
			RelayerGame::on_finalize(System::block_number());
		}
	}
}

#[allow(unused)]
pub fn println_game(game_id: MockRelayBlockNumber) {
	println!(
		"{:#?}",
		<RelayerGame as Store>::Affirmations::iter_prefix_values(game_id).collect::<Vec<_>>()
	);
}
