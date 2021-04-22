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

//! # Staking Module
//!
//! The Staking module is used to manage funds at stake by network maintainers.
//!
//! - [`staking::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The Staking module is the means by which a set of network maintainers (known as _authorities_ in
//! some contexts and _validators_ in others) are chosen based upon those who voluntarily place
//! funds under deposit. Under deposit, those funds are rewarded under normal operation but are held
//! at pain of _slash_ (expropriation) should the staked maintainer be found not to be discharging
//! its duties properly.
//!
//! ### Terminology
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! - Staking: The process of locking up funds for some time, placing them at risk of slashing
//!   (loss) in order to become a rewarded maintainer of the network.
//! - Validating: The process of running a node to actively maintain the network, either by
//!   producing blocks or guaranteeing finality of the chain.
//! - Nominating: The process of placing staked funds behind one or more validators in order to
//!   share in any reward, and punishment, they take.
//! - Stash account: The account holding an owner's funds used for staking.
//! - Controller account: The account that controls an owner's funds for staking.
//! - Era: A (whole) number of sessions, which is the period that the validator set (and each
//!   validator's active nominator set) is recalculated and where rewards are paid out.
//! - Slash: The punishment of a staker by reducing its funds.
//!
//! ### Goals
//! <!-- Original author of paragraph: @gavofyork -->
//!
//! The staking system in Hyperspace NPoS is designed to make the following possible:
//!
//! - Stake funds that are controlled by a cold wallet.
//! - Withdraw some, or deposit more, funds without interrupting the role of an entity.
//! - Switch between roles (nominator, validator, idle) with minimal overhead.
//!
//! ### Scenarios
//!
//! #### Staking
//!
//! Almost any interaction with the Staking module requires a process of _**bonding**_ (also known
//! as being a _staker_). To become *bonded*, a fund-holding account known as the _stash account_,
//! which holds some or all of the funds that become frozen in place as part of the staking process,
//! is paired with an active **controller** account, which issues instructions on how they shall be
//! used.
//!
//! An account pair can become bonded using the [`bond`](./enum.Call.html#variant.bond) call.
//!
//! Stash accounts can change their associated controller using the
//! [`set_controller`](./enum.Call.html#variant.set_controller) call.
//!
//! There are three possible roles that any staked account pair can be in: `Validator`, `Nominator`
//! and `Idle` (defined in [`StakerStatus`](./enum.StakerStatus.html)). There are three
//! corresponding instructions to change between roles, namely:
//! [`validate`](./enum.Call.html#variant.validate),
//! [`nominate`](./enum.Call.html#variant.nominate), and [`chill`](./enum.Call.html#variant.chill).
//!
//! #### Validating
//!
//! A **validator** takes the role of either validating blocks or ensuring their finality,
//! maintaining the veracity of the network. A validator should avoid both any sort of malicious
//! misbehavior and going offline. Bonded accounts that state interest in being a validator do NOT
//! get immediately chosen as a validator. Instead, they are declared as a _candidate_ and they
//! _might_ get elected at the _next era_ as a validator. The result of the election is determined
//! by nominators and their votes.
//!
//! An account can become a validator candidate via the
//! [`validate`](./enum.Call.html#variant.validate) call.
//!
//! #### Nomination
//!
//! A **nominator** does not take any _direct_ role in maintaining the network, instead, it votes on
//! a set of validators  to be elected. Once interest in nomination is stated by an account, it
//! takes effect at the next election round. The funds in the nominator's stash account indicate the
//! _weight_ of its vote. Both the rewards and any punishment that a validator earns are shared
//! between the validator and its nominators. This rule incentivizes the nominators to NOT vote for
//! the misbehaving/offline validators as much as possible, simply because the nominators will also
//! lose funds if they vote poorly.
//!
//! An account can become a nominator via the [`nominate`](enum.Call.html#variant.nominate) call.
//!
//! #### Rewards and Slash
//!
//! The **reward and slashing** procedure is the core of the Staking module, attempting to _embrace
//! valid behavior_ while _punishing any misbehavior or lack of availability_.
//!
//! `payout_stakers` call. Any account can call `payout_stakers`, which pays the reward to the
//! validator as well as its nominators. Only the [`Config::MaxNominatorRewardedPerValidator`]
//! biggest stakers can claim their reward. This is to limit the i/o cost to mutate storage for each
//! nominator's account.
//!
//! Slashing can occur at any point in time, once misbehavior is reported. Once slashing is
//! determined, a value is deducted from the balance of the validator and all the nominators who
//! voted for this validator (values are deducted from the _stash_ account of the slashed entity).
//!
//! Slashing logic is further described in the documentation of the `slashing` module.
//!
//! Similar to slashing, rewards are also shared among a validator and its associated nominators.
//! Yet, the reward funds are not always transferred to the stash account and can be configured. See
//! [Reward Calculation](#reward-calculation) for more details.
//!
//! #### Chilling
//!
//! Finally, any of the roles above can choose to step back temporarily and just chill for a while.
//! This means that if they are a nominator, they will not be considered as voters anymore and if
//! they are validators, they will no longer be a candidate for the next election.
//!
//! An account can step back via the [`chill`](enum.Call.html#variant.chill) call.
//!
//! ### Session managing
//!
//! The module implement the trait `SessionManager`. Which is the only API to query new validator
//! set and allowing these validator set to be rewarded once their era is ended.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! The dispatchable functions of the Staking module enable the steps needed for entities to accept
//! and change their role, alongside some helper functions to get/set the metadata of the module.
//!
//! ### Public Functions
//!
//! The Staking module contains many public storage items and (im)mutable functions.
//!
//! ## Usage
//!
//! ### Example: Rewarding a validator by id.
//!
//! ```
//! use frame_support::{decl_module, dispatch};
//! use frame_system::ensure_signed;
//! use hyperspace_staking as staking;
//!
//! pub trait Config: staking::Config {}
//!
//! decl_module! {
//! 	pub struct Module<T: Config> for enum Call where origin: T::Origin {
//! 		/// Reward a validator.
//! 		#[weight = 0]
//! 		pub fn reward_myself(origin) -> dispatch::DispatchResult {
//! 			let reported = ensure_signed(origin)?;
//! 			<staking::Module<T>>::reward_by_ids(vec![(reported, 10)]);
//! 			Ok(())
//! 		}
//! 	}
//! }
//! # fn main() {}
//! ```
//!
//! ## Implementation Details
//!
//! ### Era payout
//!
//! The era payout is computed using yearly inflation curve defined at
//! [`T::RewardCurve`](./trait.Config.html#associatedtype.RewardCurve) as such:
//!
//! ```nocompile
//! staker_payout = yearly_inflation(npos_token_staked / total_tokens) * total_tokens / era_per_year
//! ```
//! This payout is used to reward stakers as defined in next section
//!
//! ```nocompile
//! remaining_payout = max_yearly_inflation * total_tokens / era_per_year - staker_payout
//! ```
//! The remaining reward is send to the configurable end-point
//! [`T::RewardRemainder`](./trait.Config.html#associatedtype.RewardRemainder).
//!
//! ### Reward Calculation
//!
//! Validators and nominators are rewarded at the end of each era. The total reward of an era is
//! calculated using the era duration and the staking rate (the total amount of tokens staked by
//! nominators and validators, divided by the total token supply). It aims to incentivize toward a
//! defined staking rate. The full specification can be found
//! [here](https://research.web3.foundation/en/latest/polkadot/Token%20Economics.html#inflation-model).
//!
//! Total reward is split among validators and their nominators depending on the number of points
//! they received during the era. Points are added to a validator using
//! [`reward_by_ids`](./enum.Call.html#variant.reward_by_ids) or
//! [`reward_by_indices`](./enum.Call.html#variant.reward_by_indices).
//!
//! [`Module`](./struct.Module.html) implements
//! [`pallet_authorship::EventHandler`](../pallet_authorship/trait.EventHandler.html) to add reward
//! points to block producer and block producer of referenced uncles.
//!
//! The validator and its nominator split their reward as following:
//!
//! The validator can declare an amount, named
//! [`commission`](./struct.ValidatorPrefs.html#structfield.commission), that does not get shared
//! with the nominators at each reward payout through its
//! [`ValidatorPrefs`](./struct.ValidatorPrefs.html). This value gets deducted from the total reward
//! that is paid to the validator and its nominators. The remaining portion is split among the
//! validator and all of the nominators that nominated the validator, proportional to the value
//! staked behind this validator (_i.e._ dividing the
//! [`own`](./struct.Exposure.html#structfield.own) or
//! [`others`](./struct.Exposure.html#structfield.others) by
//! [`total`](./struct.Exposure.html#structfield.total) in [`Exposure`](./struct.Exposure.html)).
//!
//! All entities who receive a reward have the option to choose their reward destination through the
//! [`Payee`](./struct.Payee.html) storage item (see
//! [`set_payee`](enum.Call.html#variant.set_payee)), to be one of the following:
//!
//! - Controller account, (obviously) not increasing the staked value.
//! - Stash account, not increasing the staked value.
//! - Stash account, also increasing the staked value.
//!
//! ### Additional Fund Management Operations
//!
//! Any funds already placed into stash can be the target of the following operations:
//!
//! The controller account can free a portion (or all) of the funds using the
//! [`unbond`](enum.Call.html#variant.unbond) call. Note that the funds are not immediately
//! accessible. Instead, a duration denoted by
//! [`BondingDuration`](./trait.Config.html#associatedtype.BondingDuration) (in number of eras) must
//! pass until the funds can actually be removed. Once the `BondingDuration` is over, the
//! [`withdraw_unbonded`](./enum.Call.html#variant.withdraw_unbonded) call can be used to actually
//! withdraw the funds.
//!
//! Note that there is a limitation to the number of fund-chunks that can be scheduled to be
//! unlocked in the future via [`unbond`](enum.Call.html#variant.unbond). In case this maximum
//! (`MAX_UNLOCKING_CHUNKS`) is reached, the bonded account _must_ first wait until a successful
//! call to `withdraw_unbonded` to remove some of the chunks.
//!
//! ### Election Algorithm
//!
//! The current election algorithm is implemented based on Phragmén. The reference implementation
//! can be found [here](https://github.com/w3f/consensus/tree/master/NPoS).
//!
//! The election algorithm, aside from electing the validators with the most stake value and votes,
//! tries to divide the nominator votes among candidates in an equal manner. To further assure this,
//! an optional post-processing can be applied that iteratively normalizes the nominator staked
//! values until the total difference among votes of a particular nominator are less than a
//! threshold.
//!
//! ## GenesisConfig
//!
//! The Staking module depends on the [`GenesisConfig`](./struct.GenesisConfig.html). The
//! `GenesisConfig` is optional and allow to set some initial stakers.
//!
//! ## Related Modules
//!
//! - [Balances](../pallet_balances/index.html): Used to manage values at stake.
//! - [Session](../pallet_session/index.html): Used to manage sessions. Also, a list of new
//!   validators is stored in the Session module's `Validators` at the end of each era.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(drain_filter)]
#![recursion_limit = "128"]

// TODO: offchain phragmen test https://github.com/new-mvs/darwinia-common/issues/97
// #[cfg(features = "testing-utils")]
// pub mod testing_utils;
#[cfg(test)]
mod hyperspace_tests;
#[cfg(test)]
mod inflation_tests;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod substrate_tests;

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

pub mod inflation;
pub mod offchain_election;
pub mod slashing;

pub mod migrations {
	use super::*;

	#[derive(Decode)]
	struct OldValidatorPrefs {
		#[codec(compact)]
		pub commission: Perbill,
	}
	impl OldValidatorPrefs {
		fn upgraded(self) -> ValidatorPrefs {
			ValidatorPrefs {
				commission: self.commission,
				..Default::default()
			}
		}
	}
	pub fn migrate_to_blockable<T: Config>() -> frame_support::weights::Weight {
		<Validators<T>>::translate::<OldValidatorPrefs, _>(|_, p| Some(p.upgraded()));
		<ErasValidatorPrefs<T>>::translate::<OldValidatorPrefs, _>(|_, _, p| Some(p.upgraded()));
		T::BlockWeights::get().max_block
	}
}

mod types {
	// --- hyperspace ---
	use crate::*;

	/// Counter for the number of eras that have passed.
	pub type EraIndex = u32;
	/// Counter for the number of "reward" points earned by a given validator.
	pub type RewardPoint = u32;

	/// Data type used to index nominators in the compact type
	pub type NominatorIndex = u32;
	/// Data type used to index validators in the compact type.
	pub type ValidatorIndex = u16;
	// Ensure the size of both ValidatorIndex and NominatorIndex. They both need to be well below usize.
	static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<usize>());
	static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<usize>());
	static_assertions::const_assert!(size_of::<ValidatorIndex>() <= size_of::<u32>());
	static_assertions::const_assert!(size_of::<NominatorIndex>() <= size_of::<u32>());

	// Note: Maximum nomination limit is set here -- 16.
	generate_solution_type!(
		#[compact]
		pub struct CompactAssignments::<NominatorIndex, ValidatorIndex, OffchainAccuracy>(16)
	);
	/// Accuracy used for on-chain election.
	pub type ChainAccuracy = Perbill;
	/// Accuracy used for off-chain election. This better be small.
	pub type OffchainAccuracy = PerU16;

	/// Balance of an account.
	pub type Balance = u128;
	/// Power of an account.
	pub type Power = u32;
	/// A timestamp: milliseconds since the unix epoch.
	/// `u64` is enough to represent a duration of half a billion years, when the
	/// time scale is milliseconds.
	pub type TsInMs = u64;

	pub type AccountId<T> = <T as frame_system::Config>::AccountId;
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;

	/// The balance type of this module.
	pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type EtpPositiveImbalance<T> =
		<EtpCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type EtpNegativeImbalance<T> =
		<EtpCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

	/// The balance type of this module.
	pub type DnaBalance<T> = <DnaCurrency<T> as Currency<AccountId<T>>>::Balance;
	pub type DnaPositiveImbalance<T> =
		<DnaCurrency<T> as Currency<AccountId<T>>>::PositiveImbalance;
	pub type DnaNegativeImbalance<T> =
		<DnaCurrency<T> as Currency<AccountId<T>>>::NegativeImbalance;

	pub type StakingLedgerT<T> =
		StakingLedger<AccountId<T>, EtpBalance<T>, DnaBalance<T>, BlockNumber<T>>;
	pub type StakingBalanceT<T> = StakingBalance<EtpBalance<T>, DnaBalance<T>>;

	pub type ExposureT<T> = Exposure<AccountId<T>, EtpBalance<T>, DnaBalance<T>>;
	pub type ElectionResultT<T> = ElectionResult<AccountId<T>, EtpBalance<T>, DnaBalance<T>>;

	type EtpCurrency<T> = <T as Config>::EtpCurrency;
	type DnaCurrency<T> = <T as Config>::DnaCurrency;
}

// --- hyperspace ---
pub use types::EraIndex;

// --- crates ---
use codec::{Decode, Encode, HasCompact};
// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, WithPostDispatchInfo},
	ensure,
	storage::IterableStorageMap,
	traits::{
		Currency, EnsureOrigin, EstimateNextNewSession, ExistenceRequirement::KeepAlive, Get,
		Imbalance, IsSubType, OnUnbalanced, UnixTime,
	},
	weights::{
		constants::{WEIGHT_PER_MICROS, WEIGHT_PER_NANOS},
		Weight,
	},
};
use frame_system::{ensure_none, ensure_root, ensure_signed, offchain::SendTransactionTypes};
use sp_npos_elections::{
	generate_solution_type, is_score_better, seq_phragmen, to_support_map, Assignment,
	CompactSolution, ElectionResult as PrimitiveElectionResult, ElectionScore, EvaluateSupport,
	ExtendedBalance, PerThing128, SupportMap, VoteWeight,
};
use sp_runtime::{
	helpers_128bit::multiply_by_rational,
	traits::{
		AccountIdConversion, AtLeast32BitUnsigned, CheckedSub, Convert, Dispatchable,
		SaturatedConversion, Saturating, StaticLookup, Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		TransactionValidityError, ValidTransaction,
	},
	DispatchError, DispatchResult, ModuleId, PerU16, Perbill, Percent, Perquintill, RuntimeDebug,
};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_staking::{
	offence::{Offence, OffenceDetails, OffenceError, OnOffenceHandler, ReportOffence},
	SessionIndex,
};
#[cfg(not(feature = "std"))]
use sp_std::borrow::ToOwned;
use sp_std::{
	collections::btree_map::BTreeMap, convert::TryInto, marker::PhantomData, mem::size_of,
	prelude::*,
};
// --- hyperspace ---
use hyperspace_staking_rpc_runtime_api::RuntimeDispatchInfo;
use hyperspace_support::{
	balance::lock::*,
	impl_rpc,
	traits::{OnDepositRedeem, OnUnbalancedDna},
};
use types::*;

pub(crate) const LOG_TARGET: &'static str = "staking";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		frame_support::debug::$level!(
			target: crate::LOG_TARGET,
			$patter $(, $values)*
		)
	};
}

pub const MAX_NOMINATIONS: usize = <CompactAssignments as CompactSolution>::LIMIT;

pub(crate) const MAX_UNLOCKING_CHUNKS: usize = 32;
/// Maximum number of stakers that can be stored in a snapshot.
pub(crate) const MAX_VALIDATORS: usize = ValidatorIndex::max_value() as usize;
pub(crate) const MAX_NOMINATORS: usize = NominatorIndex::max_value() as usize;

const MONTH_IN_MINUTES: TsInMs = 30 * 24 * 60;
const MONTH_IN_MILLISECONDS: TsInMs = MONTH_IN_MINUTES * 60 * 1000;
const STAKING_ID: LockIdentifier = *b"da/staki";

pub trait Config: frame_system::Config + SendTransactionTypes<Call<Self>> {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type ModuleId: Get<ModuleId>;

	/// Time used for computing era duration.
	///
	/// It is guaranteed to start being called from the first `on_finalize`. Thus value at genesis
	/// is not used.
	type UnixTime: UnixTime;

	/// Number of sessions per era.
	type SessionsPerEra: Get<SessionIndex>;

	/// Number of eras that staked funds must remain bonded for.
	type BondingDurationInEra: Get<EraIndex>;
	/// Number of eras that staked funds must remain bonded for.
	type BondingDurationInBlockNumber: Get<Self::BlockNumber>;

	/// Number of eras that slashes are deferred by, after computation.
	///
	/// This should be less than the bonding duration. Set to 0 if slashes
	/// should be applied immediately, without opportunity for intervention.
	type SlashDeferDuration: Get<EraIndex>;

	/// The origin which can cancel a deferred slash. Root can always do this.
	type SlashCancelOrigin: EnsureOrigin<Self::Origin>;

	/// Interface for interacting with a session module.
	type SessionInterface: self::SessionInterface<Self::AccountId>;

	/// Something that can estimate the next session change, accurately or as a best effort guess.
	type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;

