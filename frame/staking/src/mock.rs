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

#![allow(unused)]

// --- std ---
use std::{cell::RefCell, collections::HashSet};
// --- substrate ---
use frame_support::{
	assert_ok, parameter_types,
	storage::IterableStorageMap,
	traits::{Currency, FindAuthor, Get, OnFinalize, OnInitialize},
	weights::{constants::RocksDbWeight, Weight},
	StorageValue,
};
use frame_system::mocking::*;
use sp_core::H256;
use sp_npos_elections::{reduce, StakedAssignment};
use sp_runtime::{
	testing::{Header, TestXt, UintAuthorityId},
	traits::IdentityLookup,
	Perbill,
};
use sp_staking::{
	offence::{OffenceDetails, OnOffenceHandler},
	SessionIndex,
};
// --- hyperspace ---
use crate::{self as hyperspace_staking, *};

pub(crate) type AccountId = u64;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

type Block = MockBlock<Test>;
type UncheckedExtrinsic = MockUncheckedExtrinsic<Test>;
type Extrinsic = TestXt<Call, ()>;

pub(crate) type StakingError = Error<Test>;

pub(crate) const NANO: Balance = 1;
pub(crate) const MICRO: Balance = 1_000 * NANO;
pub(crate) const MILLI: Balance = 1_000 * MICRO;
pub(crate) const COIN: Balance = 1_000 * MILLI;

pub(crate) const CAP: Balance = 10_000_000_000 * COIN;
pub(crate) const TOTAL_POWER: Power = 1_000_000_000;

pub const INIT_TIMESTAMP: TsInMs = 30_000;
pub const BLOCK_TIME: u64 = 1_000;

hyperspace_support::impl_test_account_data! {}

/// Another session handler struct to test on_disabled.
pub struct OtherSessionHandler;
impl pallet_session::OneSessionHandler<AccountId> for OtherSessionHandler {
	type Key = UintAuthorityId;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, validators: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
		SESSION_VALIDATORS.with(|x| {
			*x.borrow_mut() = (validators.map(|x| x.0.clone()).collect(), HashSet::new())
		});
	}

	fn on_disabled(validator_index: usize) {
		SESSION_VALIDATORS.with(|d| {
			let mut d = d.borrow_mut();
			let value = d.0[validator_index];
			d.1.insert(value);
		})
	}
}
impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
	type Public = UintAuthorityId;
}

pub fn is_disabled(controller: AccountId) -> bool {
	let stash = Staking::ledger(&controller).unwrap().stash;
	SESSION_VALIDATORS.with(|d| d.borrow().1.contains(&stash))
}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			frame_support::weights::constants::WEIGHT_PER_SECOND * 2
		);
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = BlockWeights;
	type BlockLength = ();
	type DbWeight = RocksDbWeight;
	type Origin = Origin;
	type Call = Call;
	type Index = AccountIndex;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
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

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub other: OtherSessionHandler,
	}
}
parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(25);
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}
impl pallet_session::Config for Test {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = StashOf<Test>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
	type SessionHandler = (OtherSessionHandler,);
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}

impl pallet_session::historical::Config for Test {
	type FullIdentification = Exposure<AccountId, Balance, Balance>;
	type FullIdentificationOf = ExposureOf<Test>;
}

