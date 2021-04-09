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

//! Mock file for ethereum-backing.

#[macro_export]
macro_rules! decl_tests {
	() => {
		// --- substrate ---
		use frame_support::{
			impl_outer_dispatch, impl_outer_origin, parameter_types, weights::Weight,
		};
		use sp_core::crypto::key_types;
		use sp_runtime::{
			testing::{Header, TestXt, UintAuthorityId},
			traits::{IdentifyAccount, IdentityLookup, OpaqueKeys, Verify},
			ModuleId, {KeyTypeId, MultiSignature, Perbill},
		};
		// --- hyperspace ---
		
		use hyperspace_staking::{EraIndex, Exposure, ExposureOf};

		type Balance = u128;
		type BlockNumber = u64;

		/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
		type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		/// Some way of identifying an account on the chain. We intentionally make it equivalent
		/// to the public key of our transaction signing scheme.
		type Signature = MultiSignature;

		type Extrinsic = TestXt<Call, ()>;

		type Session = pallet_session::Module<Test>;
		type System = frame_system::Module<Test>;
		type Timestamp = pallet_timestamp::Module<Test>;
		type Staking = hyperspace_staking::Module<Test>;
		type EthereumBacking = Module<Test>;

		impl_outer_origin! {
			pub enum Origin for Test where system = frame_system {}
		}

		impl_outer_dispatch! {
			pub enum Call for Test where origin: Origin {
				hyperspace_ethereum_relay::EthereumRelay,
				hyperspace_staking::Staking,
			}
		}

		hyperspace_support::impl_test_account_data! { deprecated }

		#[derive(Clone, PartialEq, Eq, Debug)]
		pub struct Test;
		pub struct EcdsaAuthorities;
		impl RelayAuthorityProtocol<BlockNumber> for EcdsaAuthorities {
			type Signer = EthereumAddress;

			fn schedule_mmr_root(_: BlockNumber) {}

			fn check_authorities_change_to_sync(_: Term, _: Vec<Self::Signer>) -> DispatchResult {
				Ok(())
			}

			fn sync_authorities_change() -> DispatchResult {
				Ok(())
			}
		}
		parameter_types! {
			pub const EthereumBackingModuleId: ModuleId = ModuleId(*b"da/backi");
			pub const EthereumBackingFeeModuleId: ModuleId = ModuleId(*b"da/ethfe");
			pub const EtpLockLimit: Balance = 1000;
			pub const DnaLockLimit: Balance = 1000;
			pub const AdvancedFee: Balance = 1;
		}
		impl Config for Test {
			type ModuleId = EthereumBackingModuleId;
			type FeeModuleId = EthereumBackingFeeModuleId;
			type Event = ();
			type RedeemAccountId = AccountId;
			type EthereumRelay = EthereumRelay;
			type OnDepositRedeem = Staking;
			type EtpCurrency = Etp;
			type DnaCurrency = Dna;
			type EtpLockLimit = EtpLockLimit;
			type DnaLockLimit = DnaLockLimit;
			type AdvancedFee = AdvancedFee;
			type SyncReward = ();
			type EcdsaAuthorities = EcdsaAuthorities;
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
			type Hash = sp_core::H256;
			type Hashing = ::sp_runtime::traits::BlakeTwo256;
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

		impl pallet_timestamp::Config for Test {
			type Moment = u64;
			type OnTimestampSet = ();
			type MinimumPeriod = ();
			type WeightInfo = ();
		}

		parameter_types! {
			pub const Period: BlockNumber = 1;
			pub const Offset: BlockNumber = 0;
		}
		impl pallet_session::Config for Test {
			type Event = ();
			type ValidatorId = AccountId;
			type ValidatorIdOf = ();
			type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
			type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
			type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Staking>;
			type SessionHandler = TestSessionHandler;
			type Keys = UintAuthorityId;
			type DisabledValidatorsThreshold = ();
			type WeightInfo = ();
		}

		impl pallet_session::historical::Config for Test {
			type FullIdentification = Exposure<AccountId, Balance, Balance>;
			type FullIdentificationOf = ExposureOf<Test>;
		}

		impl hyperspace_balances::Config<DnaInstance> for Test {
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

		parameter_types! {
			pub const StakingModuleId: ModuleId = ModuleId(*b"da/staki");
		}
		impl hyperspace_staking::Config for Test {
			type Event = ();
			type ModuleId = StakingModuleId;
			type UnixTime = Timestamp;
			type SessionsPerEra = ();
			type BondingDurationInEra = ();
			type BondingDurationInBlockNumber = ();
			type SlashDeferDuration = ();
			type SlashCancelOrigin = frame_system::EnsureRoot<Self::AccountId>;
			type SessionInterface = Self;
			type NextNewSession = Session;
			type ElectionLookahead = ();
			type Call = Call;
			type MaxIterations = ();
			type MinSolutionScoreBump = ();
			type MaxNominatorRewardedPerValidator = ();
			type UnsignedPriority = ();
			type OffchainSolutionWeightLimit = ();
			type EtpCurrency = Etp;
			type EtpRewardRemainder = ();
			type EtpSlash = ();
			type EtpReward = ();
			type DnaCurrency = Dna;
			type DnaSlash = ();
			type DnaReward = ();
			type Cap = ();
			type TotalPower = ();
			type WeightInfo = ();
		}

		impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
		where
			Call: From<LocalCall>,
		{
			type Extrinsic = Extrinsic;
			type OverarchingCall = Call;
		}

		pub struct TestSessionHandler;
		impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
			const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

			fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

			fn on_new_session<Ks: OpaqueKeys>(
				_changed: bool,
				_validators: &[(AccountId, Ks)],
				_queued_validators: &[(AccountId, Ks)],
			) {
			}

			fn on_disabled(_validator_index: usize) {}
		}
	};
}