	/// The number of blocks before the end of the era from which election submissions are allowed.
	///
	/// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
	/// be used.
	///
	/// This is bounded by being within the last session. Hence, setting it to a value more than the
	/// length of a session will be pointless.
	type ElectionLookahead: Get<Self::BlockNumber>;

	/// The overarching call type.
	type Call: Dispatchable + From<Call<Self>> + IsSubType<Call<Self>> + Clone;

	/// Maximum number of balancing iterations to run in the offchain submission.
	///
	/// If set to 0, balance_solution will not be executed at all.
	type MaxIterations: Get<u32>;

	/// The threshold of improvement that should be provided for a new solution to be accepted.
	type MinSolutionScoreBump: Get<Perbill>;

	/// The maximum number of nominators rewarded for each validator.
	///
	/// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
	/// their reward. This used to limit the i/o cost for the nominator payout.
	type MaxNominatorRewardedPerValidator: Get<u32>;

	/// A configuration for base priority of unsigned transactions.
	///
	/// This is exposed so that it can be tuned for particular runtime, when
	/// multiple pallets send unsigned transactions.
	type UnsignedPriority: Get<TransactionPriority>;

	/// Maximum weight that the unsigned transaction can have.
	///
	/// Chose this value with care. On one hand, it should be as high as possible, so the solution
	/// can contain as many nominators/validators as possible. On the other hand, it should be small
	/// enough to fit in the block.
	type OffchainSolutionWeightLimit: Get<Weight>;

	/// The *ETP* currency.
	type EtpCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	/// Tokens have been minted and are unused for validator-reward.
	/// See [Era payout](./index.html#era-payout).
	type EtpRewardRemainder: OnUnbalanced<EtpNegativeImbalance<Self>>;
	/// Handler for the unbalanced *ETP* reduction when slashing a staker.
	type EtpSlash: OnUnbalanced<EtpNegativeImbalance<Self>>;
	/// Handler for the unbalanced *ETP* increment when rewarding a staker.
	type EtpReward: OnUnbalanced<EtpPositiveImbalance<Self>>;

	/// The *DNA* currency.
	type DnaCurrency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	// FIXME: Ugly hack due to https://github.com/rust-lang/rust/issues/31844#issuecomment-557918823
	/// Handler for the unbalanced *DNA* reduction when slashing a staker.
	type DnaSlash: OnUnbalancedDna<DnaNegativeImbalance<Self>>;
	/// Handler for the unbalanced *DNA* increment when rewarding a staker.
	type DnaReward: OnUnbalanced<DnaPositiveImbalance<Self>>;