parameter_types! {
	pub const UncleGenerations: u64 = 0;
}
impl pallet_authorship::Config for Test {
	type FindAuthor = Author11;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = Module<Test>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 1024;
}
impl hyperspace_balances::Config<EtpInstance> for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type BalanceInfo = AccountData<Balance>;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
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
	type MaxLocks = MaxLocks;
	type OtherCurrencies = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const StakingModuleId: ModuleId = ModuleId(*b"da/staki");
	pub const BondingDurationInEra: EraIndex = 3;
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const UnsignedPriority: u64 = 1 << 20;
	pub const MinSolutionScoreBump: Perbill = Perbill::zero();
	pub OffchainSolutionWeightLimit: Weight = BlockWeights::get().max_block;
	pub const Cap: Balance = CAP;
	pub const TotalPower: Power = TOTAL_POWER;
	pub static SessionsPerEra: SessionIndex = 3;
	pub static BondingDurationInBlockNumber: BlockNumber = bonding_duration_in_blocks();
	pub static ExistentialDeposit: Balance = 1;
	pub static SlashDeferDuration: EraIndex = 0;
	pub static ElectionLookahead: BlockNumber = 0;
	pub static MaxIterations: u32 = 0;
	pub static SessionValidators: (Vec<AccountId>, HashSet<AccountId>) = Default::default();
	pub static EtpRewardRemainderUnbalanced: Balance = 0;
}
impl Config for Test {
	type Event = Event;
	type ModuleId = StakingModuleId;
	type UnixTime = SuppressUnixTimeWarning;
	type SessionsPerEra = SessionsPerEra;
	type BondingDurationInEra = BondingDurationInEra;
	type BondingDurationInBlockNumber = BondingDurationInBlockNumber;
	type SlashDeferDuration = SlashDeferDuration;
	type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type SessionInterface = Self;
	type NextNewSession = Session;
	type ElectionLookahead = ElectionLookahead;
	type Call = Call;
	type MaxIterations = MaxIterations;
	type MinSolutionScoreBump = MinSolutionScoreBump;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type UnsignedPriority = UnsignedPriority;
	type OffchainSolutionWeightLimit = OffchainSolutionWeightLimit;
	type EtpCurrency = Etp;
	type EtpRewardRemainder = EtpRewardRemainderMock;
	type EtpSlash = ();
	type EtpReward = ();
	type DnaCurrency = Dna;
	type DnaSlash = ();
	type DnaReward = ();
	type Cap = Cap;
	type TotalPower = TotalPower;
	type WeightInfo = ();
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	Call: From<LocalCall>,
{
	type Extrinsic = Extrinsic;
	type OverarchingCall = Call;
}

frame_support::construct_runtime! {
	pub enum Test
	where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
		Etp: hyperspace_balances::<Instance0>::{Module, Call, Storage, Config<T>, Event<T>},
		Dna: hyperspace_balances::<Instance1>::{Module, Call, Storage, Config<T>, Event<T>},
		Staking: hyperspace_staking::{Module, Call, Storage, Config<T>, Event<T>, ValidateUnsigned},
		Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
	}
}