	/// Hyperspace's hard cap default 10_000_000_000 * 10^9
	type Cap: Get<EtpBalance<Self>>;
	/// Hyperspace's staking vote default 1_000_000_000
	type TotalPower: Get<Power>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

/// Means for interacting with a specialized version of the `session` trait.
///
/// This is needed because `Staking` sets the `ValidatorIdOf` of the `pallet_session::Config`
pub trait SessionInterface<AccountId>: frame_system::Config {
	/// Disable a given validator by stash ID.
	///
	/// Returns `true` if new era should be forced at the end of this session.
	/// This allows preventing a situation where there is too many validators
	/// disabled and block production stalls.
	fn disable_validator(validator: &AccountId) -> Result<bool, ()>;
	/// Get the validators from session.
	fn validators() -> Vec<AccountId>;
	/// Prune historical session tries up to but not including the given index.
	fn prune_historical_up_to(up_to: SessionIndex);
}
impl<T: Config> SessionInterface<AccountId<T>> for T
where
	T: pallet_session::Config<ValidatorId = AccountId<T>>,
	T: pallet_session::historical::Config<
		FullIdentification = Exposure<AccountId<T>, EtpBalance<T>, DnaBalance<T>>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<AccountId<T>>,
	T::SessionManager: pallet_session::SessionManager<AccountId<T>>,
	T::ValidatorIdOf: Convert<AccountId<T>, Option<AccountId<T>>>,
{
	fn disable_validator(validator: &AccountId<T>) -> Result<bool, ()> {
		<pallet_session::Module<T>>::disable(validator)
	}

	fn validators() -> Vec<AccountId<T>> {
		<pallet_session::Module<T>>::validators()
	}

	fn prune_historical_up_to(up_to: SessionIndex) {
		<pallet_session::historical::Module<T>>::prune_up_to(up_to);
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = AccountId<T>,
		BlockNumber = BlockNumber<T>,
		EtpBalance = EtpBalance<T>,
		DnaBalance = DnaBalance<T>,
	{
		/// The era payout has been set; the first balance is the validator-payout; the second is
		/// the remainder from the maximum amount of reward.
		/// [era_index, validator_payout, remainder]
		EraPayout(EraIndex, EtpBalance, EtpBalance),

		/// The staker has been rewarded by this amount. [stash, amount]
		Reward(AccountId, EtpBalance),

		/// One validator (and its nominators) has been slashed by the given amount.
		/// [validator, amount, amount]
		Slash(AccountId, EtpBalance, DnaBalance),
		/// An old slashing report from a prior era was discarded because it could
		/// not be processed. [session_index]
		OldSlashingReportDiscarded(SessionIndex),

		/// A new set of stakers was elected with the given [compute].
		StakingElection(ElectionCompute),

		/// A new solution for the upcoming election has been stored. [compute]
		SolutionStored(ElectionCompute),

		/// An account has bonded this amount. [amount, start, end]
		///
		/// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
		/// it will not be emitted for staking rewards when they are added to stake.
		BondEtp(EtpBalance, TsInMs, TsInMs),
		/// An account has bonded this amount. [amount, start, end]
		///
		/// NOTE: This event is only emitted when funds are bonded via a dispatchable. Notably,
		/// it will not be emitted for staking rewards when they are added to stake.
		BondDna(DnaBalance),

		/// An account has unbonded this amount. [amount, now]
		UnbondEtp(EtpBalance, BlockNumber),
		/// An account has unbonded this amount. [amount, now]
		UnbondDna(DnaBalance, BlockNumber),

		/// A nominator has been kicked from a validator. \[nominator, stash\]
		Kicked(AccountId, AccountId),

		/// Someone claimed his deposits. [stash]
		DepositsClaimed(AccountId),
		/// Someone claimed his deposits with some *DNA*s punishment. [stash, forfeit]
		DepositsClaimedWithPunish(AccountId, DnaBalance),
	}
);

decl_error! {
	/// Error for the staking module.
	pub enum Error for Module<T: Config> {
		/// Not a controller account.
		NotController,
		/// Not a stash account.
		NotStash,
		/// Stash is already bonded.
		AlreadyBonded,
		/// Controller is already paired.
		AlreadyPaired,
		/// Targets cannot be empty.
		EmptyTargets,
		/// Duplicate index.
		DuplicateIndex,
		/// Slash record index out of bounds.
		InvalidSlashIndex,
		/// Can not bond with value less than minimum balance.
		InsufficientValue,
		/// Can not schedule more unlock chunks.
		NoMoreChunks,
		/// Can not rebond without unlocking chunks.
		NoUnlockChunk,
		/// Attempting to target a stash that still has funds.
		FundedTarget,
		/// Invalid era to reward.
		InvalidEraToReward,
		/// Invalid number of nominations.
		InvalidNumberOfNominations,
		/// Items are not sorted and unique.
		NotSortedAndUnique,
		/// Rewards for this era have already been claimed for this validator.
		AlreadyClaimed,
		/// The submitted result is received out of the open window.
		OffchainElectionEarlySubmission,
		/// The submitted result is not as good as the one stored on chain.
		OffchainElectionWeakSubmission,
		/// The snapshot data of the current window is missing.
		SnapshotUnavailable,
		/// Incorrect number of winners were presented.
		OffchainElectionBogusWinnerCount,
		/// One of the submitted winners is not an active candidate on chain (index is out of range
		/// in snapshot).
		OffchainElectionBogusWinner,
		/// Error while building the assignment type from the compact. This can happen if an index
		/// is invalid, or if the weights _overflow_.
		OffchainElectionBogusCompact,
		/// One of the submitted nominators is not an active nominator on chain.
		OffchainElectionBogusNominator,
		/// One of the submitted nominators has an edge to which they have not voted on chain.
		OffchainElectionBogusNomination,
		/// One of the submitted nominators has an edge which is submitted before the last non-zero
		/// slash of the target.
		OffchainElectionSlashedNomination,
		/// A self vote must only be originated from a validator to ONLY themselves.
		OffchainElectionBogusSelfVote,
		/// The submitted result has unknown edges that are not among the presented winners.
		OffchainElectionBogusEdge,
		/// The claimed score does not match with the one computed from the data.
		OffchainElectionBogusScore,
		/// The election size is invalid.
		OffchainElectionBogusElectionSize,
		/// The call is not allowed at the given time due to restrictions of election period.
		CallNotAllowed,
		/// Incorrect previous history depth input provided.
		IncorrectHistoryDepth,
		/// Incorrect number of slashing spans provided.
		IncorrectSlashingSpans,
		/// Internal state has become somehow corrupted and the operation cannot continue.
		BadState,
		/// Too many nomination targets supplied.
		TooManyTargets,
		/// A nomination target was supplied that was blocked or otherwise not a validator.
		BadTarget,
		/// Payout - INSUFFICIENT
		PayoutIns,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as HyperspaceStaking {
		/// Number of eras to keep in history.
		///
		/// Information is kept for eras in `[current_era - history_depth; current_era]`.
		///
		/// Must be more than the number of eras delayed by session otherwise. I.e. active era must
		/// always be in history. I.e. `active_era > current_era - history_depth` must be
		/// guaranteed.
		HistoryDepth get(fn history_depth) config(): u32 = 336;

		/// The ideal number of staking participants.
		pub ValidatorCount get(fn validator_count) config(): u32;

		/// Minimum number of staking participants before emergency conditions are imposed.
		pub MinimumValidatorCount get(fn minimum_validator_count) config(): u32;

		/// Any validators that may never be slashed or forcibly kicked. It's a Vec since they're
		/// easy to initialize and the performance hit is minimal (we expect no more than four
		/// invulnerables) and restricted to testnets.
		pub Invulnerables get(fn invulnerables) config(): Vec<T::AccountId>;

		/// Map from all locked "stash" accounts to the controller account.
		pub Bonded get(fn bonded): map hasher(twox_64_concat) T::AccountId => Option<T::AccountId>;

		/// Map from all (unlocked) "controller" accounts to the info regarding the staking.
		pub Ledger get(fn ledger): map hasher(blake2_128_concat) T::AccountId => Option<StakingLedgerT<T>>;

		/// Where the reward payment should be made. Keyed by stash.
		pub Payee get(fn payee): map hasher(twox_64_concat) T::AccountId => RewardDestination<T::AccountId>;

		/// The map from (wannabe) validator stash key to the preferences of that validator.
		pub Validators
			get(fn validators)
			: map hasher(twox_64_concat) T::AccountId => ValidatorPrefs;

		/// The map from nominator stash key to the set of stash keys of all validators to nominate.
		pub Nominators
			get(fn nominators)
			: map hasher(twox_64_concat) T::AccountId => Option<Nominations<T::AccountId>>;

		/// The current era index.
		///
		/// This is the latest planned era, depending on how the Session pallet queues the validator
		/// set, it might be active or not.
		pub CurrentEra get(fn current_era): Option<EraIndex>;

		/// The active era information, it holds index and start.
		///
		/// The active era is the era being currently rewarded. Validator set of this era must be
		/// equal to [`SessionInterface::validators`].
		pub ActiveEra get(fn active_era): Option<ActiveEraInfo>;

		/// The session index at which the era start for the last `HISTORY_DEPTH` eras.
		///
		/// Note: This tracks the starting session (i.e. session index when era start being active)
		/// for the eras in `[CurrentEra - HISTORY_DEPTH, CurrentEra]`.
		pub ErasStartSessionIndex
			get(fn eras_start_session_index)
			: map hasher(twox_64_concat) EraIndex => Option<SessionIndex>;

		/// Exposure of validator at era.
		///
		/// This is keyed first by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakers
			get(fn eras_stakers)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> ExposureT<T>;

		/// Clipped Exposure of validator at era.
		///
		/// This is similar to [`ErasStakers`] but number of nominators exposed is reduced to the
		/// `T::MaxNominatorRewardedPerValidator` biggest stakers.
		/// This is used to limit the i/o cost for the nominator payout.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		/// If stakers hasn't been set or has been removed then empty exposure is returned.
		pub ErasStakersClipped
			get(fn eras_stakers_clipped)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> ExposureT<T>;

		/// Similar to `ErasStakers`, this holds the preferences of validators.
		///
		/// This is keyed fist by the era index to allow bulk deletion and then the stash account.
		///
		/// Is it removed after `HISTORY_DEPTH` eras.
		// If prefs hasn't been set or has been removed then 0 commission is returned.
		pub ErasValidatorPrefs
			get(fn eras_validator_prefs)
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> ValidatorPrefs;

		/// The total validator era payout for the last `HISTORY_DEPTH` eras.
		///
		/// Eras that haven't finished yet or has been removed doesn't have reward.
		pub ErasValidatorReward
			get(fn eras_validator_reward)
			: map hasher(twox_64_concat) EraIndex => Option<EtpBalance<T>>;

		/// Rewards for the last `HISTORY_DEPTH` eras.
		/// If reward hasn't been set or has been removed then 0 reward is returned.
		pub ErasRewardPoints
			get(fn eras_reward_points)
			: map hasher(twox_64_concat) EraIndex => EraRewardPoints<T::AccountId>;

		/// The total amount staked for the last `HISTORY_DEPTH` eras.
		/// If total hasn't been set or has been removed then 0 stake is returned.
		pub ErasTotalStake
			get(fn eras_total_stake)
			: map hasher(twox_64_concat) EraIndex => Power;

		/// Mode of era forcing.
		pub ForceEra get(fn force_era) config(): Forcing;

		/// The percentage of the slash that is distributed to reporters.
		///
		/// The rest of the slashed value is handled by the `Slash`.
		pub SlashRewardFraction get(fn slash_reward_fraction) config(): Perbill;

		/// The amount of currency given to reporters of a slash event which was
		/// canceled by extraordinary circumstances (e.g. governance).
		pub CanceledSlashPayout get(fn canceled_payout) config(): Power;

		/// All unapplied slashes that are queued for later.
		pub UnappliedSlashes
			: map hasher(twox_64_concat) EraIndex
			=> Vec<UnappliedSlash<T::AccountId, EtpBalance<T>, DnaBalance<T>>>;

		/// A mapping from still-bonded eras to the first session index of that era.
		///
		/// Must contains information for eras for the range:
		/// `[active_era - bounding_duration; active_era]`
		BondedEras: Vec<(EraIndex, SessionIndex)>;

		/// All slashing events on validators, mapped by era to the highest slash proportion
		/// and slash value of the era.
		ValidatorSlashInEra
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> Option<(Perbill, slashing::RKT<T>)>;

		/// All slashing events on nominators, mapped by era to the highest slash value of the era.
		NominatorSlashInEra
			: double_map hasher(twox_64_concat) EraIndex, hasher(twox_64_concat) T::AccountId
			=> Option<slashing::RKT<T>>;

		/// Slashing spans for stash accounts.
		SlashingSpans
			get(fn slashing_spans)
			: map hasher(twox_64_concat) T::AccountId
			=> Option<slashing::SlashingSpans>;

		/// Records information about the maximum slash of a stash within a slashing span,
		/// as well as how much reward has been paid out.
		SpanSlash
			: map hasher(twox_64_concat) (T::AccountId, slashing::SpanIndex)
			=> slashing::SpanRecord<EtpBalance<T>, DnaBalance<T>>;

		/// The earliest era for which we have a pending, unapplied slash.
		EarliestUnappliedSlash: Option<EraIndex>;

		/// Snapshot of validators at the beginning of the current election window. This should only
		/// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
		pub SnapshotValidators get(fn snapshot_validators): Option<Vec<T::AccountId>>;

		/// Snapshot of nominators at the beginning of the current election window. This should only
		/// have a value when [`EraElectionStatus`] == `ElectionStatus::Open(_)`.
		pub SnapshotNominators get(fn snapshot_nominators): Option<Vec<T::AccountId>>;

		/// The next validator set. At the end of an era, if this is available (potentially from the
		/// result of an offchain worker), it is immediately used. Otherwise, the on-chain election
		/// is executed.
		pub QueuedElected get(fn queued_elected): Option<ElectionResultT<T>>;

		/// The score of the current [`QueuedElected`].
		pub QueuedScore get(fn queued_score): Option<ElectionScore>;

		/// Flag to control the execution of the offchain election. When `Open(_)`, we accept
		/// solutions to be submitted.
		pub EraElectionStatus get(fn era_election_status): ElectionStatus<T::BlockNumber>;

		/// True if the current **planned** session is final. Note that this does not take era
		/// forcing into account.
		pub IsCurrentSessionFinal get(fn is_current_session_final): bool = false;

		/// This is set to v5.0.0 for new networks.
		StorageVersion build(|_: &GenesisConfig<T>| Releases::V5_0_0): Releases;

		/// The chain's running time form genesis in milliseconds,
		/// use for calculate hyperspace era payout
		pub LivingTime get(fn living_time): TsInMs;

		/// The percentage of the total payout that is distributed to validators and nominators
		///
		/// The reset might go to Treasury or something else.
		pub PayoutFraction get(fn payout_fraction) config(): Perbill;

		/// Total *ETP* in pool.
		pub EtpPool get(fn etp_pool): EtpBalance<T>;
		/// Total *DNA* in pool.
		pub DnaPool get(fn dna_pool): DnaBalance<T>;
	}
	add_extra_genesis {
		config(stakers): Vec<(
			T::AccountId,
			T::AccountId,
			EtpBalance<T>,
			StakerStatus<T::AccountId>
		)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref stash, ref controller, etp_to_be_bonded, ref status) in &config.stakers {
				assert!(
					T::EtpCurrency::free_balance(&stash) >= etp_to_be_bonded,
					"Stash does not have enough balance to bond.",
				);
				let _ = <Module<T>>::bond(
					T::Origin::from(Some(stash.to_owned()).into()),
					T::Lookup::unlookup(controller.to_owned()),
					StakingBalance::EtpBalance(etp_to_be_bonded),
					RewardDestination::Staked,
					0,
				);
				let _ = match status {
					StakerStatus::Validator => {
						<Module<T>>::validate(
							T::Origin::from(Some(controller.to_owned()).into()),
							Default::default(),
						)
					},
					StakerStatus::Nominator(votes) => {
						<Module<T>>::nominate(
							T::Origin::from(Some(controller.to_owned()).into()),
							votes.iter().map(|l| T::Lookup::unlookup(l.to_owned())).collect(),
						)
					}, _ => Ok(())
				};
				let _ = T::EtpCurrency::make_free_balance_be(
					&<Module<T>>::account_id(),
					T::EtpCurrency::minimum_balance(),
				);
			}
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		const ModuleId: ModuleId = T::ModuleId::get();

		/// Number of sessions per era.
		const SessionsPerEra: SessionIndex = T::SessionsPerEra::get();

		/// Number of eras that staked funds must remain bonded for.
		const BondingDurationInEra: EraIndex = T::BondingDurationInEra::get();
		/// Number of BlockNumbers that staked funds must remain bonded for.
		const BondingDurationInBlockNumber: T::BlockNumber = T::BondingDurationInBlockNumber::get();

		/// Number of eras that slashes are deferred by, after computation.
		///
		/// This should be less than the bonding duration.
		/// Set to 0 if slashes should be applied immediately, without opportunity for
		/// intervention.
		const SlashDeferDuration: EraIndex = T::SlashDeferDuration::get();

		/// The number of blocks before the end of the era from which election submissions are allowed.
		///
		/// Setting this to zero will disable the offchain compute and only on-chain seq-phragmen will
		/// be used.
		///
		/// This is bounded by being within the last session. Hence, setting it to a value more than the
		/// length of a session will be pointless.
		const ElectionLookahead: T::BlockNumber = T::ElectionLookahead::get();

		/// Maximum number of balancing iterations to run in the offchain submission.
		///
		/// If set to 0, balance_solution will not be executed at all.
		const MaxIterations: u32 = T::MaxIterations::get();

		/// The threshold of improvement that should be provided for a new solution to be accepted.
		const MinSolutionScoreBump: Perbill = T::MinSolutionScoreBump::get();

		/// The maximum number of nominators rewarded for each validator.
		///
		/// For each validator only the `$MaxNominatorRewardedPerValidator` biggest stakers can claim
		/// their reward. This used to limit the i/o cost for the nominator payout.
		const MaxNominatorRewardedPerValidator: u32 = T::MaxNominatorRewardedPerValidator::get();

		/// Hyperspace's hard cap default 10_000_000_000 * 10^9
		const Cap: EtpBalance<T> = T::Cap::get();

		/// Hyperspace's staking vote default 1_000_000_000
		const TotalPower: Power = T::TotalPower::get();

		fn deposit_event() = default;

		fn on_runtime_upgrade() -> frame_support::weights::Weight {
			if StorageVersion::get() == Releases::V4_0_0 || !StorageVersion::exists() {
				log!(info, "Migrating Staking...");
				
				StorageVersion::put(Releases::V5_0_0);
				migrations::migrate_to_blockable::<T>()
			} else {
				0
			}
		}

		/// sets `ElectionStatus` to `Open(now)` where `now` is the block number at which the
		/// election window has opened, if we are at the last session and less blocks than
		/// `T::ElectionLookahead` is remaining until the next new session schedule. The offchain
		/// worker, if applicable, will execute at the end of the current block, and solutions may
		/// be submitted.
		fn on_initialize(now: T::BlockNumber) -> Weight {
			let mut consumed_weight = 0;
			let mut add_weight = |reads, writes, weight| {
				consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
				consumed_weight += weight;
			};
			if
				// if we don't have any ongoing offchain compute.
				Self::era_election_status().is_closed() &&
				// either current session final based on the plan, or we're forcing.
				(Self::is_current_session_final() || Self::will_era_be_forced())
			{
				if let Some(next_session_change) = T::NextNewSession::estimate_next_new_session(now) {
					if let Some(remaining) = next_session_change.checked_sub(&now) {
						if remaining <= T::ElectionLookahead::get() && !remaining.is_zero() {
							// create snapshot.
							let (did_snapshot, snapshot_weight) = Self::create_stakers_snapshot();
							add_weight(0, 0, snapshot_weight);
							if did_snapshot {
								// Set the flag to make sure we don't waste any compute here in the same era
								// after we have triggered the offline compute.
								<EraElectionStatus<T>>::put(
									ElectionStatus::<T::BlockNumber>::Open(now)
								);
								add_weight(0, 1, 0);
								log!(
									info,
									"💸 Election window is Open({:?}). Snapshot created",
									now
								);
							} else {
								log!(warn, "💸 Failed to create snapshot at {:?}.", now);
							}
						}
					}
				} else {
					log!(warn, "💸 Estimating next session change failed.");
				}
				add_weight(0, 0, T::NextNewSession::weight(now))
			}

			// For `era_election_status`, `is_current_session_final`, `will_era_be_forced`
			add_weight(3, 0, 0);
			// Additional read from `on_finalize`
			add_weight(1, 0, 0);
			consumed_weight
		}

		/// Check if the current block number is the one at which the election window has been set
		/// to open. If so, it runs the offchain worker code.
		fn offchain_worker(now: T::BlockNumber) {
			// --- hyperspace ---
			use offchain_election::{set_check_offchain_execution_status, compute_offchain_election};

			if Self::era_election_status().is_open_at(now) {
				let offchain_status = set_check_offchain_execution_status::<T>(now);
				if let Err(why) = offchain_status {
					log!(
						warn,
						"💸 skipping offchain worker in open election window due to [{}]",
						why
					);
				} else {
					if let Err(e) = compute_offchain_election::<T>() {
						log!(error, "💸 Error in election offchain worker: {:?}", e);
					} else {
						log!(debug, "💸 Executed offchain worker thread without errors.");
					}
				}

			}
		}

		fn on_finalize() {
			if let Some(mut active_era) = Self::active_era() {
				if active_era.start.is_none() {
					let now = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();
					active_era.start = Some(now);
					// This write only ever happens once, we don't include it in the weight in general
					ActiveEra::put(active_era);
				}
			}
			// `on_finalize` weight is tracked in `on_initialize`
		}

		fn integrity_test() {
			sp_io::TestExternalities::new_empty().execute_with(||
				assert!(
					T::SlashDeferDuration::get()
						< T::BondingDurationInEra::get()
						|| T::BondingDurationInEra::get()
						== 0,
					"As per documentation, \
					slash defer duration ({}) should be less than bonding duration ({}).",
					T::SlashDeferDuration::get(),
					T::BondingDurationInEra::get(),
				)
			);

			use sp_runtime::UpperOf;
			// see the documentation of `Assignment::try_normalize`. Now we can ensure that this
			// will always return `Ok`.
			// 1. Maximum sum of Vec<ChainAccuracy> must fit into `UpperOf<ChainAccuracy>`.
			assert!(
				<usize as TryInto<UpperOf<ChainAccuracy>>>::try_into(MAX_NOMINATIONS)
				.unwrap()
				.checked_mul(<ChainAccuracy>::one().deconstruct().try_into().unwrap())
				.is_some()
			);

			// 2. Maximum sum of Vec<OffchainAccuracy> must fit into `UpperOf<OffchainAccuracy>`.
			assert!(
				<usize as TryInto<UpperOf<OffchainAccuracy>>>::try_into(MAX_NOMINATIONS)
				.unwrap()
				.checked_mul(<OffchainAccuracy>::one().deconstruct().try_into().unwrap())
				.is_some()
			);
		}

		/// Take the origin account as a stash and lock up `value` of its balance. `controller` will
		/// be the account that controls it.
		///
		/// `value` must be more than the `minimum_balance` specified by `T::Currency`.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash account.
		///
		/// # <weight>
		/// - Independent of the arguments. Moderate complexity.
		/// - O(1).
		/// - Three extra DB entries.
		///
		/// NOTE: Two of the storage writes (`Self::bonded`, `Self::payee`) are _never_ cleaned
		/// unless the `origin` falls below _existential deposit_ and gets removed as dust.
		/// ------------------
		/// Weight: O(1)
		/// DB Weight:
		/// - Read: Bonded, Ledger, [Origin Account], Current Era, History Depth, Locks
		/// - Write: Bonded, Payee, [Origin Account], Locks, Ledger
		/// # </weight>
		#[weight = T::WeightInfo::bond()]
		fn bond(
			origin,
			controller: <T::Lookup as StaticLookup>::Source,
			value: StakingBalanceT<T>,
			payee: RewardDestination<T::AccountId>,
			promise_month: u8
		) {
			let stash = ensure_signed(origin)?;
			ensure!(!<Bonded<T>>::contains_key(&stash), <Error<T>>::AlreadyBonded);

			let controller = T::Lookup::lookup(controller)?;
			ensure!(!<Ledger<T>>::contains_key(&controller), <Error<T>>::AlreadyPaired);

			match value {
				StakingBalance::EtpBalance(value) => {
					// reject a bond which is considered to be _dust_.
					ensure!(
						value >= T::EtpCurrency::minimum_balance(),
						<Error<T>>::InsufficientValue,
					);
				}
				StakingBalance::DnaBalance(value) => {
					// reject a bond which is considered to be _dust_.
					ensure!(
						value >= T::DnaCurrency::minimum_balance(),
						<Error<T>>::InsufficientValue,
					);
				}
			}

			<frame_system::Module<T>>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

			// You're auto-bonded forever, here. We might improve this by only bonding when
			// you actually validate/nominate and remove once you unbond __everything__.
			<Bonded<T>>::insert(&stash, &controller);
			<Payee<T>>::insert(&stash, payee);

			let ledger = StakingLedger {
				stash: stash.clone(),
				claimed_rewards: {
					let current_era = CurrentEra::get().unwrap_or(0);
					let last_reward_era = current_era.saturating_sub(Self::history_depth());
					(last_reward_era..current_era).collect()
				},
				..Default::default()
			};

			match value {
				StakingBalance::EtpBalance(value) => {
					let stash_balance = T::EtpCurrency::free_balance(&stash);
					let value = value.min(stash_balance);
					let promise_month = promise_month.min(36);
					let (start_time, expire_time) = Self::bond_etp(
						&stash,
						&controller,
						value,
						promise_month,
						ledger,
					)?;

					Self::deposit_event(RawEvent::BondEtp(value, start_time, expire_time));
				},
				StakingBalance::DnaBalance(value) => {
					let stash_balance = T::DnaCurrency::free_balance(&stash);
					let value = value.min(stash_balance);

					Self::bond_dna(&controller, value, ledger)?;
					Self::deposit_event(RawEvent::BondDna(value));
				},
			}
		}

		/// Add some extra amount that have appeared in the stash `free_balance` into the balance up
		/// for staking.
		///
		/// Use this if there are additional funds in your stash account that you wish to bond.
		/// Unlike [`bond`] or [`unbond`] this function does not impose any limitation on the amount
		/// that can be added.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller and
		/// it can be only called when [`EraElectionStatus`] is `Closed`.
		///
		/// Emits `Bonded`.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - O(1).
		/// - One DB entry.
		/// ------------
		/// DB Weight:
		/// - Read: Era Election Status, Bonded, Ledger, [Origin Account], Locks
		/// - Write: [Origin Account], Locks, Ledger
		/// # </weight>
		#[weight = T::WeightInfo::bond_extra()]
		fn bond_extra(origin, max_additional: StakingBalanceT<T>, promise_month: u8) {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let stash = ensure_signed(origin)?;
			let controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let promise_month = promise_month.min(36);

			match max_additional {
				 StakingBalance::EtpBalance(max_additional) => {
					let stash_balance = T::EtpCurrency::free_balance(&stash);

					if let Some(extra) = stash_balance.checked_sub(
						&ledger.etp_locked_amount_at(<frame_system::Module<T>>::block_number())
					) {
						let extra = extra.min(max_additional);
						let (start_time, expire_time) = Self::bond_etp(
							&stash,
							&controller,
							extra,
							promise_month,
							ledger,
						)?;

						Self::deposit_event(RawEvent::BondEtp(extra, start_time, expire_time));
					}
				},
				StakingBalance::DnaBalance(max_additional) => {
					let stash_balance = T::DnaCurrency::free_balance(&stash);

					if let Some(extra) = stash_balance.checked_sub(
						&ledger.dna_locked_amount_at(<frame_system::Module<T>>::block_number())
					) {
						let extra = extra.min(max_additional);

						Self::bond_dna(&controller, extra, ledger)?;
						Self::deposit_event(RawEvent::BondDna(extra));
					}
				},
			}
		}

		/// Deposit some extra amount ring, and return dna to the controller.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// Is a no-op if value to be deposited is zero.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - O(1).
		/// - One DB entry.
		/// ------------
		/// DB Weight:
		/// - Read: Era Election Status, Bonded, Ledger, [Origin Account]
		/// - Write: [Origin Account], Ledger
		/// # </weight>
		#[weight = T::WeightInfo::deposit_extra()]
		fn deposit_extra(origin, value: EtpBalance<T>, promise_month: u8) {
			let stash = ensure_signed(origin)?;
			let controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;

			if value.is_zero() {
				return Ok(());
			}

			let start_time = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();
			let promise_month = promise_month.max(1).min(36);
			let expire_time = start_time + promise_month as TsInMs * MONTH_IN_MILLISECONDS;
			let mut ledger = Self::clear_mature_deposits(ledger).0;
			let StakingLedger {
				stash,
				active_etp,
				active_deposit_etp,
				deposit_items,
				..
			} = &mut ledger;
			let value = value.min(active_etp.saturating_sub(*active_deposit_etp));

			if value.is_zero() {
				return Ok(());
			}

			let dna_return = inflation::compute_dna_reward::<T>(value, promise_month);
			let dna_positive_imbalance = T::DnaCurrency::deposit_creating(&stash, dna_return);

			T::DnaReward::on_unbalanced(dna_positive_imbalance);
			*active_deposit_etp = active_deposit_etp.saturating_add(value);
			deposit_items.push(TimeDepositItem {
				value,
				start_time,
				expire_time,
			});

			<Ledger<T>>::insert(&controller, ledger);
			Self::deposit_event(RawEvent::BondEtp(value, start_time, expire_time));
		}

		/// Schedule a portion of the stash to be unlocked ready for transfer out after the bond
		/// period ends. If this leaves an amount actively bonded less than
		/// T::Currency::minimum_balance(), then it is increased to the full amount.
		///
		/// Once the unlock period is done, the funds will be withdrew automatically and ready for transfer.
		///
		/// No more than a limited number of unlocking chunks (see `MAX_UNLOCKING_CHUNKS`)
		/// can co-exists at the same time. In that case,  [`StakingLock::shrink`] need
		/// to be called first to remove some of the chunks (if possible).
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		/// And, it can be only called when [`EraElectionStatus`] is `Closed`.
		///
		/// After all pledged Etp and Dna are unbonded, the bonded accounts, namely stash and
		/// controller, will also be unbonded.  Once user want to bond again, the `bond` method
		/// should be called. If there are still pledged Etp or Dna and user want to bond more
		/// values, the `bond_extra` method should be called.
		///
		/// # <weight>
		/// - Independent of the arguments. Limited but potentially exploitable complexity.
		/// - Contains a limited number of reads.
		/// - Each call (requires the remainder of the bonded balance to be above `minimum_balance`)
		///   will cause a new entry to be inserted into a vector (`StakingLock.unbondings`) kept in storage.
		/// - One DB entry.
		/// ----------
		/// Weight: O(1)
		/// DB Weight:
		/// - Read: EraElectionStatus, Ledger, CurrentEra, Locks, BalanceOf Stash,
		/// - Write: Locks, Ledger, BalanceOf Stash,
		/// </weight>
		#[weight = T::WeightInfo::unbond()]
		fn unbond(origin, value: StakingBalanceT<T>) {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let controller = ensure_signed(origin)?;
			let mut ledger = Self::clear_mature_deposits(
					Self::ledger(&controller).ok_or(<Error<T>>::NotController)?
			).0;
			let StakingLedger {
				active_etp,
				active_deposit_etp,
				active_dna,
				etp_staking_lock,
				dna_staking_lock,
				..
			} = &mut ledger;
			let now = <frame_system::Module<T>>::block_number();

			etp_staking_lock.update(now);
			dna_staking_lock.update(now);

			// Due to the macro parser, we've to add a bracket.
			// Actually, this's totally wrong:
			//	 `a as u32 + b as u32 < c`
			// Workaround:
			//	 1. `(a as u32 + b as u32) < c`
			//	 2. `let c_ = a as u32 + b as u32; c_ < c`
			ensure!(
				(etp_staking_lock.unbondings.len() + dna_staking_lock.unbondings.len())
					< MAX_UNLOCKING_CHUNKS,
				<Error<T>>::NoMoreChunks,
			);

			let mut unbond_etp: EtpBalance<T> = Zero::zero();
			let mut unbond_dna: DnaBalance<T> = Zero::zero();

			match value {
				StakingBalance::EtpBalance(r) => {
					// Only active normal ring can be unbond:
					// `active_etp = active_normal_etp + active_deposit_etp`
					let active_normal_etp = *active_etp - *active_deposit_etp;
					unbond_etp = r.min(active_normal_etp);

					if !unbond_etp.is_zero() {
						*active_etp -= unbond_etp;

						// Avoid there being a dust balance left in the staking system.
						if (*active_etp < T::EtpCurrency::minimum_balance())
							&& (*active_dna < T::DnaCurrency::minimum_balance()) {
							unbond_etp += *active_etp;
							unbond_dna += *active_dna;

							*active_etp = Zero::zero();
							*active_dna = Zero::zero();
						}

						etp_staking_lock.unbondings.push(Unbonding {
							amount: unbond_etp,
							until: now + T::BondingDurationInBlockNumber::get(),
						});

						Self::deposit_event(RawEvent::UnbondEtp(unbond_etp, now));

						if !unbond_dna.is_zero() {
							dna_staking_lock.unbondings.push(Unbonding {
								amount: unbond_dna,
								until: now + T::BondingDurationInBlockNumber::get(),
							});

							Self::deposit_event(RawEvent::UnbondDna(unbond_dna, now));
						}
					}
				},
				StakingBalance::DnaBalance(k) => {
					unbond_dna = k.min(*active_dna);

					if !unbond_dna.is_zero() {
						*active_dna -= unbond_dna;

						// Avoid there being a dust balance left in the staking system.
						if (*active_dna < T::DnaCurrency::minimum_balance())
							&& (*active_etp < T::EtpCurrency::minimum_balance()) {
							unbond_dna += *active_dna;
							unbond_etp += *active_etp;

							*active_dna = Zero::zero();
							*active_etp = Zero::zero();
						}

						dna_staking_lock.unbondings.push(Unbonding {
							amount: unbond_dna,
							until: now + T::BondingDurationInBlockNumber::get(),
						});


						Self::deposit_event(RawEvent::UnbondDna(unbond_dna, now));

						if !unbond_etp.is_zero() {
							etp_staking_lock.unbondings.push(Unbonding {
								amount: unbond_etp,
								until: now + T::BondingDurationInBlockNumber::get(),
							});

							Self::deposit_event(RawEvent::UnbondEtp(unbond_etp, now));
						}
					}
				},
			}

			Self::update_ledger(&controller, &mut ledger);

			// TODO: https://github.com/new-mvs/darwinia-common/issues/96
			// FIXME: https://github.com/new-mvs/darwinia-common/issues/121
			// let StakingLedger {
			// 	active_etp,
			// 	active_dna,
			// 	..
			// } = ledger;

			// // All bonded *ETP* and *DNA* is withdrawing, then remove Ledger to save storage
			// if active_etp.is_zero() && active_dna.is_zero() {
			// 	//
			// 	// `OnKilledAccount` would be a method to collect the locks.
			// 	//
			// 	// These locks are still in the system, and should be removed after 14 days
			// 	//
			// 	// There two situations should be considered after the 14 days
			// 	// - the user never bond again, so the locks should be released.
			// 	// - the user is bonded again in the 14 days, so the after 14 days
			// 	//   the lock should not be removed
			// 	//
			// 	// If the locks are not deleted, this lock will waste the storage in the future
			// 	// blocks.
			// 	//
			// 	// T::Etp::remove_lock(STAKING_ID, &stash);
			// 	// T::Dna::remove_lock(STAKING_ID, &stash);
			// 	// Self::kill_stash(&stash)?;
			// }
		}

		/// Stash accounts can get their ring back after the depositing time exceeded,
		/// and the ring getting back is still in staking status.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - One storage read.
		/// - One storage write.
		/// - Writes are limited to the `origin` account key.
		/// ----------
		/// DB Weight:
		/// - Read: Ledger, [Origin Account]
		/// - Write: [Origin Account], Ledger
		/// # </weight>
		#[weight = 50 * WEIGHT_PER_MICROS + T::DbWeight::get().reads_writes(2, 2)]
		fn claim_mature_deposits(origin) {
			let controller = ensure_signed(origin)?;
			let (ledger, mutated) = Self::clear_mature_deposits(
				Self::ledger(&controller).ok_or(<Error<T>>::NotController)?
			);

			if mutated {
				<Ledger<T>>::insert(controller, ledger);
			}
		}

		/// Claim deposits while the depositing time has not been exceeded, the ring
		/// will not be slashed, but the account is required to pay DNA as punish.
		///
		/// Refer to https://talk.hyperspace.network/topics/55
		///
		/// Assume the `expire_time` is a unique ID for the deposit
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - One storage read.
		/// - One storage write.
		/// - Writes are limited to the `origin` account key.
		/// ----------
		/// DB Weight:
		/// - Read: Ledger, Locks, [Origin Account]
		/// - Write: [Origin Account], Locks, Ledger
		/// # </weight>
		#[weight = 50 * WEIGHT_PER_MICROS + T::DbWeight::get().reads_writes(3, 2)]
		fn try_claim_deposits_with_punish(origin, expire_time: TsInMs) {
			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let now = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();

			if expire_time <= now {
				return Ok(());
			}

			let mut claim_deposits_with_punish = (false, Zero::zero());

			{
				let StakingLedger {
					stash,
					active_deposit_etp,
					deposit_items,
					..
				} = &mut ledger;

				deposit_items.retain(|item| {
					if item.expire_time != expire_time {
						return true;
					}

					let dna_slash = {
						let plan_duration_in_months = {
							let plan_duration_in_milliseconds =
								item.expire_time.saturating_sub(item.start_time);

							plan_duration_in_milliseconds / MONTH_IN_MILLISECONDS
						};
						let passed_duration_in_months = {
							let passed_duration_in_milliseconds =
								now.saturating_sub(item.start_time);

							passed_duration_in_milliseconds / MONTH_IN_MILLISECONDS
						};

						(
							inflation::compute_dna_reward::<T>(
								item.value, plan_duration_in_months as _
							)
							-
							inflation::compute_dna_reward::<T>(
								item.value, passed_duration_in_months as _
							)
						).max(1u32.into()) * 3u32.into()
					};

					// check total free balance and locked one
					// strict on punishing in dna
					if T::DnaCurrency::usable_balance(stash) >= dna_slash {
						*active_deposit_etp = active_deposit_etp.saturating_sub(item.value);

						let imbalance = T::DnaCurrency::slash(stash, dna_slash).0;
						T::DnaSlash::on_unbalanced(imbalance);

						claim_deposits_with_punish = (true, dna_slash);

						false
					} else {
						true
					}
				});
			}

			<Ledger<T>>::insert(&controller, &ledger);

			if claim_deposits_with_punish.0 {
				Self::deposit_event(
					RawEvent::DepositsClaimedWithPunish(
						ledger.stash.clone(),
						claim_deposits_with_punish.1,
					));
			}
		}

		/// Declare the desire to validate for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		/// And, it can be only called when [`EraElectionStatus`] is `Closed`.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// -----------
		/// Weight: O(1)
		/// DB Weight:
		/// - Read: Era Election Status, Ledger
		/// - Write: Nominators, Validators
		/// # </weight>
		#[weight = T::WeightInfo::validate()]
		fn validate(origin, prefs: ValidatorPrefs) {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			<Nominators<T>>::remove(stash);
			<Validators<T>>::insert(stash, prefs);
		}

		/// Declare the desire to nominate `targets` for the origin controller.
		///
		/// Effects will be felt at the beginning of the next era. This can only be called when
		/// [`EraElectionStatus`] is `Closed`.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		/// And, it can be only called when [`EraElectionStatus`] is `Closed`.
		///
		/// # <weight>
		/// - The transaction's complexity is proportional to the size of `targets` (N)
		/// which is capped at CompactAssignments::LIMIT (MAX_NOMINATIONS).
		/// - Both the reads and writes follow a similar pattern.
		/// ---------
		/// Weight: O(N)
		/// where N is the number of targets
		/// DB Weight:
		/// - Reads: Era Election Status, Ledger, Current Era
		/// - Writes: Validators, Nominators
		/// # </weight>
		#[weight = T::WeightInfo::nominate(targets.len() as u32)]
		fn nominate(origin, targets: Vec<<T::Lookup as StaticLookup>::Source>) {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			ensure!(!targets.is_empty(), <Error<T>>::EmptyTargets);
			ensure!(targets.len() <= MAX_NOMINATIONS, Error::<T>::TooManyTargets);

			let old = <Nominators<T>>::get(stash).map_or_else(Vec::new, |x| x.targets);
			let targets = targets
				.into_iter()
				.map(|t| T::Lookup::lookup(t).map_err(DispatchError::from))
				.map(|n| n.and_then(|n| if old.contains(&n) || !<Validators<T>>::get(&n).blocked {
					Ok(n)
				} else {
					Err(<Error<T>>::BadTarget.into())
				}))
				.collect::<Result<Vec<T::AccountId>, _>>()?;
			let nominations = Nominations {
				targets,
				// initial nominations are considered submitted at era 0. See `Nominations` doc
				submitted_in: Self::current_era().unwrap_or(0),
				suppressed: false,
			};

			<Validators<T>>::remove(stash);
			<Nominators<T>>::insert(stash, &nominations);
		}

		/// Declare no desire to either validate or nominate.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		/// And, it can be only called when [`EraElectionStatus`] is `Closed`.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains one read.
		/// - Writes are limited to the `origin` account key.
		/// --------
		/// Weight: O(1)
		/// DB Weight:
		/// - Read: EraElectionStatus, Ledger
		/// - Write: Validators, Nominators
		/// # </weight>
		#[weight = T::WeightInfo::chill()]
		fn chill(origin) {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;

			Self::chill_stash(&ledger.stash);
		}

		/// (Re-)set the payment target for a controller.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// ---------
		/// - Weight: O(1)
		/// - DB Weight:
		///     - Read: Ledger
		///     - Write: Payee
		/// # </weight>
		#[weight = T::WeightInfo::set_payee()]
		fn set_payee(origin, payee: RewardDestination<T::AccountId>) {
			let controller = ensure_signed(origin)?;
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			<Payee<T>>::insert(stash, payee);
		}

		/// (Re-)set the controller of a stash.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the stash, not the controller.
		///
		/// # <weight>
		/// - Independent of the arguments. Insignificant complexity.
		/// - Contains a limited number of reads.
		/// - Writes are limited to the `origin` account key.
		/// ----------
		/// Weight: O(1)
		/// DB Weight:
		/// - Read: Bonded, Ledger New Controller, Ledger Old Controller
		/// - Write: Bonded, Ledger New Controller, Ledger Old Controller
		/// # </weight>
		#[weight = T::WeightInfo::set_controller()]
		fn set_controller(origin, controller: <T::Lookup as StaticLookup>::Source) {
			let stash = ensure_signed(origin)?;
			let old_controller = Self::bonded(&stash).ok_or(<Error<T>>::NotStash)?;
			let controller = T::Lookup::lookup(controller)?;

			ensure!(!<Ledger<T>>::contains_key(&controller), <Error<T>>::AlreadyPaired);

			if controller != old_controller {
				<Bonded<T>>::insert(&stash, &controller);
				if let Some(l) = <Ledger<T>>::take(&old_controller) {
					<Ledger<T>>::insert(&controller, l);
				}
			}
		}

		// --- root call ---

		/// Sets the ideal number of validators.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// Weight: O(1)
		/// Write: Validator Count
		/// # </weight>
		#[weight = T::WeightInfo::set_validator_count()]
		fn set_validator_count(origin, #[compact] new: u32) {
			ensure_root(origin)?;
			ValidatorCount::put(new);
		}

		/// Increments the ideal number of validators.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// Same as [`set_validator_count`].
		/// # </weight>
		#[weight = T::WeightInfo::set_validator_count()]
		fn increase_validator_count(origin, #[compact] additional: u32) {
			ensure_root(origin)?;
			ValidatorCount::mutate(|n| *n += additional);
		}

		/// Scale up the ideal number of validators by a factor.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// Same as [`set_validator_count`].
		/// # </weight>
		#[weight = T::WeightInfo::set_validator_count()]
		fn scale_validator_count(origin, factor: Percent) {
			ensure_root(origin)?;
			ValidatorCount::mutate(|n| *n += factor * *n);
		}

		/// Force there to be no new eras indefinitely.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// - No arguments.
		/// - Weight: O(1)
		/// - Write: ForceEra
		/// # </weight>
		#[weight = T::WeightInfo::force_no_eras()]
		fn force_no_eras(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNone);
		}

		/// Force there to be a new era at the end of the next session. After this, it will be
		/// reset to normal (non-forced) behaviour.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// - No arguments.
		/// - Weight: O(1)
		/// - Write ForceEra
		/// # </weight>
		#[weight = T::WeightInfo::force_new_era()]
		fn force_new_era(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceNew);
		}

		/// Set the validators who cannot be slashed (if any).
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// - O(V)
		/// - Write: Invulnerables
		/// # </weight>
		#[weight = T::WeightInfo::set_invulnerables(invulnerables.len() as u32)]
		fn set_invulnerables(origin, invulnerables: Vec<T::AccountId>) {
			ensure_root(origin)?;
			<Invulnerables<T>>::put(invulnerables);
		}

		/// Force a current staker to become completely unstaked, immediately.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// O(S) where S is the number of slashing spans to be removed
		/// Reads: Bonded, Slashing Spans, Account, Locks
		/// Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Account, Locks
		/// Writes Each: SpanSlash * S
		/// # </weight>
		#[weight = T::WeightInfo::force_unstake(*num_slashing_spans)]
		fn force_unstake(origin, stash: T::AccountId, num_slashing_spans: u32) {
			ensure_root(origin)?;

			// remove all staking-related information.
			Self::kill_stash(&stash, num_slashing_spans)?;

			// remove the lock.
			T::EtpCurrency::remove_lock(STAKING_ID, &stash);
			T::DnaCurrency::remove_lock(STAKING_ID, &stash);
		}

		/// Force there to be a new era at the end of sessions indefinitely.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// - Weight: O(1)
		/// - Write: ForceEra
		/// # </weight>
		#[weight = T::WeightInfo::force_new_era_always()]
		fn force_new_era_always(origin) {
			ensure_root(origin)?;
			ForceEra::put(Forcing::ForceAlways);
		}

		/// Cancel enactment of a deferred slash.
		///
		/// Can be called by the `T::SlashCancelOrigin`.
		///
		/// Parameters: era and indices of the slashes for that era to kill.
		///
		/// # <weight>
		/// Complexity: O(U + S)
		/// with U unapplied slashes weighted with U=1000
		/// and S is the number of slash indices to be canceled.
		/// - Read: Unapplied Slashes
		/// - Write: Unapplied Slashes
		/// # </weight>
		#[weight = T::WeightInfo::cancel_deferred_slash(slash_indices.len() as u32)]
		fn cancel_deferred_slash(origin, era: EraIndex, slash_indices: Vec<u32>) {
			T::SlashCancelOrigin::ensure_origin(origin)?;

			ensure!(!slash_indices.is_empty(), <Error<T>>::EmptyTargets);
			ensure!(is_sorted_and_unique(&slash_indices), <Error<T>>::NotSortedAndUnique);

			let mut unapplied = <Self as Store>::UnappliedSlashes::get(&era);
			let last_item = slash_indices[slash_indices.len() - 1];
			ensure!((last_item as usize) < unapplied.len(), <Error<T>>::InvalidSlashIndex);

			for (removed, index) in slash_indices.into_iter().enumerate() {
				let index = (index as usize) - removed;
				unapplied.remove(index);
			}

			<Self as Store>::UnappliedSlashes::insert(&era, &unapplied);
		}

		/// Pay out all the stakers behind a single validator for a single era.
		///
		/// - `validator_stash` is the stash account of the validator. Their nominators, up to
		///   `T::MaxNominatorRewardedPerValidator`, will also receive their rewards.
		/// - `era` may be any era between `[current_era - history_depth; current_era]`.
		///
		/// The origin of this call must be _Signed_. Any account can call this function, even if
		/// it is not one of the stakers.
		///
		/// This can only be called when [`EraElectionStatus`] is `Closed`.
		///
		/// # <weight>
		/// - Time complexity: at most O(MaxNominatorRewardedPerValidator).
		/// - Contains a limited number of reads and writes.
		/// -----------
		/// N is the Number of payouts for the validator (including the validator)
		/// Weight:
		/// - Reward Destination Staked: O(N)
		/// - Reward Destination Controller (Creating): O(N)
		/// DB Weight:
		/// - Read: EraElectionStatus, CurrentEra, HistoryDepth, ErasValidatorReward,
		///         ErasStakersClipped, ErasRewardPoints, ErasValidatorPrefs (8 items)
		/// - Read Each: Bonded, Ledger, Payee, Locks, System Account (5 items)
		/// - Write Each: System Account, Locks, Ledger (3 items)
		///
		///   NOTE: weights are assuming that payouts are made to alive stash account (Staked).
		///   Paying even a dead controller is cheaper weight-wise. We don't do any refunds here.
		/// # </weight>
		#[weight = T::WeightInfo::payout_stakers_alive_staked(T::MaxNominatorRewardedPerValidator::get())]
		fn payout_stakers(origin, validator_stash: T::AccountId, era: EraIndex) -> DispatchResult {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			ensure_signed(origin)?;
			Self::do_payout_stakers(validator_stash, era)
		}

		/// Rebond a portion of the stash scheduled to be unlocked.
		///
		/// The dispatch origin must be signed by the controller, and it can be only called when
		/// [`EraElectionStatus`] is `Closed`.
		///
		/// # <weight>
		/// - Time complexity: O(L), where L is unlocking chunks
		/// - Bounded by `MAX_UNLOCKING_CHUNKS`.
		/// - Storage changes: Can't increase storage, only decrease it.
		/// ---------------
		/// - DB Weight:
		///     - Reads: EraElectionStatus, Ledger, Locks, [Origin Account]
		///     - Writes: [Origin Account], Locks, Ledger
		/// # </weight>
		#[weight = T::WeightInfo::rebond(MAX_UNLOCKING_CHUNKS as u32)]
		fn rebond(
			origin,
			#[compact] plan_to_rebond_etp: EtpBalance<T>,
			#[compact] plan_to_rebond_dna: DnaBalance<T>
		) -> DispatchResultWithPostInfo {
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);

			let controller = ensure_signed(origin)?;
			let mut ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let now = <frame_system::Module<T>>::block_number();

			ledger.etp_staking_lock.update(now);
			ledger.dna_staking_lock.update(now);

			ensure!(
				!ledger.etp_staking_lock.unbondings.is_empty()
					|| !ledger.dna_staking_lock.unbondings.is_empty(),
				<Error<T>>::NoUnlockChunk
			);

			let origin_active_etp = ledger.active_etp;
			let origin_active_dna = ledger.active_dna;

			ledger.rebond(plan_to_rebond_etp, plan_to_rebond_dna);

			// last check: the new active amount of ledger must be more than ED.
			ensure!(
				ledger.active_etp >= T::EtpCurrency::minimum_balance()
					|| ledger.active_dna >= T::DnaCurrency::minimum_balance(),
				<Error<T>>::InsufficientValue
			);

			Self::update_ledger(&controller, &mut ledger);

			let rebond_etp = ledger.active_etp.saturating_sub(origin_active_etp);
			let rebond_dna = ledger.active_dna.saturating_sub(origin_active_dna);

			if !rebond_etp.is_zero() {
				let now = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();

				Self::deposit_event(RawEvent::BondEtp(rebond_etp, now, now));
			}
			if !rebond_dna.is_zero() {
				Self::deposit_event(RawEvent::BondDna(rebond_dna));
			}

			Ok(Some(
				35 * WEIGHT_PER_MICROS
					+ 50 * WEIGHT_PER_NANOS * (
						ledger.etp_staking_lock.unbondings.len() as Weight
							+ ledger.dna_staking_lock.unbondings.len() as Weight
					)
					+ T::DbWeight::get().reads_writes(3, 2)
			).into())
		}

		/// Set `HistoryDepth` value. This function will delete any history information
		/// when `HistoryDepth` is reduced.
		///
		/// Parameters:
		/// - `new_history_depth`: The new history depth you would like to set.
		/// - `era_items_deleted`: The number of items that will be deleted by this dispatch.
		///    This should report all the storage items that will be deleted by clearing old
		///    era history. Needed to report an accurate weight for the dispatch. Trusted by
		///    `Root` to report an accurate number.
		///
		/// Origin must be root.
		///
		/// # <weight>
		/// - E: Number of history depths removed, i.e. 10 -> 7 = 3
		/// - Weight: O(E)
		/// - DB Weight:
		///     - Reads: Current Era, History Depth
		///     - Writes: History Depth
		///     - Clear Prefix Each: Era Stakers, EraStakersClipped, ErasValidatorPrefs
		///     - Writes Each: ErasValidatorReward, ErasRewardPoints, ErasTotalStake, ErasStartSessionIndex
		/// # </weight>
		#[weight = T::WeightInfo::set_history_depth(*_era_items_deleted)]
		fn set_history_depth(
			origin,
			#[compact] new_history_depth: EraIndex,
			#[compact] _era_items_deleted: u32,
		) {
			ensure_root(origin)?;
			if let Some(current_era) = Self::current_era() {
				HistoryDepth::mutate(|history_depth| {
					let last_kept = current_era.checked_sub(*history_depth).unwrap_or(0);
					let new_last_kept = current_era.checked_sub(new_history_depth).unwrap_or(0);
					for era_index in last_kept..new_last_kept {
						Self::clear_era_information(era_index);
					}
					*history_depth = new_history_depth
				})
			}
		}