pub struct ExtBuilder {
	validator_pool: bool,
	nominate: bool,
	validator_count: u32,
	minimum_validator_count: u32,
	fair: bool,
	num_validators: Option<u32>,
	invulnerables: Vec<AccountId>,
	has_stakers: bool,
	initialize_first_session: bool,
	init_dna: bool,
}
impl ExtBuilder {
	pub fn sessions_per_era(self, length: SessionIndex) -> Self {
		SESSIONS_PER_ERA.with(|v| *v.borrow_mut() = length);
		self
	}
	pub fn election_lookahead(self, look: BlockNumber) -> Self {
		ELECTION_LOOKAHEAD.with(|v| *v.borrow_mut() = look);
		self
	}
	pub fn period(self, length: BlockNumber) -> Self {
		PERIOD.with(|v| *v.borrow_mut() = length);
		self
	}
	pub fn existential_deposit(self, existential_deposit: Balance) -> Self {
		EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = existential_deposit);
		self
	}
	pub fn validator_pool(mut self, validator_pool: bool) -> Self {
		self.validator_pool = validator_pool;
		self
	}
	pub fn nominate(mut self, nominate: bool) -> Self {
		self.nominate = nominate;
		self
	}
	pub fn validator_count(mut self, count: u32) -> Self {
		self.validator_count = count;
		self
	}
	pub fn minimum_validator_count(mut self, count: u32) -> Self {
		self.minimum_validator_count = count;
		self
	}
	pub fn slash_defer_duration(mut self, eras: EraIndex) -> Self {
		SLASH_DEFER_DURATION.with(|v| *v.borrow_mut() = eras);
		self
	}
	pub fn fair(mut self, is_fair: bool) -> Self {
		self.fair = is_fair;
		self
	}
	pub fn num_validators(mut self, num_validators: u32) -> Self {
		self.num_validators = Some(num_validators);
		self
	}
	pub fn invulnerables(mut self, invulnerables: Vec<AccountId>) -> Self {
		self.invulnerables = invulnerables;
		self
	}
	pub fn has_stakers(mut self, has: bool) -> Self {
		self.has_stakers = has;
		self
	}
	pub fn max_offchain_iterations(self, iterations: u32) -> Self {
		MAX_ITERATIONS.with(|v| *v.borrow_mut() = iterations);
		self
	}
	pub fn init_dna(mut self, init: bool) -> Self {
		self.init_dna = init;
		self
	}
	pub fn offchain_election_ext(self) -> Self {
		self.sessions_per_era(4).period(5).election_lookahead(3)
	}
	pub fn initialize_first_session(mut self, init: bool) -> Self {
		self.initialize_first_session = init;
		self
	}
	pub fn offset(self, offset: BlockNumber) -> Self {
		OFFSET.with(|v| *v.borrow_mut() = offset);
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		sp_tracing::try_init_simple();
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let balance_factor = if ExistentialDeposit::get() > 1 {
			256
		} else {
			1
		};

		let num_validators = self.num_validators.unwrap_or(self.validator_count);
		// Check that the number of validators is sensible.
		assert!(num_validators <= 8);
		let validators = (0..num_validators)
			.map(|x| ((x + 1) * 10 + 1) as AccountId)
			.collect::<Vec<_>>();

		let _ = hyperspace_balances::GenesisConfig::<Test, EtpInstance> {
			balances: vec![
				(1, 10 * balance_factor),
				(2, 20 * balance_factor),
				(3, 300 * balance_factor),
				(4, 400 * balance_factor),
				(10, balance_factor),
				(11, balance_factor * 1000),
				(20, balance_factor),
				(21, balance_factor * 2000),
				(30, balance_factor),
				(31, balance_factor * 2000),
				(40, balance_factor),
				(41, balance_factor * 2000),
				(50, balance_factor),
				(51, balance_factor * 2000),
				(60, balance_factor),
				(61, balance_factor * 2000),
				(70, balance_factor),
				(71, balance_factor * 2000),
				(80, balance_factor),
				(81, balance_factor * 2000),
				(100, 2000 * balance_factor),
				(101, 2000 * balance_factor),
				// This allows us to have a total_payout different from 0.
				(999, 1_000_000_000_000),
			],
		}
		.assimilate_storage(&mut storage);
		if self.init_dna {
			let _ = hyperspace_balances::GenesisConfig::<Test, DnaInstance> {
				balances: vec![
					(1, 10 * balance_factor),
					(2, 20 * balance_factor),
					(3, 300 * balance_factor),
					(4, 400 * balance_factor),
					(10, balance_factor),
					(11, balance_factor * 1000),
					(20, balance_factor),
					(21, balance_factor * 2000),
					(30, balance_factor),
					(31, balance_factor * 2000),
					(40, balance_factor),
					(41, balance_factor * 2000),
					(100, 2000 * balance_factor),
					(101, 2000 * balance_factor),
					// This allows us to have a total_payout different from 0.
					(999, 1_000_000_000_000),
				],
			}
			.assimilate_storage(&mut storage);
		}

		let mut stakers = vec![];
		if self.has_stakers {
			let stake_21 = if self.fair { 1000 } else { 2000 };
			let stake_31 = if self.validator_pool {
				balance_factor * 1000
			} else {
				1
			};
			let status_41 = if self.validator_pool {
				StakerStatus::<AccountId>::Validator
			} else {
				StakerStatus::<AccountId>::Idle
			};
			let nominated = if self.nominate { vec![11, 21] } else { vec![] };
			stakers = vec![
				// (stash, controller, staked_amount, status)
				(
					11,
					10,
					balance_factor * 1000,
					StakerStatus::<AccountId>::Validator,
				),
				(21, 20, stake_21, StakerStatus::<AccountId>::Validator),
				(31, 30, stake_31, StakerStatus::<AccountId>::Validator),
				(41, 40, balance_factor * 1000, status_41),
				// nominator
				(
					101,
					100,
					balance_factor * 500,
					StakerStatus::<AccountId>::Nominator(nominated),
				),
			];
		}
		let _ = hyperspace_staking::GenesisConfig::<Test> {
			history_depth: 84,
			stakers,
			validator_count: self.validator_count,
			minimum_validator_count: self.minimum_validator_count,
			invulnerables: self.invulnerables,
			slash_reward_fraction: Perbill::from_percent(10),
			payout_fraction: Perbill::from_percent(50),
			..Default::default()
		}
		.assimilate_storage(&mut storage);

		let _ = pallet_session::GenesisConfig::<Test> {
			keys: validators
				.iter()
				.map(|x| {
					(
						*x,
						*x,
						SessionKeys {
							other: UintAuthorityId(*x as u64),
						},
					)
				})
				.collect(),
		}
		.assimilate_storage(&mut storage);

		let mut ext = sp_io::TestExternalities::from(storage);
		ext.execute_with(|| {
			let validators = Session::validators();
			SESSION_VALIDATORS.with(|x| *x.borrow_mut() = (validators.clone(), HashSet::new()));
		});

		if self.initialize_first_session {
			// We consider all test to start after timestamp is initialized This must be ensured by
			// having `timestamp::on_initialize` called before `staking::on_initialize`. Also, if
			// session length is 1, then it is already triggered.
			ext.execute_with(|| {
				System::set_block_number(1);
				Session::on_initialize(1);
				Staking::on_initialize(1);
				Timestamp::set_timestamp(INIT_TIMESTAMP);
			});
		}

		ext
	}
	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
		ext.execute_with(post_conditions);
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			validator_pool: false,
			nominate: true,
			validator_count: 2,
			minimum_validator_count: 0,
			fair: true,
			num_validators: None,
			invulnerables: vec![],
			has_stakers: true,
			initialize_first_session: true,
			init_dna: false,
		}
	}
}

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<AccountId> for Author11 {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some(11)
	}
}