		/// Remove all data structure concerning a staker/stash once its balance is at the minimum.
		/// This is essentially equivalent to `withdraw_unbonded` except it can be called by anyone
		/// and the target `stash` must have no funds left beyond the ED.
		///
		/// This can be called from any origin.
		///
		/// - `stash`: The stash account to reap. Its balance must be zero.
		///
		/// # <weight>
		/// Complexity: O(S) where S is the number of slashing spans on the account.
		/// DB Weight:
		/// - Reads: Stash Account, Bonded, Slashing Spans, Locks
		/// - Writes: Bonded, Slashing Spans (if S > 0), Ledger, Payee, Validators, Nominators, Stash Account, Locks
		/// - Writes Each: SpanSlash * S
		/// # </weight>
		#[weight = T::WeightInfo::reap_stash(*num_slashing_spans)]
		fn reap_stash(_origin, stash: T::AccountId, num_slashing_spans: u32) {
			let total_etp = T::EtpCurrency::total_balance(&stash);
			let minimum_etp = T::EtpCurrency::minimum_balance();
			let total_dna = T::DnaCurrency::total_balance(&stash);
			let minimum_dna = T::DnaCurrency::minimum_balance();
			let at_minimum = (
				total_etp == minimum_etp
				&&
				total_dna <= minimum_dna
			) || (
				total_dna == minimum_dna
				&&
				total_etp <= minimum_etp
			);

			ensure!(at_minimum, <Error<T>>::FundedTarget);

			Self::kill_stash(&stash, num_slashing_spans)?;
			T::EtpCurrency::remove_lock(STAKING_ID, &stash);
			T::DnaCurrency::remove_lock(STAKING_ID, &stash);
		}