pub struct EtpRewardRemainderMock;
impl OnUnbalanced<EtpNegativeImbalance<Test>> for EtpRewardRemainderMock {
	fn on_nonzero_unbalanced(amount: EtpNegativeImbalance<Test>) {
		ETP_REWARD_REMAINDER_UNBALANCED.with(|v| {
			*v.borrow_mut() += amount.peek();
		});
		drop(amount);
	}
}

pub struct SuppressUnixTimeWarning;
impl UnixTime for SuppressUnixTimeWarning {
	fn now() -> core::time::Duration {
		core::time::Duration::from_millis(Timestamp::now().saturated_into::<u64>())
	}
}

fn post_conditions() {
	check_nominators();
	check_exposures();
	check_ledgers();
}

fn check_ledgers() {
	// check the ledger of all stakers.
	<Bonded<Test>>::iter().for_each(|(_, controller)| assert_ledger_consistent(controller))
}

fn check_exposures() {
	// a check per validator to ensure the exposure struct is always sane.
	let era = active_era();
	<ErasStakers<Test>>::iter_prefix_values(era).for_each(|expo| {
		assert_eq!(
			expo.total_power,
			expo.own_power + expo.others.iter().map(|e| e.power).sum::<Power>(),
			"wrong total exposure.",
		);
	})
}

fn check_nominators() {
	// a check per nominator to ensure their entire stake is correctly distributed. Will only kick-
	// in if the nomination was submitted before the current era.
	let era = active_era();
	<Nominators<Test>>::iter()
		.filter_map(|(nominator, nomination)| {
			if nomination.submitted_in > era {
				Some(nominator)
			} else {
				None
			}
		})
		.for_each(|nominator| {
			// must be bonded.
			assert_is_stash(nominator);
			let mut sum = 0;
			Session::validators()
				.iter()
				.map(|v| Staking::eras_stakers(era, v))
				.for_each(|e| {
					let individual = e
						.others
						.iter()
						.filter(|e| e.who == nominator)
						.collect::<Vec<_>>();
					let len = individual.len();
					match len {
						0 => { /* not supporting this validator at all. */ }
						1 => sum += individual[0].power,
						_ => panic!("nominator cannot back a validator more than once."),
					};
				});

			let nominator_stake = Staking::power_of(&nominator);
			// a nominator cannot over-spend.
			assert!(
				nominator_stake >= sum,
				"failed: Nominator({}) stake({}) >= sum divided({})",
				nominator,
				nominator_stake,
				sum,
			);

			let diff = nominator_stake - sum;
			assert!(diff < 100);
		});
}

fn assert_is_stash(acc: AccountId) {
	assert!(Staking::bonded(&acc).is_some(), "Not a stash.");
}

pub fn assert_ledger_consistent(controller: AccountId) {
	let ledger = Staking::ledger(controller).unwrap();

	assert_eq!(ledger.active_etp, ledger.etp_staking_lock.staking_amount);
	assert_eq!(ledger.active_dna, ledger.dna_staking_lock.staking_amount);

	let real_total_etp: Balance = ledger
		.etp_staking_lock
		.unbondings
		.iter()
		.fold(ledger.active_etp, |a, c| a + c.amount);
	let real_total_dna: Balance = ledger
		.dna_staking_lock
		.unbondings
		.iter()
		.fold(ledger.active_dna, |a, c| a + c.amount);

	assert!(
		ledger.active_etp >= Etp::minimum_balance()
			|| ledger.active_dna >= Dna::minimum_balance()
			|| (ledger.active_etp == 0 && ledger.active_dna == 0),
		"{}: active ledger amount ({}/{}) must be greater than ED {}/{}",
		controller,
		ledger.active_etp,
		ledger.active_dna,
		Etp::minimum_balance(),
		Dna::minimum_balance()
	);
}

pub(crate) fn active_era() -> EraIndex {
	Staking::active_era().unwrap().index
}

pub(crate) fn current_era() -> EraIndex {
	Staking::current_era().unwrap()
}

fn bond(stash: AccountId, controller: AccountId, val: StakingBalanceT<Test>) {
	match val {
		StakingBalance::EtpBalance(r) => {
			let _ = Etp::make_free_balance_be(&(stash), r);
			let _ = Etp::make_free_balance_be(&(controller), r);
		}
		StakingBalance::DnaBalance(k) => {
			let _ = Dna::make_free_balance_be(&(stash), k);
			let _ = Dna::make_free_balance_be(&(controller), k);
		}
	}
	assert_ok!(Staking::bond(
		Origin::signed(stash),
		controller,
		val,
		RewardDestination::Controller,
		0,
	));
}

pub(crate) fn bond_validator(stash: AccountId, controller: AccountId, val: StakingBalanceT<Test>) {
	bond(stash, controller, val);
	assert_ok!(Staking::validate(
		Origin::signed(controller),
		ValidatorPrefs::default()
	));
}

pub(crate) fn bond_nominator(
	stash: AccountId,
	controller: AccountId,
	val: StakingBalanceT<Test>,
	target: Vec<AccountId>,
) {
	bond(stash, controller, val);
	assert_ok!(Staking::nominate(Origin::signed(controller), target));
}

/// Progress to the given block, triggering session and era changes as we progress.
///
/// This will finalize the previous block, initialize up to the given block, essentially simulating
/// a block import/propose process where we first initialize the block, then execute some stuff (not
/// in the function), and then finalize the block.
pub(crate) fn run_to_block(n: BlockNumber) {
	Staking::on_finalize(System::block_number());
	for b in System::block_number() + 1..=n {
		System::set_block_number(b);
		Session::on_initialize(b);
		Staking::on_initialize(b);
		Timestamp::set_timestamp(System::block_number() * BLOCK_TIME + INIT_TIMESTAMP);
		if b != n {
			Staking::on_finalize(System::block_number());
		}
	}
}