		/// Submit an election result to the chain. If the solution:
		///
		/// 1. is valid.
		/// 2. has a better score than a potentially existing solution on chain.
		///
		/// then, it will be _put_ on chain.
		///
		/// A solution consists of two pieces of data:
		///
		/// 1. `winners`: a flat vector of all the winners of the round.
		/// 2. `assignments`: the compact version of an assignment vector that encodes the edge
		///	weights.
		///
		/// Both of which may be computed using _phragmen_, or any other algorithm.
		///
		/// Additionally, the submitter must provide:
		///
		/// - The `score` that they claim their solution has.
		///
		/// Both validators and nominators will be represented by indices in the solution. The
		/// indices should respect the corresponding types ([`ValidatorIndex`] and
		/// [`NominatorIndex`]). Moreover, they should be valid when used to index into
		/// [`SnapshotValidators`] and [`SnapshotNominators`]. Any invalid index will cause the
		/// solution to be rejected. These two storage items are set during the election window and
		/// may be used to determine the indices.
		///
		/// A solution is valid if:
		///
		/// 0. It is submitted when [`EraElectionStatus`] is `Open`.
		/// 1. Its claimed score is equal to the score computed on-chain.
		/// 2. Presents the correct number of winners.
		/// 3. All indexes must be value according to the snapshot vectors. All edge values must
		///	also be correct and should not overflow the granularity of the ratio type (i.e. 256
		///	or billion).
		/// 4. For each edge, all targets are actually nominated by the voter.
		/// 5. Has correct self-votes.
		///
		/// A solutions score is consisted of 3 parameters:
		///
		/// 1. `min { support.total }` for each support of a winner. This value should be maximized.
		/// 2. `sum { support.total }` for each support of a winner. This value should be minimized.
		/// 3. `sum { support.total^2 }` for each support of a winner. This value should be
		///	minimized (to ensure less variance)
		///
		/// # <weight>
		/// The transaction is assumed to be the longest path, a better solution.
		///   - Initial solution is almost the same.
		///   - Worse solution is retraced in pre-dispatch-checks which sets its own weight.
		/// # </weight>
		#[weight = T::WeightInfo::submit_solution_better(
			size.validators.into(),
			size.nominators.into(),
			compact.voter_count() as u32,
			winners.len() as u32,
		)]
		pub fn submit_election_solution(
			origin,
			winners: Vec<ValidatorIndex>,
			compact: CompactAssignments,
			score: ElectionScore,
			era: EraIndex,
			size: ElectionSize,
		) -> DispatchResultWithPostInfo {
			let _who = ensure_signed(origin)?;
			Self::check_and_replace_solution(
				winners,
				compact,
				ElectionCompute::Signed,
				score,
				era,
				size,
			)
		}

		/// Unsigned version of `submit_election_solution`.
		///
		/// Note that this must pass the [`ValidateUnsigned`] check which only allows transactions
		/// from the local node to be included. In other words, only the block author can include a
		/// transaction in the block.
		///
		/// # <weight>
		/// See [`submit_election_solution`].
		/// # </weight>
		#[weight = T::WeightInfo::submit_solution_better(
			size.validators.into(),
			size.nominators.into(),
			compact.voter_count() as u32,
			winners.len() as u32,
		)]
		pub fn submit_election_solution_unsigned(
			origin,
			winners: Vec<ValidatorIndex>,
			compact: CompactAssignments,
			score: ElectionScore,
			era: EraIndex,
			size: ElectionSize,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let adjustments = Self::check_and_replace_solution(
				winners,
				compact,
				ElectionCompute::Unsigned,
				score,
				era,
				size,
			).expect(
				"An unsigned solution can only be submitted by validators; A validator should \
				always produce correct solutions, else this block should not be imported, thus \
				effectively depriving the validators from their authoring reward. Hence, this panic
				is expected."
			);

			Ok(adjustments)
		}

		/// Remove the given nominations from the calling validator.
		///
		/// Effects will be felt at the beginning of the next era.
		///
		/// The dispatch origin for this call must be _Signed_ by the controller, not the stash.
		/// And, it can be only called when [`EraElectionStatus`] is `Closed`. The controller
		/// account should represent a validator.
		///
		/// - `who`: A list of nominator stash accounts who are nominating this validator which
		///   should no longer be nominating this validator.
		///
		/// Note: Making this call only makes sense if you first set the validator preferences to
		/// block any further nominations.
		#[weight = T::WeightInfo::kick(who.len() as u32)]
		pub fn kick(origin, who: Vec<<T::Lookup as StaticLookup>::Source>) -> DispatchResult {
			let controller = ensure_signed(origin)?;
			ensure!(Self::era_election_status().is_closed(), <Error<T>>::CallNotAllowed);
			let ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;
			let stash = &ledger.stash;

			for nom_stash in who.into_iter()
				.map(T::Lookup::lookup)
				.collect::<Result<Vec<T::AccountId>, _>>()?
				.into_iter()
			{
				<Nominators<T>>::mutate(&nom_stash, |maybe_nom| if let Some(ref mut nom) = maybe_nom {
					if let Some(pos) = nom.targets.iter().position(|v| v == stash) {
						nom.targets.swap_remove(pos);
						Self::deposit_event(RawEvent::Kicked(nom_stash.clone(), stash.clone()));
					}
				});
			}

			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}

	/// Update the ledger while bonding ring and compute the *DNA* reward
	fn bond_etp(
		stash: &T::AccountId,
		controller: &T::AccountId,
		value: EtpBalance<T>,
		promise_month: u8,
		mut ledger: StakingLedgerT<T>,
	) -> Result<(TsInMs, TsInMs), DispatchError> {
		let StakingLedger {
			active_etp,
			active_deposit_etp,
			deposit_items,
			active_dna,
			..
		} = &mut ledger;

		let start_time = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();
		let mut expire_time = start_time;

		*active_etp = active_etp.saturating_add(value);

		// last check: the new active amount of ledger must be more than ED.
		ensure!(
			*active_etp >= T::EtpCurrency::minimum_balance()
				|| *active_dna >= T::DnaCurrency::minimum_balance(),
			<Error<T>>::InsufficientValue
		);

		// if stash promise to an extra-lock
		// there will be extra reward (*DNA*), which can also be used for staking
		if promise_month > 0 {
			expire_time += promise_month as TsInMs * MONTH_IN_MILLISECONDS;
			*active_deposit_etp += value;

			let dna_return = inflation::compute_dna_reward::<T>(value, promise_month);
			let dna_positive_imbalance = T::DnaCurrency::deposit_creating(&stash, dna_return);

			T::DnaReward::on_unbalanced(dna_positive_imbalance);
			deposit_items.push(TimeDepositItem {
				value,
				start_time,
				expire_time,
			});
		}

		Self::update_ledger(&controller, &mut ledger);

		Ok((start_time, expire_time))
	}

	/// Update the ledger while bonding controller with *DNA*
	fn bond_dna(
		controller: &T::AccountId,
		value: DnaBalance<T>,
		mut ledger: StakingLedgerT<T>,
	) -> DispatchResult {
		ledger.active_dna = ledger.active_dna.saturating_add(value);

		// last check: the new active amount of ledger must be more than ED.
		ensure!(
			ledger.active_etp >= T::EtpCurrency::minimum_balance()
				|| ledger.active_dna >= T::DnaCurrency::minimum_balance(),
			<Error<T>>::InsufficientValue
		);

		Self::update_ledger(&controller, &mut ledger);

		Ok(())
	}

	/// Turn the expired deposit items into normal bond
	pub fn clear_mature_deposits(mut ledger: StakingLedgerT<T>) -> (StakingLedgerT<T>, bool) {
		let now = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();
		let StakingLedger {
			stash,
			active_deposit_etp,
			deposit_items,
			..
		} = &mut ledger;
		let mut mutated = false;

		deposit_items.retain(|item| {
			if item.expire_time > now {
				true
			} else {
				mutated = true;
				*active_deposit_etp = active_deposit_etp.saturating_sub(item.value);

				false
			}
		});

		if mutated {
			Self::deposit_event(RawEvent::DepositsClaimed(stash.to_owned()));
		}

		(ledger, mutated)
	}

	// power is a mixture of ring and dna
	// For *ETP* power = etp_ratio * POWER_COUNT / 2
	// For *DNA* power = dna_ratio * POWER_COUNT / 2
	pub fn currency_to_power<S: TryInto<Balance>>(active: S, pool: S) -> Power {
		(Perquintill::from_rational_approximation(
			active.saturated_into::<Balance>(),
			pool.saturated_into::<Balance>().max(1),
		) * (T::TotalPower::get() as Balance / 2)) as _
	}

	/// The total power that can be slashed from a stash account as of right now.
	pub fn power_of(stash: &T::AccountId) -> Power {
		// Weight note: consider making the stake accessible through stash.
		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| {
				Self::currency_to_power::<_>(l.active_etp, Self::etp_pool())
					+ Self::currency_to_power::<_>(l.active_dna, Self::dna_pool())
			})
			.unwrap_or_default()
	}

	impl_rpc! {
		pub fn power_of_rpc(
			stash: impl sp_std::borrow::Borrow<T::AccountId>,
		) -> RuntimeDispatchInfo<Power> {
			RuntimeDispatchInfo { power: Self::power_of(stash.borrow()) }
		}
	}

	pub fn stake_of(stash: &T::AccountId) -> (EtpBalance<T>, DnaBalance<T>) {
		// Weight note: consider making the stake accessible through stash.
		Self::bonded(stash)
			.and_then(Self::ledger)
			.map(|l| (l.active_etp, l.active_dna))
			.unwrap_or_default()
	}

	/// Dump the list of validators and nominators into vectors and keep them on-chain.
	///
	/// This data is used to efficiently evaluate election results. returns `true` if the operation
	/// is successful.
	pub fn create_stakers_snapshot() -> (bool, Weight) {
		let mut consumed_weight = 0;
		let mut add_db_reads_writes = |reads, writes| {
			consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
		};
		let validators = <Validators<T>>::iter().map(|(v, _)| v).collect::<Vec<_>>();
		let mut nominators = <Nominators<T>>::iter().map(|(n, _)| n).collect::<Vec<_>>();

		let num_validators = validators.len();
		let num_nominators = nominators.len();
		add_db_reads_writes((num_validators + num_nominators) as Weight, 0);

		if num_validators > MAX_VALIDATORS
			|| num_nominators.saturating_add(num_validators) > MAX_NOMINATORS
		{
			log!(
				warn,
				"💸 Snapshot size too big [{} <> {}][{} <> {}].",
				num_validators,
				MAX_VALIDATORS,
				num_nominators,
				MAX_NOMINATORS,
			);
			(false, consumed_weight)
		} else {
			// all validators nominate themselves;
			nominators.extend(validators.clone());

			<SnapshotValidators<T>>::put(validators);
			<SnapshotNominators<T>>::put(nominators);
			add_db_reads_writes(0, 2);
			(true, consumed_weight)
		}
	}

	/// Clears both snapshots of stakers.
	fn kill_stakers_snapshot() {
		<SnapshotValidators<T>>::kill();
		<SnapshotNominators<T>>::kill();
	}

	fn do_payout_stakers(validator_stash: T::AccountId, era: EraIndex) -> DispatchResult {
		// Validate input data
		let current_era = CurrentEra::get().ok_or(<Error<T>>::InvalidEraToReward)?;
		ensure!(era <= current_era, <Error<T>>::InvalidEraToReward);
		let history_depth = Self::history_depth();
		ensure!(
			era >= current_era.saturating_sub(history_depth),
			<Error<T>>::InvalidEraToReward
		);

		// Note: if era has no reward to be claimed, era may be future. better not to update
		// `ledger.claimed_rewards` in this case.
		let era_payout =
			<ErasValidatorReward<T>>::get(&era).ok_or_else(|| <Error<T>>::InvalidEraToReward)?;

		let controller = Self::bonded(&validator_stash).ok_or(<Error<T>>::NotStash)?;
		let mut ledger = <Ledger<T>>::get(&controller).ok_or_else(|| <Error<T>>::NotController)?;

		ledger
			.claimed_rewards
			.retain(|&x| x >= current_era.saturating_sub(history_depth));
		match ledger.claimed_rewards.binary_search(&era) {
			Ok(_) => Err(<Error<T>>::AlreadyClaimed)?,
			Err(pos) => ledger.claimed_rewards.insert(pos, era),
		}

		let exposure = <ErasStakersClipped<T>>::get(&era, &ledger.stash);

		/* Input data seems good, no errors allowed after this point */

		<Ledger<T>>::insert(&controller, &ledger);

		// Get Era reward points. It has TOTAL and INDIVIDUAL
		// Find the fraction of the era reward that belongs to the validator
		// Take that fraction of the eras rewards to split to nominator and validator
		//
		// Then look at the validator, figure out the proportion of their reward
		// which goes to them and each of their nominators.

		let era_reward_points = <ErasRewardPoints<T>>::get(&era);
		let total_reward_points = era_reward_points.total;
		let validator_reward_points = era_reward_points
			.individual
			.get(&ledger.stash)
			.map(|points| *points)
			.unwrap_or_else(|| Zero::zero());

		// Nothing to do if they have no reward points.
		if validator_reward_points.is_zero() {
			return Ok(());
		}

		// This is the fraction of the total reward that the validator and the
		// nominators will get.
		let validator_total_reward_part =
			Perbill::from_rational_approximation(validator_reward_points, total_reward_points);

		// This is how much validator + nominators are entitled to.
		let validator_total_payout = validator_total_reward_part * era_payout;

		let module_account = Self::account_id();

		ensure!(
			T::EtpCurrency::usable_balance(&module_account) >= validator_total_payout,
			<Error<T>>::PayoutIns
		);

		let validator_prefs = Self::eras_validator_prefs(&era, &validator_stash);
		// Validator first gets a cut off the top.
		let validator_commission = validator_prefs.commission;
		let validator_commission_payout = validator_commission * validator_total_payout;

		let validator_leftover_payout = validator_total_payout - validator_commission_payout;
		// Now let's calculate how this is split to the validator.
		let validator_exposure_part =
			Perbill::from_rational_approximation(exposure.own_power, exposure.total_power);
		let validator_staking_payout = validator_exposure_part * validator_leftover_payout;

		// Due to the `payout * percent` there might be some losses
		let mut actual_payout = <EtpPositiveImbalance<T>>::zero();

		// We can now make total validator payout:
		if let Some(imbalance) = Self::make_payout(
			&ledger.stash,
			validator_staking_payout + validator_commission_payout,
		) {
			let payout = imbalance.peek();

			actual_payout.subsume(imbalance);

			Self::deposit_event(RawEvent::Reward(ledger.stash, payout));
		}

		// Lets now calculate how this is split to the nominators.
		// Reward only the clipped exposures. Note this is not necessarily sorted.
		for nominator in exposure.others.iter() {
			let nominator_exposure_part =
				Perbill::from_rational_approximation(nominator.power, exposure.total_power);

			let nominator_reward: EtpBalance<T> =
				nominator_exposure_part * validator_leftover_payout;
			// We can now make nominator payout:
			if let Some(imbalance) = Self::make_payout(&nominator.who, nominator_reward) {
				let payout = imbalance.peek();

				actual_payout.subsume(imbalance);

				Self::deposit_event(RawEvent::Reward(nominator.who.clone(), payout));
			}
		}

		T::EtpCurrency::settle(
			&module_account,
			actual_payout,
			WithdrawReasons::all(),
			KeepAlive,
		)
		.map_err(|_| <Error<T>>::PayoutIns)?;

		Ok(())
	}

	/// Update the ledger for a controller.
	///
	/// BE CAREFUL:
	/// 	This will also update the stash lock.
	/// 	DO NOT modify the locks' staking amount outside this function.
	fn update_ledger(controller: &T::AccountId, ledger: &mut StakingLedgerT<T>) {
		let StakingLedger {
			active_etp,
			active_dna,
			etp_staking_lock,
			dna_staking_lock,
			..
		} = ledger;

		if *active_etp != etp_staking_lock.staking_amount {
			let origin_active_etp = etp_staking_lock.staking_amount;

			etp_staking_lock.staking_amount = *active_etp;

			<EtpPool<T>>::mutate(|pool| {
				if origin_active_etp > *active_etp {
					*pool = pool.saturating_sub(origin_active_etp - *active_etp);
				} else {
					*pool = pool.saturating_add(*active_etp - origin_active_etp);
				}
			});

			T::EtpCurrency::set_lock(
				STAKING_ID,
				&ledger.stash,
				LockFor::Staking(ledger.etp_staking_lock.clone()),
				WithdrawReasons::all(),
			);
		}

		if *active_dna != dna_staking_lock.staking_amount {
			let origin_active_dna = dna_staking_lock.staking_amount;

			dna_staking_lock.staking_amount = *active_dna;

			<DnaPool<T>>::mutate(|pool| {
				if origin_active_dna > *active_dna {
					*pool = pool.saturating_sub(origin_active_dna - *active_dna);
				} else {
					*pool = pool.saturating_add(*active_dna - origin_active_dna);
				}
			});

			T::DnaCurrency::set_lock(
				STAKING_ID,
				&ledger.stash,
				LockFor::Staking(ledger.dna_staking_lock.clone()),
				WithdrawReasons::all(),
			);
		}

		<Ledger<T>>::insert(controller, ledger);
	}

	/// Chill a stash account.
	fn chill_stash(stash: &T::AccountId) {
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);
	}

	/// Actually make a payment to a staker. This uses the currency's reward function
	/// to pay the right payee for the given staker account.
	fn make_payout(
		stash: &T::AccountId,
		amount: EtpBalance<T>,
	) -> Option<EtpPositiveImbalance<T>> {
		let dest = Self::payee(stash);
		match dest {
			RewardDestination::Controller => Self::bonded(stash).and_then(|controller| {
				Some(T::EtpCurrency::deposit_creating(&controller, amount))
			}),
			RewardDestination::Stash => T::EtpCurrency::deposit_into_existing(stash, amount).ok(),
			RewardDestination::Staked => Self::bonded(stash)
				.and_then(|c| Self::ledger(&c).map(|l| (c, l)))
				.and_then(|(c, mut l)| {
					let r = T::EtpCurrency::deposit_into_existing(stash, amount).ok();

					if r.is_some() {
						l.active_etp += amount;

						Self::update_ledger(&c, &mut l);
					}

					r
				}),
			RewardDestination::Account(dest_account) => {
				Some(T::EtpCurrency::deposit_creating(&dest_account, amount))
			}
		}
	}

	/// Plan a new session potentially trigger a new era.
	fn new_session(session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		if let Some(current_era) = Self::current_era() {
			// Initial era has been set.

			let current_era_start_session_index = Self::eras_start_session_index(current_era)
				.unwrap_or_else(|| {
					frame_support::print("Error: start_session_index must be set for current_era");
					0
				});

			let era_length = session_index
				.checked_sub(current_era_start_session_index)
				.unwrap_or(0); // Must never happen.

			match ForceEra::get() {
				Forcing::ForceNew => ForceEra::kill(),
				Forcing::ForceAlways => (),
				Forcing::NotForcing if era_length >= T::SessionsPerEra::get() => (),
				_ => {
					// Either `ForceNone`, or `NotForcing && era_length < T::SessionsPerEra::get()`.
					if era_length + 1 == T::SessionsPerEra::get() {
						IsCurrentSessionFinal::put(true);
					} else if era_length >= T::SessionsPerEra::get() {
						// Should only happen when we are ready to trigger an era but we have ForceNone,
						// otherwise previous arm would short circuit.
						Self::close_election_window();
					}
					return None;
				}
			}

			// new era.
			Self::new_era(session_index)
		} else {
			// Set initial era
			Self::new_era(session_index)
		}
	}

	/// Basic and cheap checks that we perform in validate unsigned, and in the execution.
	///
	/// State reads: ElectionState, CurrentEr, QueuedScore.
	///
	/// This function does weight refund in case of errors, which is based upon the fact that it is
	/// called at the very beginning of the call site's function.
	pub fn pre_dispatch_checks(score: ElectionScore, era: EraIndex) -> DispatchResultWithPostInfo {
		// discard solutions that are not in-time
		// check window open
		ensure!(
			Self::era_election_status().is_open(),
			<Error<T>>::OffchainElectionEarlySubmission.with_weight(T::DbWeight::get().reads(1)),
		);

		// check current era.
		if let Some(current_era) = Self::current_era() {
			ensure!(
				current_era == era,
				<Error<T>>::OffchainElectionEarlySubmission
					.with_weight(T::DbWeight::get().reads(2)),
			)
		}

		// assume the given score is valid. Is it better than what we have on-chain, if we have any?
		if let Some(queued_score) = Self::queued_score() {
			ensure!(
				is_score_better(score, queued_score, T::MinSolutionScoreBump::get()),
				<Error<T>>::OffchainElectionWeakSubmission.with_weight(T::DbWeight::get().reads(3)),
			)
		}

		Ok(None.into())
	}

	/// Checks a given solution and if correct and improved, writes it on chain as the queued result
	/// of the next round. This may be called by both a signed and an unsigned transaction.
	pub fn check_and_replace_solution(
		winners: Vec<ValidatorIndex>,
		compact_assignments: CompactAssignments,
		compute: ElectionCompute,
		claimed_score: ElectionScore,
		era: EraIndex,
		election_size: ElectionSize,
	) -> DispatchResultWithPostInfo {
		// Do the basic checks. era, claimed score and window open.
		let _ = Self::pre_dispatch_checks(claimed_score, era)?;

		// before we read any further state, we check that the unique targets in compact is same as
		// compact. is a all in-memory check and easy to do. Moreover, it ensures that the solution
		// is not full of bogus edges that can cause lots of reads to SlashingSpans. Thus, we can
		// assume that the storage access of this function is always O(|winners|), not
		// O(|compact.edge_count()|).
		ensure!(
			compact_assignments.unique_targets().len() == winners.len(),
			<Error<T>>::OffchainElectionBogusWinnerCount,
		);

		// Check that the number of presented winners is sane. Most often we have more candidates
		// than we need. Then it should be `Self::validator_count()`. Else it should be all the
		// candidates.
		let snapshot_validators_length = <SnapshotValidators<T>>::decode_len()
			.map(|l| l as u32)
			.ok_or_else(|| <Error<T>>::SnapshotUnavailable)?;

		// size of the solution must be correct.
		ensure!(
			snapshot_validators_length == u32::from(election_size.validators),
			<Error<T>>::OffchainElectionBogusElectionSize,
		);

		// check the winner length only here and when we know the length of the snapshot validators
		// length.
		let desired_winners = Self::validator_count().min(snapshot_validators_length);
		ensure!(
			winners.len() as u32 == desired_winners,
			<Error<T>>::OffchainElectionBogusWinnerCount
		);

		let snapshot_nominators_len = <SnapshotNominators<T>>::decode_len()
			.map(|l| l as u32)
			.ok_or_else(|| <Error<T>>::SnapshotUnavailable)?;

		// rest of the size of the solution must be correct.
		ensure!(
			snapshot_nominators_len == election_size.nominators,
			<Error<T>>::OffchainElectionBogusElectionSize,
		);

		// decode snapshot validators.
		let snapshot_validators =
			Self::snapshot_validators().ok_or(<Error<T>>::SnapshotUnavailable)?;

		// check if all winners were legit; this is rather cheap. Replace with accountId.
		let winners = winners
			.into_iter()
			.map(|widx| {
				// NOTE: at the moment, since staking is explicitly blocking any offence until election
				// is closed, we don't check here if the account id at `snapshot_validators[widx]` is
				// actually a validator. If this ever changes, this loop needs to also check this.
				snapshot_validators
					.get(widx as usize)
					.cloned()
					.ok_or(<Error<T>>::OffchainElectionBogusWinner)
			})
			.collect::<Result<Vec<T::AccountId>, Error<T>>>()?;

		// decode the rest of the snapshot.
		let snapshot_nominators =
			Self::snapshot_nominators().ok_or(<Error<T>>::SnapshotUnavailable)?;

		// helpers
		let nominator_at = |i: NominatorIndex| -> Option<T::AccountId> {
			snapshot_nominators.get(i as usize).cloned()
		};
		let validator_at = |i: ValidatorIndex| -> Option<T::AccountId> {
			snapshot_validators.get(i as usize).cloned()
		};

		// un-compact.
		let assignments = compact_assignments
			.into_assignment(nominator_at, validator_at)
			.map_err(|e| {
				// log the error since it is not propagated into the runtime error.
				log!(warn, "💸 un-compacting solution failed due to {:?}", e);
				<Error<T>>::OffchainElectionBogusCompact
			})?;

		// check all nominators actually including the claimed vote. Also check correct self votes.
		// Note that we assume all validators and nominators in `assignments` are properly bonded,
		// because they are coming from the snapshot via a given index.
		for Assignment { who, distribution } in assignments.iter() {
			let is_validator = <Validators<T>>::contains_key(&who);
			let maybe_nomination = Self::nominators(&who);

			if !(maybe_nomination.is_some() ^ is_validator) {
				// all of the indices must map to either a validator or a nominator. If this is ever
				// not the case, then the locking system of staking is most likely faulty, or we
				// have bigger problems.
				log!(
					error,
					"💸 detected an error in the staking locking and snapshot."
				);
				// abort.
				return Err(<Error<T>>::OffchainElectionBogusNominator.into());
			}

			if !is_validator {
				// a normal vote
				let nomination = maybe_nomination.expect(
					"exactly one of `maybe_validator` and `maybe_nomination.is_some` is true. \
					is_validator is false; maybe_nomination is some; qed",
				);

				// NOTE: we don't really have to check here if the sum of all edges are the
				// nominator correct. Un-compacting assures this by definition.

				for (t, _) in distribution {
					// each target in the provided distribution must be actually nominated by the
					// nominator after the last non-zero slash.
					if nomination.targets.iter().find(|&tt| tt == t).is_none() {
						return Err(<Error<T>>::OffchainElectionBogusNomination.into());
					}

					if <Self as Store>::SlashingSpans::get(&t).map_or(false, |spans| {
						nomination.submitted_in < spans.last_nonzero_slash()
					}) {
						return Err(<Error<T>>::OffchainElectionSlashedNomination.into());
					}
				}
			} else {
				// a self vote
				ensure!(
					distribution.len() == 1,
					<Error<T>>::OffchainElectionBogusSelfVote
				);
				ensure!(
					distribution[0].0 == *who,
					<Error<T>>::OffchainElectionBogusSelfVote
				);
				// defensive only. A compact assignment of length one does NOT encode the weight and
				// it is always created to be 100%.
				ensure!(
					distribution[0].1 == OffchainAccuracy::one(),
					<Error<T>>::OffchainElectionBogusSelfVote,
				);
			}
		}

		// convert into staked assignments.
		let staked_assignments =
			sp_npos_elections::assignment_ratio_to_staked(assignments, |s| Self::power_of(s) as _);

		// build the support map thereof in order to evaluate.
		let supports = to_support_map::<T::AccountId>(&winners, &staked_assignments)
			.map_err(|_| <Error<T>>::OffchainElectionBogusEdge)?;

		// Check if the score is the same as the claimed one.
		let submitted_score = (&supports).evaluate();
		ensure!(
			submitted_score == claimed_score,
			<Error<T>>::OffchainElectionBogusScore
		);

		// At last, alles Ok. Exposures and store the result.
		let exposures = Self::collect_exposure(supports);
		log!(
			info,
			"💸 A better solution (with compute {:?} and score {:?}) has been validated and stored on chain.",
			compute,
			submitted_score,
		);

		// write new results.
		<QueuedElected<T>>::put(ElectionResult {
			elected_stashes: winners,
			compute,
			exposures,
		});
		QueuedScore::put(submitted_score);

		// emit event.
		Self::deposit_event(RawEvent::SolutionStored(compute));

		Ok(None.into())
	}

	/// Start a session potentially starting an era.
	fn start_session(start_session: SessionIndex) {
		let next_active_era = Self::active_era().map(|e| e.index + 1).unwrap_or(0);
		// This is only `Some` when current era has already progressed to the next era, while the
		// active era is one behind (i.e. in the *last session of the active era*, or *first session
		// of the new current era*, depending on how you look at it).
		if let Some(next_active_era_start_session_index) =
			Self::eras_start_session_index(next_active_era)
		{
			if next_active_era_start_session_index == start_session {
				Self::start_era(start_session);
			} else if next_active_era_start_session_index < start_session {
				// This arm should never happen, but better handle it than to stall the staking
				// pallet.
				frame_support::print("Warning: A session appears to have been skipped.");
				Self::start_era(start_session);
			}
		}
	}

	/// End a session potentially ending an era.
	fn end_session(session_index: SessionIndex) {
		if let Some(active_era) = Self::active_era() {
			let next_active_era_start_session_index =
				Self::eras_start_session_index(active_era.index + 1).unwrap_or_else(|| {
					frame_support::print(
						"Error: start_session_index must be set for active_era + 1",
					);
					0
				});

			if next_active_era_start_session_index == session_index + 1 {
				Self::end_era(active_era, session_index);
			}
		}
	}

	/// * Increment `active_era.index`,
	/// * reset `active_era.start`,
	/// * update `BondedEras` and apply slashes.
	fn start_era(start_session: SessionIndex) {
		let active_era = ActiveEra::mutate(|active_era| {
			let new_index = active_era.as_ref().map(|info| info.index + 1).unwrap_or(0);
			*active_era = Some(ActiveEraInfo {
				index: new_index,
				// Set new active era start in next `on_finalize`. To guarantee usage of `Time`
				start: None,
			});
			new_index
		});

		let bonding_duration = T::BondingDurationInEra::get();

		BondedEras::mutate(|bonded| {
			bonded.push((active_era, start_session));

			if active_era > bonding_duration {
				let first_kept = active_era - bonding_duration;

				// prune out everything that's from before the first-kept index.
				let n_to_prune = bonded
					.iter()
					.take_while(|&&(era_idx, _)| era_idx < first_kept)
					.count();

				// kill slashing metadata.
				for (pruned_era, _) in bonded.drain(..n_to_prune) {
					slashing::clear_era_metadata::<T>(pruned_era);
				}

				if let Some(&(_, first_session)) = bonded.first() {
					T::SessionInterface::prune_historical_up_to(first_session);
				}
			}
		});

		Self::apply_unapplied_slashes(active_era);
	}

	/// Compute payout for era.
	#[allow(unused_variables, unused_mut)]
	fn end_era(active_era: ActiveEraInfo, _session_index: SessionIndex) {
		// Temporary Disable era reward payout
		// Note: active_era_start can be None if end era is called during genesis config.
		//if let Some(active_era_start) = active_era.start {
			//let now = T::UnixTime::now().as_millis().saturated_into::<TsInMs>();
			//let living_time = Self::living_time();
			//let era_duration = now - active_era_start;

			//let (validator_payout, max_payout) = inflation::compute_total_payout::<T>(
			//	era_duration,
			//	Self::living_time(),
			//	T::Cap::get().saturating_sub(T::EtpCurrency::total_issuance()),
			//	PayoutFraction::get(),
			//);
			//let rest = max_payout.saturating_sub(validator_payout);

			//Self::deposit_event(RawEvent::EraPayout(
			//	active_era.index,
			//	validator_payout,
			//	rest,
			//));

			//LivingTime::put(living_time + era_duration);
			// Set ending era reward.
			
			//<ErasValidatorReward<T>>::insert(&active_era.index, validator_payout);
			//T::EtpCurrency::deposit_creating(&Self::account_id(), validator_payout);
			//T::EtpRewardRemainder::on_unbalanced(T::EtpCurrency::issue(rest));
		//}
	}

	/// Plan a new era. Return the potential new staking set.
	fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		// Increment or set current era.
		let current_era = CurrentEra::mutate(|s| {
			*s = Some(s.map(|s| s + 1).unwrap_or(0));
			s.unwrap()
		});
		ErasStartSessionIndex::insert(&current_era, &start_session_index);

		// Clean old era information.
		if let Some(old_era) = current_era.checked_sub(Self::history_depth() + 1) {
			Self::clear_era_information(old_era);
		}

		// Set staking information for new era.
		let maybe_new_validators = Self::select_and_update_validators(current_era);

		maybe_new_validators
	}

	/// Remove all the storage items associated with the election.
	fn close_election_window() {
		// Close window.
		<EraElectionStatus<T>>::put(ElectionStatus::Closed);
		// Kill snapshots.
		Self::kill_stakers_snapshot();
		// Don't track final session.
		IsCurrentSessionFinal::put(false);
	}

	/// Select the new validator set at the end of the era.
	///
	/// Runs [`try_do_phragmen`] and updates the following storage items:
	/// - [`EraElectionStatus`]: with `None`.
	/// - [`ErasStakers`]: with the new staker set.
	/// - [`ErasStakersClipped`].
	/// - [`ErasValidatorPrefs`].
	/// - [`ErasTotalStake`]: with the new total stake.
	/// - [`SnapshotValidators`] and [`SnapshotNominators`] are both removed.
	///
	/// Internally, [`QueuedElected`], snapshots and [`QueuedScore`] are also consumed.
	///
	/// If the election has been successful, It passes the new set upwards.
	///
	/// This should only be called at the end of an era.
	fn select_and_update_validators(current_era: EraIndex) -> Option<Vec<T::AccountId>> {
		if let Some(ElectionResultT::<T> {
			elected_stashes,
			exposures,
			compute,
		}) = Self::try_do_election()
		{
			// Totally close the election round and data.
			Self::close_election_window();

			// Populate Stakers and write slot stake.
			let mut total_stake = 0;
			exposures.into_iter().for_each(|(stash, exposure)| {
				// Total `Power` is `1_000_000_000`, never get overflow; qed
				total_stake += exposure.total_power;
				<ErasStakers<T>>::insert(current_era, &stash, &exposure);

				let mut exposure_clipped = exposure;
				let clipped_max_len = T::MaxNominatorRewardedPerValidator::get() as usize;
				if exposure_clipped.others.len() > clipped_max_len {
					exposure_clipped
						.others
						.sort_by(|a, b| a.power.cmp(&b.power).reverse());
					exposure_clipped.others.truncate(clipped_max_len);
				}
				<ErasStakersClipped<T>>::insert(&current_era, &stash, exposure_clipped);
			});
			// Insert current era staking information
			ErasTotalStake::insert(&current_era, total_stake);

			// collect the pref of all winners
			for stash in &elected_stashes {
				let pref = Self::validators(stash);
				<ErasValidatorPrefs<T>>::insert(&current_era, stash, pref);
			}

			// emit event
			Self::deposit_event(RawEvent::StakingElection(compute));

			log!(
				info,
				"💸 new validator set of size {:?} has been elected via {:?} for era {:?}",
				elected_stashes.len(),
				compute,
				current_era,
			);

			Some(elected_stashes)
		} else {
			None
		}
	}

	/// Select a new validator set from the assembled stakers and their role preferences. It tries
	/// first to peek into [`QueuedElected`]. Otherwise, it runs a new on-chain phragmen election.
	///
	/// If [`QueuedElected`] and [`QueuedScore`] exists, they are both removed. No further storage
	/// is updated.
	fn try_do_election() -> Option<ElectionResultT<T>> {
		// an election result from either a stored submission or locally executed one.
		let next_result = <QueuedElected<T>>::take().or_else(|| Self::do_on_chain_phragmen());

		// either way, kill this. We remove it here to make sure it always has the exact same
		// lifetime as `QueuedElected`.
		QueuedScore::kill();

		next_result
	}

	/// Execute election and return the new results. The edge weights are processed into support
	/// values.
	///
	/// This is basically a wrapper around [`do_phragmen`] which translates
	/// `PrimitiveElectionResult` into `ElectionResult`.
	///
	/// No storage item is updated.
	fn do_on_chain_phragmen() -> Option<ElectionResultT<T>> {
		if let Some(phragmen_result) = Self::do_phragmen::<ChainAccuracy>(0) {
			let elected_stashes = phragmen_result
				.winners
				.iter()
				.map(|(s, _)| s.clone())
				.collect::<Vec<T::AccountId>>();
			let assignments = phragmen_result.assignments;

			let staked_assignments =
				sp_npos_elections::assignment_ratio_to_staked(assignments, |s| {
					Self::power_of(s) as _
				});

			let supports = to_support_map::<T::AccountId>(&elected_stashes, &staked_assignments)
				.map_err(|_| {
					log!(
						error,
						"💸 on-chain phragmen is failing due to a problem in the result. This must be a bug."
					)
				})
				.ok()?;

			// collect exposures
			let exposures = Self::collect_exposure(supports);

			// In order to keep the property required by `on_session_ending` that we must return the
			// new validator set even if it's the same as the old, as long as any underlying
			// economic conditions have changed, we don't attempt to do any optimization where we
			// compare against the prior set.
			Some(ElectionResultT::<T> {
				elected_stashes,
				exposures,
				compute: ElectionCompute::OnChain,
			})
		} else {
			// There were not enough candidates for even our minimal level of functionality. This is
			// bad. We should probably disable all functionality except for block production and let
			// the chain keep producing blocks until we can decide on a sufficiently substantial
			// set. TODO: #2494
			None
		}
	}

	/// Execute phragmen election and return the new results. No post-processing is applied and the
	/// raw edge weights are returned.
	///
	/// Self votes are added and nominations before the most recent slashing span are ignored.
	///
	/// No storage item is updated.
	pub fn do_phragmen<Accuracy: PerThing128>(
		iterations: usize,
	) -> Option<PrimitiveElectionResult<T::AccountId, Accuracy>> {
		let mut all_nominators: Vec<(T::AccountId, VoteWeight, Vec<T::AccountId>)> = vec![];
		let mut all_validators = vec![];
		for (validator, _) in <Validators<T>>::iter() {
			// append self vote
			let self_vote = (
				validator.clone(),
				Self::power_of(&validator) as _,
				vec![validator.clone()],
			);
			all_nominators.push(self_vote);
			all_validators.push(validator);
		}

		let nominator_votes = <Nominators<T>>::iter().map(|(nominator, nominations)| {
			let Nominations {
				submitted_in,
				mut targets,
				suppressed: _,
			} = nominations;

			// Filter out nomination targets which were nominated before the most recent
			// slashing span.
			targets.retain(|stash| {
				<Self as Store>::SlashingSpans::get(&stash)
					.map_or(true, |spans| submitted_in >= spans.last_nonzero_slash())
			});

			(nominator, targets)
		});
		all_nominators.extend(nominator_votes.map(|(n, ns)| {
			let s = Self::power_of(&n) as _;
			(n, s, ns)
		}));

		if all_validators.len() < Self::minimum_validator_count().max(1) as usize {
			// If we don't have enough candidates, nothing to do.
			log!(
				error,
				"💸 Chain does not have enough staking candidates to operate. Era {:?}.",
				Self::current_era()
			);
			None
		} else {
			seq_phragmen::<_, Accuracy>(
				Self::validator_count() as usize,
				all_validators,
				all_nominators,
				Some((iterations, 0)), // exactly run `iterations` rounds.
			)
			.map_err(|err| log!(error, "Call to seq-phragmen failed due to {:?}", err))
			.ok()
		}
	}

	/// Consume a set of [`Supports`] from [`sp_npos_elections`] and collect them into a [`Exposure`]
	fn collect_exposure(supports: SupportMap<T::AccountId>) -> Vec<(T::AccountId, ExposureT<T>)> {
		supports
			.into_iter()
			.map(|(validator, support)| {
				// build `struct exposure` from `support`
				let mut own_etp_balance: EtpBalance<T> = Zero::zero();
				let mut own_dna_balance: DnaBalance<T> = Zero::zero();
				let mut own_power = 0;
				let mut total_power = 0;
				let mut others = Vec::with_capacity(support.voters.len());
				support.voters.into_iter().for_each(|(nominator, weight)| {
					let origin_weight = Self::power_of(&nominator) as ExtendedBalance;
					let (origin_etp_balance, origin_dna_balance) = Self::stake_of(&nominator);

					let etp_balance = if let Ok(etp_balance) = multiply_by_rational(
						origin_etp_balance.saturated_into(),
						weight,
						origin_weight,
					) {
						etp_balance.saturated_into()
					} else {
						log!(
							error,
							"[staking] Origin ETP: {:?}, Weight: {:?}, Origin Weight: {:?}",
							origin_etp_balance,
							weight,
							origin_weight
						);
						Zero::zero()
					};
					let dna_balance = if let Ok(dna_balance) = multiply_by_rational(
						origin_dna_balance.saturated_into(),
						weight,
						origin_weight,
					) {
						dna_balance.saturated_into()
					} else {
						log!(
							error,
							"[staking] Origin DNA: {:?}, Weight: {:?}, Origin Weight: {:?}",
							origin_dna_balance,
							weight,
							origin_weight
						);
						Zero::zero()
					};
					let power = weight as Power;

					if nominator == validator {
						own_etp_balance = own_etp_balance.saturating_add(etp_balance);
						own_dna_balance = own_dna_balance.saturating_add(dna_balance);
						own_power = own_power.saturating_add(power);
					} else {
						others.push(IndividualExposure {
							who: nominator,
							etp_balance,
							dna_balance,
							power,
						});
					}
					total_power = total_power.saturating_add(power);
				});

				let exposure = Exposure {
					own_etp_balance,
					own_dna_balance,
					own_power,
					total_power,
					others,
				};

				(validator, exposure)
			})
			.collect::<Vec<(T::AccountId, ExposureT<T>)>>()
	}

	/// Remove all associated data of a stash account from the staking system.
	///
	/// Assumes storage is upgraded before calling.
	///
	/// This is called:
	/// - after a `withdraw_unbond()` call that frees all of a stash's bonded balance.
	/// - through `reap_stash()` if the balance has fallen to zero (through slashing).
	fn kill_stash(stash: &T::AccountId, num_slashing_spans: u32) -> DispatchResult {
		let controller = <Bonded<T>>::get(stash).ok_or(<Error<T>>::NotStash)?;

		slashing::clear_stash_metadata::<T>(stash, num_slashing_spans)?;

		<Bonded<T>>::remove(stash);
		<Ledger<T>>::remove(&controller);

		<Payee<T>>::remove(stash);
		<Validators<T>>::remove(stash);
		<Nominators<T>>::remove(stash);

		<frame_system::Module<T>>::dec_consumers(stash);

		Ok(())
	}

	/// Clear all era information for given era.
	fn clear_era_information(era_index: EraIndex) {
		<ErasStakers<T>>::remove_prefix(era_index);
		<ErasStakersClipped<T>>::remove_prefix(era_index);
		<ErasValidatorPrefs<T>>::remove_prefix(era_index);
		<ErasValidatorReward<T>>::remove(era_index);
		<ErasRewardPoints<T>>::remove(era_index);
		ErasTotalStake::remove(era_index);
		ErasStartSessionIndex::remove(era_index);
	}

	/// Apply previously-unapplied slashes on the beginning of a new era, after a delay.
	fn apply_unapplied_slashes(active_era: EraIndex) {
		let slash_defer_duration = T::SlashDeferDuration::get();
		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if let Some(ref mut earliest) = earliest {
				let keep_from = active_era.saturating_sub(slash_defer_duration);
				for era in (*earliest)..keep_from {
					let era_slashes = <Self as Store>::UnappliedSlashes::take(&era);
					for slash in era_slashes {
						slashing::apply_slash::<T>(slash);
					}
				}

				*earliest = (*earliest).max(keep_from)
			}
		})
	}

	/// Add reward points to validators using their stash account ID.
	///
	/// Validators are keyed by stash account ID and must be in the current elected set.
	///
	/// For each element in the iterator the given number of points in u32 is added to the
	/// validator, thus duplicates are handled.
	///
	/// At the end of the era each the total payout will be distributed among validator
	/// relatively to their points.
	///
	/// COMPLEXITY: Complexity is `number_of_validator_to_reward x current_elected_len`.
	/// If you need to reward lots of validator consider using `reward_by_indices`.
	pub fn reward_by_ids(validators_points: impl IntoIterator<Item = (T::AccountId, u32)>) {
		if let Some(active_era) = Self::active_era() {
			<ErasRewardPoints<T>>::mutate(active_era.index, |era_rewards| {
				for (validator, points) in validators_points.into_iter() {
					*era_rewards.individual.entry(validator).or_default() += points;
					era_rewards.total += points;
				}
			});
		}
	}

	/// Ensures that at the end of the current session there will be a new era.
	fn ensure_new_era() {
		match ForceEra::get() {
			Forcing::ForceAlways | Forcing::ForceNew => (),
			_ => ForceEra::put(Forcing::ForceNew),
		}
	}

	fn will_era_be_forced() -> bool {
		match ForceEra::get() {
			Forcing::ForceAlways | Forcing::ForceNew => true,
			Forcing::ForceNone | Forcing::NotForcing => false,
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn add_era_stakers(
		current_era: EraIndex,
		controller: T::AccountId,
		exposure: ExposureT<T>,
	) {
		<ErasStakers<T>>::insert(&current_era, &controller, &exposure);
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn put_election_status(status: ElectionStatus<T::BlockNumber>) {
		<EraElectionStatus<T>>::put(status);
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn set_slash_reward_fraction(fraction: Perbill) {
		SlashRewardFraction::put(fraction);
	}
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(new_index: SessionIndex) -> Option<Vec<T::AccountId>> {
		frame_support::debug::native::trace!(
			target: LOG_TARGET,
			"[{}] planning new_session({})",
			<frame_system::Module<T>>::block_number(),
			new_index
		);
		Self::new_session(new_index)
	}
	fn end_session(end_index: SessionIndex) {
		frame_support::debug::native::trace!(
			target: LOG_TARGET,
			"[{}] ending end_session({})",
			<frame_system::Module<T>>::block_number(),
			end_index
		);
		Self::end_session(end_index)
	}
	fn start_session(start_index: SessionIndex) {
		frame_support::debug::native::trace!(
			target: LOG_TARGET,
			"[{}] starting start_session({})",
			<frame_system::Module<T>>::block_number(),
			start_index
		);
		Self::start_session(start_index)
	}
}

impl<T: Config> pallet_session::historical::SessionManager<T::AccountId, ExposureT<T>>
	for Module<T>
{
	fn new_session(new_index: SessionIndex) -> Option<Vec<(T::AccountId, ExposureT<T>)>> {
		<Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
			let current_era = Self::current_era()
				// Must be some as a new era has been created.
				.unwrap_or(0);

			validators
				.into_iter()
				.map(|v| {
					let exposure = Self::eras_stakers(current_era, &v);
					(v, exposure)
				})
				.collect()
		})
	}
	fn start_session(start_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::start_session(start_index)
	}
	fn end_session(end_index: SessionIndex) {
		<Self as pallet_session::SessionManager<_>>::end_session(end_index)
	}
}

/// This is intended to be used with `FilterHistoricalOffences`.
impl<T: Config>
	OnOffenceHandler<T::AccountId, pallet_session::historical::IdentificationTuple<T>, Weight>
	for Module<T>
where
	T: pallet_session::Config<ValidatorId = AccountId<T>>,
	T: pallet_session::historical::Config<
		FullIdentification = ExposureT<T>,
		FullIdentificationOf = ExposureOf<T>,
	>,
	T::SessionHandler: pallet_session::SessionHandler<AccountId<T>>,
	T::SessionManager: pallet_session::SessionManager<AccountId<T>>,
	T::ValidatorIdOf: Convert<AccountId<T>, Option<AccountId<T>>>,
{
	fn on_offence(
		offenders: &[OffenceDetails<
			T::AccountId,
			pallet_session::historical::IdentificationTuple<T>,
		>],
		slash_fraction: &[Perbill],
		slash_session: SessionIndex,
	) -> Result<Weight, ()> {
		if !Self::can_report() {
			return Err(());
		}

		let reward_proportion = SlashRewardFraction::get();
		let mut consumed_weight: Weight = 0;
		let mut add_db_reads_writes = |reads, writes| {
			consumed_weight += T::DbWeight::get().reads_writes(reads, writes);
		};

		let active_era = {
			let active_era = Self::active_era();
			add_db_reads_writes(1, 0);
			if active_era.is_none() {
				// this offence need not be re-submitted.
				return Ok(consumed_weight);
			}
			active_era
				.expect("value checked not to be `None`; qed")
				.index
		};
		let active_era_start_session_index = Self::eras_start_session_index(active_era)
			.unwrap_or_else(|| {
				frame_support::print("Error: start_session_index must be set for current_era");
				0
			});
		add_db_reads_writes(1, 0);

		let window_start = active_era.saturating_sub(T::BondingDurationInEra::get());

		// fast path for active-era report - most likely.
		// `slash_session` cannot be in a future active era. It must be in `active_era` or before.
		let slash_era = if slash_session >= active_era_start_session_index {
			active_era
		} else {
			let eras = BondedEras::get();
			add_db_reads_writes(1, 0);

			// reverse because it's more likely to find reports from recent eras.
			match eras
				.iter()
				.rev()
				.filter(|&&(_, ref sesh)| sesh <= &slash_session)
				.next()
			{
				Some(&(ref slash_era, _)) => *slash_era,
				// before bonding period. defensive - should be filtered out.
				None => return Ok(consumed_weight),
			}
		};

		<Self as Store>::EarliestUnappliedSlash::mutate(|earliest| {
			if earliest.is_none() {
				*earliest = Some(active_era)
			}
		});
		add_db_reads_writes(1, 1);

		let slash_defer_duration = T::SlashDeferDuration::get();

		let invulnerables = Self::invulnerables();
		add_db_reads_writes(1, 0);

		for (details, slash_fraction) in offenders.iter().zip(slash_fraction) {
			let (stash, exposure) = &details.offender;

			// Skip if the validator is invulnerable.
			if invulnerables.contains(stash) {
				continue;
			}

			let unapplied = slashing::compute_slash::<T>(slashing::SlashParams {
				stash,
				slash: *slash_fraction,
				exposure,
				slash_era,
				window_start,
				now: active_era,
				reward_proportion,
			});

			if let Some(mut unapplied) = unapplied {
				let nominators_len = unapplied.others.len() as u64;
				let reporters_len = details.reporters.len() as u64;

				{
					let upper_bound = 1 /* Validator/NominatorSlashInEra */ + 2 /* fetch_spans */;
					let rw = upper_bound + nominators_len * upper_bound;
					add_db_reads_writes(rw, rw);
				}
				unapplied.reporters = details.reporters.clone();
				if slash_defer_duration == 0 {
					// apply right away.
					slashing::apply_slash::<T>(unapplied);
					{
						let slash_cost = (6, 5);
						let reward_cost = (2, 2);
						add_db_reads_writes(
							(1 + nominators_len) * slash_cost.0 + reward_cost.0 * reporters_len,
							(1 + nominators_len) * slash_cost.1 + reward_cost.1 * reporters_len,
						);
					}
				} else {
					// defer to end of some `slash_defer_duration` from now.
					<Self as Store>::UnappliedSlashes::mutate(active_era, move |for_later| {
						for_later.push(unapplied)
					});
					add_db_reads_writes(1, 1);
				}
			} else {
				add_db_reads_writes(4 /* fetch_spans */, 5 /* kick_out_if_recent */)
			}
		}

		Ok(consumed_weight)
	}

	fn can_report() -> bool {
		Self::era_election_status().is_closed()
	}
}

impl<T: Config> OnDepositRedeem<T::AccountId, EtpBalance<T>> for Module<T> {
	fn on_deposit_redeem(
		backing: &T::AccountId,
		stash: &T::AccountId,
		amount: EtpBalance<T>,
		start_time: TsInMs,
		months: u8,
	) -> DispatchResult {
		// The timestamp unit is different between Ethereum and Hyperspace
		// Converting from seconds to milliseconds
		let start_time = start_time * 1000;
		let promise_month = months.min(36);
		let expire_time = start_time + promise_month as TsInMs * MONTH_IN_MILLISECONDS;

		if let Some(controller) = Self::bonded(&stash) {
			let mut ledger = Self::ledger(&controller).ok_or(<Error<T>>::NotController)?;

			T::EtpCurrency::transfer(&backing, &stash, amount, KeepAlive)?;

			let StakingLedger {
				active_etp,
				active_deposit_etp,
				deposit_items,
				..
			} = &mut ledger;

			*active_etp = active_etp.saturating_add(amount);
			*active_deposit_etp = active_deposit_etp.saturating_add(amount);
			deposit_items.push(TimeDepositItem {
				value: amount,
				start_time,
				expire_time,
			});

			Self::update_ledger(&controller, &mut ledger);
		} else {
			ensure!(
				!<Bonded<T>>::contains_key(&stash),
				<Error<T>>::AlreadyBonded
			);

			let controller = stash;

			ensure!(
				!<Ledger<T>>::contains_key(controller),
				<Error<T>>::AlreadyPaired
			);

			T::EtpCurrency::transfer(&backing, &stash, amount, KeepAlive)?;

			<Bonded<T>>::insert(&stash, controller);
			<Payee<T>>::insert(&stash, RewardDestination::Stash);

			<frame_system::Module<T>>::inc_consumers(&stash).map_err(|_| Error::<T>::BadState)?;

			let mut ledger = StakingLedger {
				stash: stash.clone(),
				active_etp: amount,
				active_deposit_etp: amount,
				deposit_items: vec![TimeDepositItem {
					value: amount,
					start_time,
					expire_time,
				}],
				claimed_rewards: {
					let current_era = CurrentEra::get().unwrap_or(0);
					let last_reward_era = current_era.saturating_sub(Self::history_depth());
					(last_reward_era..current_era).collect()
				},
				..Default::default()
			};

			Self::update_ledger(controller, &mut ledger);
		};

		Self::deposit_event(RawEvent::BondEtp(amount, start_time, expire_time));

		Ok(())
	}
}

#[allow(deprecated)]
impl<T: Config> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;
	fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {
		if let Call::submit_election_solution_unsigned(_, _, score, era, _) = call {
			// --- substrate ---
			use offchain_election::DEFAULT_LONGEVITY;

			// discard solution not coming from the local OCW.
			match source {
				TransactionSource::Local | TransactionSource::InBlock => { /* allowed */ }
				_ => {
					log!(
						debug,
						"rejecting unsigned transaction because it is not local/in-block."
					);
					return InvalidTransaction::Call.into();
				}
			}

			if let Err(error_with_post_info) = Self::pre_dispatch_checks(*score, *era) {
				let invalid = to_invalid(error_with_post_info);
				log!(
					debug,
					"💸 validate unsigned pre dispatch checks failed due to error #{:?}.",
					invalid,
				);
				return invalid.into();
			}

			log!(
				debug,
				"💸 validateUnsigned succeeded for a solution at era {}.",
				era
			);

			ValidTransaction::with_tag_prefix("StakingOffchain")
				// The higher the score[0], the better a solution is.
				.priority(T::UnsignedPriority::get().saturating_add(score[0].saturated_into()))
				// Defensive only. A single solution can exist in the pool per era. Each validator
				// will run OCW at most once per era, hence there should never exist more than one
				// transaction anyhow.
				.and_provides(era)
				// Note: this can be more accurate in the future. We do something like
				// `era_end_block - current_block` but that is not needed now as we eagerly run
				// offchain workers now and the above should be same as `T::ElectionLookahead`
				// without the need to query more storage in the validation phase. If we randomize
				// offchain worker, then we might re-consider this.
				.longevity(
					TryInto::<u64>::try_into(T::ElectionLookahead::get())
						.unwrap_or(DEFAULT_LONGEVITY),
				)
				// We don't propagate this. This can never the validated at a remote node.
				.propagate(false)
				.build()
		} else {
			InvalidTransaction::Call.into()
		}
	}

	fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
		if let Call::submit_election_solution_unsigned(_, _, score, era, _) = call {
			// IMPORTANT NOTE: These checks are performed in the dispatch call itself, yet we need
			// to duplicate them here to prevent a block producer from putting a previously
			// validated, yet no longer valid solution on chain.
			// OPTIMISATION NOTE: we could skip this in the `submit_election_solution_unsigned`
			// since we already do it here. The signed version needs it though. Yer for now we keep
			// this duplicate check here so both signed and unsigned can use a singular
			// `check_and_replace_solution`.
			Self::pre_dispatch_checks(*score, *era)
				.map(|_| ())
				.map_err(to_invalid)
				.map_err(Into::into)
		} else {
			Err(InvalidTransaction::Call.into())
		}
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T>
where
	T: Config + pallet_authorship::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		Self::reward_by_ids(vec![(author, 20)]);
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		Self::reward_by_ids(vec![
			(<pallet_authorship::Module<T>>::author(), 2),
			(author, 1),
		]);
	}
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(PhantomData<T>);
impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		<Module<T>>::ledger(&controller).map(|l| l.stash)
	}
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
///
/// Active exposure is the exposure of the validator set currently validating, i.e. in
/// `active_era`. It can differ from the latest planned exposure in `current_era`.
pub struct ExposureOf<T>(PhantomData<T>);
impl<T: Config> Convert<T::AccountId, Option<ExposureT<T>>> for ExposureOf<T> {
	fn convert(validator: T::AccountId) -> Option<ExposureT<T>> {
		if let Some(active_era) = <Module<T>>::active_era() {
			Some(<Module<T>>::eras_stakers(active_era.index, &validator))
		} else {
			None
		}
	}
}