/// Progresses from the current block number (whatever that may be) to the `P * session_index + 1`.
pub(crate) fn start_session(session_index: SessionIndex) {
	let end: u64 = if Offset::get().is_zero() {
		(session_index as u64) * Period::get()
	} else {
		Offset::get() + (session_index.saturating_sub(1) as u64) * Period::get()
	};
	run_to_block(end);
	// session must have progressed properly.
	assert_eq!(
		Session::current_index(),
		session_index,
		"current session index = {}, expected = {}",
		Session::current_index(),
		session_index,
	);
}

pub(crate) fn advance_session() {
	let current_index = Session::current_index();
	start_session(current_index + 1);
}

/// Progress until the given era.
pub(crate) fn start_active_era(era_index: EraIndex) {
	start_session((era_index * <SessionsPerEra as Get<u32>>::get()).into());
	assert_eq!(active_era(), era_index);
	// One way or another, current_era must have changed before the active era, so they must match
	// at this point.
	assert_eq!(current_era(), active_era());
}

pub(crate) fn current_total_payout_for_duration(duration: TsInMs) -> Balance {
	inflation::compute_total_payout::<Test>(
		duration,
		Staking::living_time(),
		<Test as Config>::Cap::get() - Etp::total_issuance(),
		Perbill::from_percent(50),
	)
	.0
}

pub(crate) fn maximum_payout_for_duration(duration: u64) -> Balance {
	inflation::compute_total_payout::<Test>(
		duration,
		Staking::living_time(),
		<Test as Config>::Cap::get() - Etp::total_issuance(),
		Perbill::from_percent(50),
	)
	.1
}

/// Time it takes to finish a session.
///
/// Note, if you see `time_per_session() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_session() -> u64 {
	Period::get() * BLOCK_TIME
}

/// Time it takes to finish an era.
///
/// Note, if you see `time_per_era() - BLOCK_TIME`, it is fine. This is because we set the
/// timestamp after on_initialize, so the timestamp is always one block old.
pub(crate) fn time_per_era() -> u64 {
	time_per_session() * SessionsPerEra::get() as u64
}

/// Time that will be calculated for the reward per era.
pub(crate) fn reward_time_per_era() -> u64 {
	time_per_era() - BLOCK_TIME
}

pub(crate) fn bonding_duration_in_blocks() -> BlockNumber {
	BondingDurationInEra::get() as BlockNumber * Period::get()
}

pub(crate) fn reward_all_elected() {
	let rewards = <Test as Config>::SessionInterface::validators()
		.into_iter()
		.map(|v| (v, 1));

	Staking::reward_by_ids(rewards)
}

pub(crate) fn validator_controllers() -> Vec<AccountId> {
	Session::validators()
		.into_iter()
		.map(|s| Staking::bonded(&s).expect("no controller for validator"))
		.collect()
}

pub(crate) fn on_offence_in_era(
	offenders: &[OffenceDetails<
		AccountId,
		pallet_session::historical::IdentificationTuple<Test>,
	>],
	slash_fraction: &[Perbill],
	era: EraIndex,
) {
	let bonded_eras = BondedEras::get();
	for &(bonded_era, start_session) in bonded_eras.iter() {
		if bonded_era == era {
			let _ = Staking::on_offence(offenders, slash_fraction, start_session).unwrap();
			return;
		} else if bonded_era > era {
			break;
		}
	}

	if active_era() == era {
		Staking::on_offence(
			offenders,
			slash_fraction,
			Staking::eras_start_session_index(era).unwrap(),
		)
		.unwrap();
	} else {
		panic!("cannot slash in era {}", era);
	}
}

pub(crate) fn on_offence_now(
	offenders: &[OffenceDetails<
		AccountId,
		pallet_session::historical::IdentificationTuple<Test>,
	>],
	slash_fraction: &[Perbill],
) {
	let now = active_era();
	on_offence_in_era(offenders, slash_fraction, now)
}

pub(crate) fn add_slash(who: &AccountId) {
	on_offence_now(
		&[OffenceDetails {
			offender: (
				who.clone(),
				Staking::eras_stakers(active_era(), who.clone()),
			),
			reporters: vec![],
		}],
		&[Perbill::from_percent(10)],
	);
}