/// Filter historical offences out and only allow those from the bonding period.
pub struct FilterHistoricalOffences<T, R> {
	_inner: PhantomData<(T, R)>,
}
impl<T, Reporter, Offender, R, O> ReportOffence<Reporter, Offender, O>
	for FilterHistoricalOffences<Module<T>, R>
where
	T: Config,
	R: ReportOffence<Reporter, Offender, O>,
	O: Offence<Offender>,
{
	fn report_offence(reporters: Vec<Reporter>, offence: O) -> Result<(), OffenceError> {
		// disallow any slashing from before the current bonding period.
		let offence_session = offence.session_index();
		let bonded_eras = BondedEras::get();

		if bonded_eras
			.first()
			.filter(|(_, start)| offence_session >= *start)
			.is_some()
		{
			R::report_offence(reporters, offence)
		} else {
			<Module<T>>::deposit_event(RawEvent::OldSlashingReportDiscarded(offence_session));
			Ok(())
		}
	}

	fn is_known_offence(offenders: &[Offender], time_slot: &O::TimeSlot) -> bool {
		R::is_known_offence(offenders, time_slot)
	}
}

// A value placed in storage that represents the current version of the Staking storage. This value
// is used by the `on_runtime_upgrade` logic to determine whether we run storage migration logic.
// This should match directly with the semantic versions of the Rust crate.
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug)]
enum Releases {
	V1_0_0Ancient,
	V2_0_0,
	V3_0_0,
	V4_0_0,
	V5_0_0,
}
impl Default for Releases {
	fn default() -> Self {
		Releases::V5_0_0
	}
}