// winners will be chosen by simply their unweighted total backing stake. Nominator stake is
// distributed evenly.
pub(crate) fn horrible_npos_solution(
	do_reduce: bool,
) -> (CompactAssignments, Vec<ValidatorIndex>, ElectionScore) {
	let mut backing_stake_of: BTreeMap<AccountId, Balance> = BTreeMap::new();

	// self stake
	<Validators<Test>>::iter().for_each(|(who, _p)| {
		*backing_stake_of.entry(who).or_insert(0) += Staking::power_of(&who) as Balance
	});

	// add nominator stuff
	<Nominators<Test>>::iter().for_each(|(who, nomination)| {
		nomination.targets.iter().for_each(|v| {
			*backing_stake_of.entry(*v).or_insert(0) += Staking::power_of(&who) as Balance
		})
	});

	// elect winners
	let mut sorted: Vec<AccountId> = backing_stake_of.keys().cloned().collect();
	sorted.sort_by_key(|x| backing_stake_of.get(x).unwrap());
	let winners: Vec<AccountId> = sorted
		.iter()
		.cloned()
		.take(Staking::validator_count() as usize)
		.collect();

	// create assignments
	let mut staked_assignment: Vec<StakedAssignment<AccountId>> = Vec::new();
	<Nominators<Test>>::iter().for_each(|(who, nomination)| {
		let mut dist: Vec<(AccountId, ExtendedBalance)> = Vec::new();
		nomination.targets.iter().for_each(|v| {
			if winners.iter().find(|w| *w == v).is_some() {
				dist.push((*v, ExtendedBalance::zero()));
			}
		});

		if dist.len() == 0 {
			return;
		}

		// assign real stakes. just split the stake.
		let stake = Staking::power_of(&who) as ExtendedBalance;
		let mut sum: ExtendedBalance = Zero::zero();
		let dist_len = dist.len();
		{
			dist.iter_mut().for_each(|(_, w)| {
				let partial = stake / (dist_len as ExtendedBalance);
				*w = partial;
				sum += partial;
			});
		}

		// assign the leftover to last.
		{
			let leftover = stake - sum;
			let last = dist.last_mut().unwrap();
			last.1 += leftover;
		}

		staked_assignment.push(StakedAssignment {
			who,
			distribution: dist,
		});
	});

	// Ensure that this result is worse than seq-phragmen. Otherwise, it should not have been used
	// for testing.
	let score = {
		let (_, _, better_score) = prepare_submission_with(true, true, 0, |_| {});

		let support = to_support_map::<AccountId>(&winners, &staked_assignment).unwrap();
		let score = support.evaluate();

		assert!(sp_npos_elections::is_score_better::<Perbill>(
			better_score,
			score,
			MinSolutionScoreBump::get(),
		));

		score
	};

	if do_reduce {
		reduce(&mut staked_assignment);
	}

	let snapshot_validators = Staking::snapshot_validators().unwrap();
	let snapshot_nominators = Staking::snapshot_nominators().unwrap();
	let nominator_index = |a: &AccountId| -> Option<NominatorIndex> {
		snapshot_nominators
			.iter()
			.position(|x| x == a)
			.map(|i| i as NominatorIndex)
	};
	let validator_index = |a: &AccountId| -> Option<ValidatorIndex> {
		snapshot_validators
			.iter()
			.position(|x| x == a)
			.map(|i| i as ValidatorIndex)
	};

	// convert back to ratio assignment. This takes less space.
	let assignments_reduced = sp_npos_elections::assignment_staked_to_ratio::<
		AccountId,
		OffchainAccuracy,
	>(staked_assignment);

	let compact =
		CompactAssignments::from_assignment(assignments_reduced, nominator_index, validator_index)
			.unwrap();

	// winner ids to index
	let winners = winners
		.into_iter()
		.map(|w| validator_index(&w).unwrap())
		.collect::<Vec<_>>();

	(compact, winners, score)
}