/// Indicates the initial status of the staker.
#[derive(RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StakerStatus<AccountId> {
	/// Chilling.
	Idle,
	/// Declared desire in validating or already participating in it.
	Validator,
	/// Nominating for a group of other stakers.
	Nominator(Vec<AccountId>),
}

/// A destination account for payment.
#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
pub enum RewardDestination<AccountId> {
	/// Pay into the stash account, increasing the amount at stake accordingly.
	Staked,
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into the controller account.
	Controller,
	/// Pay into a specified account.
	Account(AccountId),
}
impl<AccountId> Default for RewardDestination<AccountId> {
	fn default() -> Self {
		RewardDestination::Staked
	}
}

/// Mode of era-forcing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Forcing {
	/// Not forcing anything - just let whatever happen.
	NotForcing,
	/// Force a new era, then reset to `NotForcing` as soon as it is done.
	ForceNew,
	/// Avoid a new era indefinitely.
	ForceNone,
	/// Force a new era at the end of all sessions indefinitely.
	ForceAlways,
}
impl Default for Forcing {
	fn default() -> Self {
		Forcing::NotForcing
	}
}

/// Indicate how an election round was computed.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug)]
pub enum ElectionCompute {
	/// Result was forcefully computed on chain at the end of the session.
	OnChain,
	/// Result was submitted and accepted to the chain via a signed transaction.
	Signed,
	/// Result was submitted and accepted to the chain via an unsigned transaction (by an
	/// authority).
	Unsigned,
}

/// The status of the upcoming (offchain) election.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum ElectionStatus<BlockNumber> {
	/// Nothing has and will happen for now. submission window is not open.
	Closed,
	/// The submission window has been open since the contained block number.
	Open(BlockNumber),
}
impl<BlockNumber: PartialEq> ElectionStatus<BlockNumber> {
	pub fn is_open_at(&self, n: BlockNumber) -> bool {
		*self == Self::Open(n)
	}

	pub fn is_closed(&self) -> bool {
		match self {
			Self::Closed => true,
			_ => false,
		}
	}

	pub fn is_open(&self) -> bool {
		!self.is_closed()
	}
}
impl<BlockNumber> Default for ElectionStatus<BlockNumber> {
	fn default() -> Self {
		Self::Closed
	}
}

/// To unify *ETP* and *DNA* balances.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum StakingBalance<EtpBalance, DnaBalance>
where
	EtpBalance: HasCompact,
	DnaBalance: HasCompact,
{
	EtpBalance(EtpBalance),
	DnaBalance(DnaBalance),
}
impl<EtpBalance, DnaBalance> Default for StakingBalance<EtpBalance, DnaBalance>
where
	EtpBalance: Zero + HasCompact,
	DnaBalance: Zero + HasCompact,
{
	fn default() -> Self {
		StakingBalance::EtpBalance(Zero::zero())
	}
}

/// Information regarding the active era (era in used in session).
#[derive(Encode, Decode, RuntimeDebug)]
pub struct ActiveEraInfo {
	/// Index of era.
	pub index: EraIndex,
	/// Moment of start expressed as millisecond from `$UNIX_EPOCH`.
	///
	/// Start can be none if start hasn't been set for the era yet,
	/// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
	start: Option<u64>,
}

/// Reward points of an era. Used to split era total payout between validators.
///
/// This points will be used to reward validators and their respective nominators.
#[derive(PartialEq, Encode, Decode, Default, Debug)]
pub struct EraRewardPoints<AccountId: Ord> {
	/// Total number of points. Equals the sum of reward points for each validator.
	total: RewardPoint,
	/// The reward points earned by a given validator.
	individual: BTreeMap<AccountId, RewardPoint>,
}

/// Preference of what happens regarding validation.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ValidatorPrefs {
	/// Reward that validator takes up-front; only the rest is split between themselves and
	/// nominators.
	#[codec(compact)]
	pub commission: Perbill,
	/// Whether or not this validator is accepting more nominations. If `true`, then no nominator
	/// who is not already nominating this validator may nominate them. By default, validators
	/// are accepting nominations.
	pub blocked: bool,
}
impl Default for ValidatorPrefs {
	fn default() -> Self {
		ValidatorPrefs {
			commission: Perbill::zero(),
			blocked: false,
		}
	}
}

/// The ledger of a (bonded) stash.
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
pub struct StakingLedger<AccountId, EtpBalance, DnaBalance, BlockNumber>
where
	EtpBalance: HasCompact,
	DnaBalance: HasCompact,
{
	/// The stash account whose balance is actually locked and at stake.
	pub stash: AccountId,

	/// The total amount of the stash's *ETP* that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_etp: EtpBalance,
	/// active time-deposit ring
	#[codec(compact)]
	pub active_deposit_etp: EtpBalance,

	/// The total amount of the stash's *DNA* that will be at stake in any forthcoming
	/// rounds.
	#[codec(compact)]
	pub active_dna: DnaBalance,

	/// If you deposit *ETP* for a minimum period,
	/// you can get *DNA* as bonus which can also be used for staking.
	pub deposit_items: Vec<TimeDepositItem<EtpBalance>>,

	/// The staking lock on *ETP* balance, use for updating hyperspace balance module's lock
	pub etp_staking_lock: StakingLock<EtpBalance, BlockNumber>,
	/// The staking lock on *DNA* balance, use for updating hyperspace balance module's lock
	pub dna_staking_lock: StakingLock<DnaBalance, BlockNumber>,

	/// List of eras for which the stakers behind a validator have claimed rewards. Only updated
	/// for validators.
	pub claimed_rewards: Vec<EraIndex>,
}
impl<AccountId, EtpBalance, DnaBalance, BlockNumber>
	StakingLedger<AccountId, EtpBalance, DnaBalance, BlockNumber>
where
	EtpBalance: Copy + AtLeast32BitUnsigned + Saturating,
	DnaBalance: Copy + AtLeast32BitUnsigned + Saturating,
	BlockNumber: Copy + PartialOrd,
	TsInMs: PartialOrd,
{
	pub fn etp_locked_amount_at(&self, at: BlockNumber) -> EtpBalance {
		self.etp_staking_lock.locked_amount(at)
	}

	pub fn dna_locked_amount_at(&self, at: BlockNumber) -> DnaBalance {
		self.dna_staking_lock.locked_amount(at)
	}

	/// Re-bond funds that were scheduled for unlocking.
	fn rebond(&mut self, plan_to_rebond_etp: EtpBalance, plan_to_rebond_dna: DnaBalance) {
		fn update<Balance, _M>(
			bonded: &mut Balance,
			lock: &mut StakingLock<Balance, _M>,
			plan_to_rebond: Balance,
		) where
			Balance: Copy + AtLeast32BitUnsigned + Saturating,
		{
			let mut rebonded = Balance::zero();

			while let Some(Unbonding { amount, .. }) = lock.unbondings.last_mut() {
				let new_rebonded = rebonded.saturating_add(*amount);

				if new_rebonded <= plan_to_rebond {
					rebonded = new_rebonded;
					*bonded = bonded.saturating_add(*amount);

					lock.unbondings.pop();
				} else {
					let diff = plan_to_rebond.saturating_sub(rebonded);

					rebonded = rebonded.saturating_add(diff);
					*bonded = bonded.saturating_add(diff);
					*amount = amount.saturating_sub(diff);
				}

				if rebonded >= plan_to_rebond {
					break;
				}
			}
		}

		update(
			&mut self.active_etp,
			&mut self.etp_staking_lock,
			plan_to_rebond_etp,
		);
		update(
			&mut self.active_dna,
			&mut self.dna_staking_lock,
			plan_to_rebond_dna,
		);
	}

	/// Slash the validator for a given amount of balance. This can grow the value
	/// of the slash in the case that the validator has less than `minimum_balance`
	/// active funds. Returns the amount of funds actually slashed.
	///
	/// Slashes from `active` funds first, and then `unlocking`, starting with the
	/// chunks that are closest to unlocking.
	pub fn slash(
		&mut self,
		slash_etp: EtpBalance,
		slash_dna: DnaBalance,
		bn: BlockNumber,
		ts: TsInMs,
	) -> (EtpBalance, DnaBalance) {
		let slash_out_of = |active_etp: &mut EtpBalance,
		                    active_deposit_etp: &mut EtpBalance,
		                    deposit_item: &mut Vec<TimeDepositItem<EtpBalance>>,
		                    active_dna: &mut DnaBalance,
		                    slash_etp: &mut EtpBalance,
		                    slash_dna: &mut DnaBalance| {
			let slashable_active_etp = (*slash_etp).min(*active_etp);
			let slashable_active_dna = (*slash_dna).min(*active_dna);

			if !slashable_active_etp.is_zero() {
				let slashable_normal_etp = *active_etp - *active_deposit_etp;
				if let Some(mut slashable_deposit_etp) =
					slashable_active_etp.checked_sub(&slashable_normal_etp)
				{
					*active_deposit_etp -= slashable_deposit_etp;

					deposit_item.drain_filter(|item| {
						if ts >= item.expire_time {
							true
						} else {
							if slashable_deposit_etp.is_zero() {
								false
							} else {
								if let Some(new_slashable_deposit_etp) =
									slashable_deposit_etp.checked_sub(&item.value)
								{
									slashable_deposit_etp = new_slashable_deposit_etp;
									true
								} else {
									item.value -= sp_std::mem::replace(
										&mut slashable_deposit_etp,
										Zero::zero(),
									);
									false
								}
							}
						}
					});
				}

				*active_etp -= slashable_active_etp;
				*slash_etp -= slashable_active_etp;
			}

			if !slashable_active_dna.is_zero() {
				*active_dna -= slashable_active_dna;
				*slash_dna -= slashable_active_dna;
			}
		};

		let (mut apply_slash_etp, mut apply_slash_dna) = (slash_etp, slash_dna);
		let StakingLedger {
			active_etp,
			active_deposit_etp,
			deposit_items,
			active_dna,
			etp_staking_lock,
			dna_staking_lock,
			..
		} = self;

		slash_out_of(
			active_etp,
			active_deposit_etp,
			deposit_items,
			active_dna,
			&mut apply_slash_etp,
			&mut apply_slash_dna,
		);

		if !apply_slash_etp.is_zero() {
			etp_staking_lock.unbondings.drain_filter(|lock| {
				if bn >= lock.until {
					true
				} else {
					if apply_slash_etp.is_zero() {
						false
					} else {
						if apply_slash_etp >= lock.amount {
							apply_slash_etp -= lock.amount;
							true
						} else {
							lock.amount -=
								sp_std::mem::replace(&mut apply_slash_etp, Zero::zero());
							false
						}
					}
				}
			});
		}
		if !apply_slash_dna.is_zero() {
			dna_staking_lock.unbondings.drain_filter(|lock| {
				if bn >= lock.until {
					true
				} else {
					if apply_slash_dna.is_zero() {
						false
					} else {
						if apply_slash_dna > lock.amount {
							apply_slash_dna -= lock.amount;

							true
						} else {
							lock.amount -=
								sp_std::mem::replace(&mut apply_slash_dna, Zero::zero());
							false
						}
					}
				}
			});
		}

		(slash_etp - apply_slash_etp, slash_dna - apply_slash_dna)
	}
}

/// The *ETP* under deposit.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct TimeDepositItem<EtpBalance: HasCompact> {
	#[codec(compact)]
	pub value: EtpBalance,
	#[codec(compact)]
	pub start_time: TsInMs,
	#[codec(compact)]
	pub expire_time: TsInMs,
}

/// A record of the nominations made by a specific account.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Nominations<AccountId> {
	/// The targets of nomination.
	pub targets: Vec<AccountId>,
	/// The era the nominations were submitted.
	///
	/// Except for initial nominations which are considered submitted at era 0.
	pub submitted_in: EraIndex,
	/// Whether the nominations have been suppressed. This can happen due to slashing of the
	/// validators, or other events that might invalidate the nomination.
	///
	/// NOTE: this for future proofing and is thus far not used.
	pub suppressed: bool,
}

/// A snapshot of the stake backing a single validator in the system.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct Exposure<AccountId, EtpBalance, DnaBalance>
where
	EtpBalance: HasCompact,
	DnaBalance: HasCompact,
{
	/// The validator's own stash that is exposed.
	#[codec(compact)]
	pub own_etp_balance: EtpBalance,
	#[codec(compact)]
	pub own_dna_balance: DnaBalance,
	pub own_power: Power,
	/// The total balance backing this validator.
	pub total_power: Power,
	/// The portions of nominators stashes that are exposed.
	pub others: Vec<IndividualExposure<AccountId, EtpBalance, DnaBalance>>,
}

/// The amount of exposure (to slashing) than an individual nominator has.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, RuntimeDebug)]
pub struct IndividualExposure<AccountId, EtpBalance, DnaBalance>
where
	EtpBalance: HasCompact,
	DnaBalance: HasCompact,
{
	/// The stash account of the nominator in question.
	who: AccountId,
	/// Amount of funds exposed.
	#[codec(compact)]
	etp_balance: EtpBalance,
	#[codec(compact)]
	dna_balance: DnaBalance,
	power: Power,
}

/// A pending slash record. The value of the slash has been computed but not applied yet,
/// rather deferred for several eras.
#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct UnappliedSlash<AccountId, EtpBalance, DnaBalance> {
	/// The stash ID of the offending validator.
	validator: AccountId,
	/// The validator's own slash.
	own: slashing::RK<EtpBalance, DnaBalance>,
	/// All other slashed stakers and amounts.
	others: Vec<(AccountId, slashing::RK<EtpBalance, DnaBalance>)>,
	/// Reporters of the offence; bounty payout recipients.
	reporters: Vec<AccountId>,
	/// The amount of payout.
	payout: slashing::RK<EtpBalance, DnaBalance>,
}

/// Some indications about the size of the election. This must be submitted with the solution.
///
/// Note that these values must reflect the __total__ number, not only those that are present in the
/// solution. In short, these should be the same size as the size of the values dumped in
/// `SnapshotValidators` and `SnapshotNominators`.
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, Default)]
pub struct ElectionSize {
	/// Number of validators in the snapshot of the current election round.
	#[codec(compact)]
	pub validators: ValidatorIndex,
	/// Number of nominators in the snapshot of the current election round.
	#[codec(compact)]
	pub nominators: NominatorIndex,
}

/// The result of an election round.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct ElectionResult<AccountId, EtpBalance: HasCompact, DnaBalance: HasCompact> {
	/// Flat list of validators who have been elected.
	elected_stashes: Vec<AccountId>,
	/// Flat list of new exposures, to be updated in the [`Exposure`] storage.
	exposures: Vec<(AccountId, Exposure<AccountId, EtpBalance, DnaBalance>)>,
	/// Type of the result. This is kept on chain only to track and report the best score's
	/// submission type. An optimisation could remove this.
	compute: ElectionCompute,
}

/// Check that list is sorted and has no duplicates.
fn is_sorted_and_unique(list: &Vec<u32>) -> bool {
	list.windows(2).all(|w| w[0] < w[1])
}

/// convert a DispatchErrorWithPostInfo to a custom InvalidTransaction with the inner code being the
/// error number.
fn to_invalid(error_with_post_info: DispatchErrorWithPostInfo) -> InvalidTransaction {
	let error = error_with_post_info.error;
	let error_number = match error {
		DispatchError::Module { error, .. } => error,
		_ => 0,
	};
	InvalidTransaction::Custom(error_number)
}