/// Note: this should always logically reproduce [`offchain_election::prepare_submission`], yet we
/// cannot do it since we want to have `tweak` injected into the process.
///
/// If the input is being tweaked in a way that the score cannot be compute accurately,
/// `compute_real_score` can be set to true. In this case a `Default` score is returned.
pub(crate) fn prepare_submission_with(
	compute_real_score: bool,
	do_reduce: bool,
	iterations: usize,
	tweak: impl FnOnce(&mut Vec<StakedAssignment<AccountId>>),
) -> (CompactAssignments, Vec<ValidatorIndex>, ElectionScore) {
	// run election on the default stuff.
	let sp_npos_elections::ElectionResult {
		winners,
		assignments,
	} = Staking::do_phragmen::<OffchainAccuracy>(iterations).unwrap();
	let winners = sp_npos_elections::to_without_backing(winners);

	let mut staked = sp_npos_elections::assignment_ratio_to_staked(assignments, |stash| {
		Staking::power_of(stash) as _
	});

	// apply custom tweaks. awesome for testing.
	tweak(&mut staked);

	if do_reduce {
		reduce(&mut staked);
	}

	// convert back to ratio assignment. This takes less space.
	let snapshot_validators = Staking::snapshot_validators().expect("snapshot not created.");
	let snapshot_nominators = Staking::snapshot_nominators().expect("snapshot not created.");
	let nominator_index = |a: &AccountId| -> Option<NominatorIndex> {
		snapshot_nominators.iter().position(|x| x == a).map_or_else(
			|| {
				println!("unable to find nominator index for {:?}", a);
				None
			},
			|i| Some(i as NominatorIndex),
		)
	};
	let validator_index = |a: &AccountId| -> Option<ValidatorIndex> {
		snapshot_validators.iter().position(|x| x == a).map_or_else(
			|| {
				println!("unable to find validator index for {:?}", a);
				None
			},
			|i| Some(i as ValidatorIndex),
		)
	};

	let assignments_reduced = sp_npos_elections::assignment_staked_to_ratio(staked);

	// re-compute score by converting, yet again, into staked type
	let score = if compute_real_score {
		let staked =
			sp_npos_elections::assignment_ratio_to_staked(assignments_reduced.clone(), |stash| {
				Staking::power_of(stash) as _
			});

		let support_map =
			to_support_map::<AccountId>(winners.as_slice(), staked.as_slice()).unwrap();
		support_map.evaluate()
	} else {
		Default::default()
	};

	let compact =
		CompactAssignments::from_assignment(assignments_reduced, nominator_index, validator_index)
			.map_err(|e| {
				println!("error in compact: {:?}", e);
				e
			})
			.expect("Failed to create compact");

	// winner ids to index
	let winners = winners
		.into_iter()
		.map(|w| validator_index(&w).unwrap())
		.collect::<Vec<_>>();

	(compact, winners, score)
}

/// Make all validator and nominator request their payment
pub(crate) fn make_all_reward_payment(era: EraIndex) {
	let validators_with_reward = <ErasRewardPoints<Test>>::get(era)
		.individual
		.keys()
		.cloned()
		.collect::<Vec<_>>();

	// reward validators
	for validator_controller in validators_with_reward.iter().filter_map(Staking::bonded) {
		let ledger = <Ledger<Test>>::get(&validator_controller).unwrap();

		assert_ok!(Staking::payout_stakers(
			Origin::signed(1337),
			ledger.stash,
			era
		));
	}
}

pub(crate) fn staking_events() -> Vec<hyperspace_staking::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::hyperspace_staking(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect()
}

pub(crate) fn etp_balances(who: &AccountId) -> (Balance, Balance) {
	(Etp::free_balance(who), Etp::reserved_balance(who))
}
pub(crate) fn dna_balances(who: &AccountId) -> (Balance, Balance) {
	(Dna::free_balance(who), Dna::reserved_balance(who))
}

#[macro_export]
macro_rules! assert_session_era {
	($session:expr, $era:expr) => {
		assert_eq!(
			Session::current_index(),
			$session,
			"wrong session {} != {}",
			Session::current_index(),
			$session,
		);
		assert_eq!(
			active_era(),
			$era,
			"wrong active era {} != {}",
			active_era(),
			$era,
		);
	};
}
