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

//! Tests for the module.
//!
//! These tests are migrated from Substrate `dd97b1478b31a4715df7e88a5ebc6664425fb6c6`
//! for upgrade usages, do not **add** functions to this file unless you are doing the
//! upgrading work :-P
//!
//! + If you want to add some external tests for this crate, please gather
//! them into `tests_local.rs`.
//!
//! + If you want to delete some functions, please left some comments
//! explaining why you delete them.

mod offchain_election {
	// --- hyperspace ---
	use crate::{mock::*, *};

	/// setup a new set of validators and nominator storage items independent of the parent mock
	/// file. This produces a edge graph that can be reduced.
	pub fn build_offchain_election_test_ext() {
		for i in (10..=40).step_by(10) {
			// Note: we respect the convention of the mock (10, 11 pairs etc.) since these accounts
			// have corresponding keys in session which makes everything more ergonomic and
			// realistic.
			bond_validator(i + 1, i, StakingBalance::EtpBalance(100));
		}

		let mut voter = 1;
		bond_nominator(
			voter,
			1000 + voter,
			StakingBalance::EtpBalance(100),
			vec![11],
		);
		voter = 2;
		bond_nominator(
			voter,
			1000 + voter,
			StakingBalance::EtpBalance(100),
			vec![11, 11],
		);
		voter = 3;
		bond_nominator(
			voter,
			1000 + voter,
			StakingBalance::EtpBalance(100),
			vec![21, 41],
		);
		voter = 4;
		bond_nominator(
			voter,
			1000 + voter,
			StakingBalance::EtpBalance(100),
			vec![21, 31, 41],
		);
		voter = 5;
		bond_nominator(
			voter,
			1000 + voter,
			StakingBalance::EtpBalance(100),
			vec![21, 31, 41],
		);
	}
}

// --- substrate ---
use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, OnFinalize, OnInitialize, ReservableCurrency},
	StorageMap,
};
use sp_runtime::{assert_eq_error_rate, traits::BadOrigin};
use sp_staking::offence::OffenceDetails;
use substrate_test_utils::assert_eq_uvec;
// --- hyperspace ---
use crate::{
	mock::{AccountId, Balance, *},
	*,
};
use hyperspace_support::balance::lock::*;

#[test]
fn force_unstake_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Account 11 is stashed and locked, and account 10 is the controller
		assert_eq!(Staking::bonded(&11), Some(10));
		// Adds 2 slashing spans
		add_slash(&11);
		// Cant transfer
		assert_noop!(
			Etp::transfer(Origin::signed(11), 1, 10),
			EtpError::LiquidityRestrictions,
		);
		// Force unstake requires root.
		assert_noop!(Staking::force_unstake(Origin::signed(11), 11, 2), BadOrigin);
		// Force unstake needs correct number of slashing spans (for weight calculation)
		assert_noop!(
			Staking::force_unstake(Origin::root(), 11, 0),
			StakingError::IncorrectSlashingSpans
		);
		// We now force them to unstake
		assert_ok!(Staking::force_unstake(Origin::root(), 11, 2));
		// No longer bonded.
		assert_eq!(Staking::bonded(&11), None);
		// Transfer works.
		assert_ok!(Etp::transfer(Origin::signed(11), 1, 10));
	});
}

#[test]
fn kill_stash_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Account 11 is stashed and locked, and account 10 is the controller
		assert_eq!(Staking::bonded(&11), Some(10));
		// Adds 2 slashing spans
		add_slash(&11);
		// Only can kill a stash account
		assert_noop!(Staking::kill_stash(&12, 0), StakingError::NotStash);
		// Respects slashing span count
		assert_noop!(
			Staking::kill_stash(&11, 0),
			StakingError::IncorrectSlashingSpans
		);
		// Correct inputs, everything works
		assert_ok!(Staking::kill_stash(&11, 2));
		// No longer bonded.
		assert_eq!(Staking::bonded(&11), None);
	});
}

#[test]
fn basic_setup_works() {
	// Verifies initial conditions of mock
	ExtBuilder::default().build_and_execute(|| {
		// Account 11 is stashed and locked, and account 10 is the controller
		assert_eq!(Staking::bonded(&11), Some(10));
		// Account 21 is stashed and locked, and account 20 is the controller
		assert_eq!(Staking::bonded(&21), Some(20));
		// Account 1 is not a stashed
		assert_eq!(Staking::bonded(&1), None);

		// Account 10 controls the stash from account 11, which is 100 * balance_factor units
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000,
				etp_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![]
				},
				..Default::default()
			})
		);
		// Account 20 controls the stash from account 21, which is 200 * balance_factor units
		assert_eq!(
			Staking::ledger(&20),
			Some(StakingLedger {
				stash: 21,
				active_etp: 1000,
				etp_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![]
				},
				..Default::default()
			})
		);
		// Account 1 does not control any stash
		assert_eq!(Staking::ledger(&1), None);

		// ValidatorPrefs are default
		assert_eq_uvec!(
			<Validators<Test>>::iter().collect::<Vec<_>>(),
			vec![
				(31, ValidatorPrefs::default()),
				(21, ValidatorPrefs::default()),
				(11, ValidatorPrefs::default())
			]
		);

		assert_eq!(
			Staking::ledger(100),
			Some(StakingLedger {
				stash: 101,
				active_etp: 500,
				etp_staking_lock: StakingLock {
					staking_amount: 500,
					unbondings: vec![]
				},
				..Default::default()
			})
		);
		assert_eq!(Staking::nominators(101).unwrap().targets, vec![11, 21]);

		assert_eq!(
			Staking::eras_stakers(active_era(), 11),
			Exposure {
				own_etp_balance: 1000,
				own_dna_balance: 0,
				own_power: Staking::power_of(&11),
				// Allow error rate 1
				total_power: Staking::power_of(&11)
					+ Staking::currency_to_power(125, Staking::etp_pool())
					+ 1,
				others: vec![IndividualExposure {
					who: 101,
					etp_balance: 125,
					dna_balance: 0,
					// Allow error rate 1
					power: Staking::currency_to_power(125, Staking::etp_pool()) + 1,
				}]
			}
		);

		assert_eq!(
			Staking::eras_stakers(active_era(), 21),
			Exposure {
				own_etp_balance: 1000,
				own_dna_balance: 0,
				own_power: Staking::power_of(&21),
				// Allow error rate 1
				total_power: Staking::power_of(&21)
					+ Staking::currency_to_power(375, Staking::etp_pool())
					- 1,
				others: vec![IndividualExposure {
					who: 101,
					// Allow error rate 1
					etp_balance: 375 - 1,
					dna_balance: 0,
					// Allow error rate 1
					power: Staking::currency_to_power(375, Staking::etp_pool()) - 1,
				}]
			}
		);

		// initial total stake = 1125 + 1375
		assert_eq!(
			Staking::eras_total_stake(active_era()),
			Staking::power_of(&11) + Staking::power_of(&21) + Staking::power_of(&101)
		);

		// The number of validators required.
		assert_eq!(Staking::validator_count(), 2);

		// Initial Era and session
		assert_eq!(Staking::current_era(), Some(0));

		// Account 10 has `balance_factor` free balance
		assert_eq!(Etp::free_balance(10), 1);
		assert_eq!(Etp::free_balance(10), 1);

		// New era is not being forced
		assert_eq!(Staking::force_era(), Forcing::NotForcing);
	});
}

#[test]
fn change_controller_works() {
	ExtBuilder::default().build_and_execute(|| {
		// 10 and 11 are bonded as stash controller.
		assert_eq!(Staking::bonded(&11), Some(10));

		// 10 can control 11 who is initially a validator.
		assert_ok!(Staking::chill(Origin::signed(10)));

		// change controller
		assert_ok!(Staking::set_controller(Origin::signed(11), 5));

		assert_eq!(Staking::bonded(&11), Some(5));
		start_active_era(1);

		// 10 is no longer in control.
		assert_noop!(
			Staking::validate(Origin::signed(10), ValidatorPrefs::default()),
			StakingError::NotController,
		);
		assert_ok!(Staking::validate(
			Origin::signed(5),
			ValidatorPrefs::default()
		));
	})
}

#[test]
fn rewards_should_work() {
	ExtBuilder::default()
		.nominate(true)
		.sessions_per_era(3)
		.build_and_execute(|| {
			let init_balance_10 = Etp::free_balance(&10);
			let init_balance_11 = Etp::free_balance(&11);
			let init_balance_20 = Etp::free_balance(&20);
			let init_balance_21 = Etp::free_balance(&21);
			let init_balance_100 = Etp::free_balance(&100);
			let init_balance_101 = Etp::free_balance(&101);

			// Set payees
			<Payee<Test>>::insert(11, RewardDestination::Controller);
			<Payee<Test>>::insert(21, RewardDestination::Controller);
			<Payee<Test>>::insert(101, RewardDestination::Controller);

			Staking::reward_by_ids(vec![(11, 50)]);
			Staking::reward_by_ids(vec![(11, 50)]);
			// This is the second validator of the current elected set.
			Staking::reward_by_ids(vec![(21, 50)]);

			// Compute total payout now for whole duration of the session.
			let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
			let maximum_payout = maximum_payout_for_duration(reward_time_per_era());

			start_session(1);

			assert_eq!(Etp::free_balance(&10), init_balance_10);
			assert_eq!(Etp::free_balance(&11), init_balance_11);
			assert_eq!(Etp::free_balance(&20), init_balance_20);
			assert_eq!(Etp::free_balance(&21), init_balance_21);
			assert_eq!(Etp::free_balance(&100), init_balance_100);
			assert_eq!(Etp::free_balance(&101), init_balance_101);
			assert_eq_uvec!(Session::validators(), vec![11, 21]);
			assert_eq!(
				Staking::eras_reward_points(active_era()),
				EraRewardPoints {
					total: 50 * 3,
					individual: vec![(11, 100), (21, 50)].into_iter().collect(),
				}
			);
			let part_for_10 = Perbill::from_rational_approximation::<u32>(1000, 1125);
			let part_for_20 = Perbill::from_rational_approximation::<u32>(1000, 1375);
			let part_for_100_from_10 = Perbill::from_rational_approximation::<u32>(125, 1125);
			let part_for_100_from_20 = Perbill::from_rational_approximation::<u32>(375, 1375);

			start_session(2);
			start_session(3);

			assert_eq!(active_era(), 1);
			assert_eq!(
				ETP_REWARD_REMAINDER_UNBALANCED.with(|v| *v.borrow()),
				maximum_payout - total_payout_0,
			);
			assert_eq!(
				*staking_events().last().unwrap(),
				RawEvent::EraPayout(0, total_payout_0, maximum_payout - total_payout_0)
			);
			make_all_reward_payment(0);

			assert_eq_error_rate!(
				Etp::free_balance(&10),
				init_balance_10 + part_for_10 * total_payout_0 * 2 / 3,
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&11), init_balance_11, MICRO);
			assert_eq_error_rate!(
				Etp::free_balance(&20),
				init_balance_20 + part_for_20 * total_payout_0 * 1 / 3,
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&21), init_balance_21, MICRO);
			assert_eq_error_rate!(
				Etp::free_balance(&100),
				init_balance_100
					+ part_for_100_from_10 * total_payout_0 * 2 / 3
					+ part_for_100_from_20 * total_payout_0 * 1 / 3,
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&101), init_balance_101, MICRO);

			assert_eq_uvec!(Session::validators(), vec![11, 21]);
			Staking::reward_by_ids(vec![(11, 1)]);

			// Compute total payout now for whole duration as other parameter won't change
			let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());

			start_active_era(2);
			assert_eq_error_rate!(
				ETP_REWARD_REMAINDER_UNBALANCED.with(|v| *v.borrow()),
				maximum_payout * 2 - total_payout_0 - total_payout_1,
				MICRO,
			);
			if let RawEvent::EraPayout(a, b, c) = *staking_events().last().unwrap() {
				assert_eq!(a, 1);
				assert_eq!(b, total_payout_1);
				assert_eq_error_rate!(c, maximum_payout - total_payout_1, MICRO);
			} else {
				panic!();
			}
			make_all_reward_payment(1);

			assert_eq_error_rate!(
				Etp::free_balance(&10),
				init_balance_10 + part_for_10 * (total_payout_0 * 2 / 3 + total_payout_1),
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&11), init_balance_11, MICRO);
			assert_eq_error_rate!(
				Etp::free_balance(&20),
				init_balance_20 + part_for_20 * total_payout_0 * 1 / 3,
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&21), init_balance_21, MICRO);
			assert_eq_error_rate!(
				Etp::free_balance(&100),
				init_balance_100
					+ part_for_100_from_10 * (total_payout_0 * 2 / 3 + total_payout_1)
					+ part_for_100_from_20 * total_payout_0 * 1 / 3,
				MICRO,
			);
			assert_eq_error_rate!(Etp::free_balance(&101), init_balance_101, MICRO);
		});
}

#[test]
fn staking_should_work() {
	ExtBuilder::default()
		.nominate(false)
		.fair(false) // to give 20 more staked value
		.build()
		.execute_with(|| {
			// remember + compare this along with the test.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// put some money in account that we'll use.
			for i in 1..5 {
				let _ = Etp::make_free_balance_be(&i, 2000);
			}

			// --- Block 2:
			start_session(2);
			// add a new candidate for being a validator. account 3 controlled by 4.
			assert_ok!(Staking::bond(
				Origin::signed(3),
				4,
				StakingBalance::EtpBalance(1500),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::validate(
				Origin::signed(4),
				ValidatorPrefs::default()
			));

			// No effects will be seen so far.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// --- Block 3:
			start_session(3);

			// No effects will be seen so far. Era has not been yet triggered.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// --- Block 4: the validators will now be queued.
			start_session(4);
			assert_eq!(active_era(), 1);

			// --- Block 5: the validators are still in queue.
			start_session(5);

			// --- Block 6: the validators will now be changed.
			start_session(6);

			assert_eq_uvec!(validator_controllers(), vec![20, 4]);
			// --- Block 4: Unstake 4 as a validator, freeing up the balance stashed in 3
			// 4 will chill
			Staking::chill(Origin::signed(4)).unwrap();

			// --- Block 7: nothing. 4 is still there.
			start_session(7);
			assert_eq_uvec!(validator_controllers(), vec![20, 4]);

			// --- Block 8:
			start_session(8);

			// --- Block 9: 4 will not be a validator.
			start_session(9);
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// Note: the stashed value of 4 is still lock
			assert_eq!(
				Staking::ledger(&4),
				Some(StakingLedger {
					stash: 3,
					active_etp: 1500,
					etp_staking_lock: StakingLock {
						staking_amount: 1500,
						unbondings: vec![]
					},
					claimed_rewards: vec![0],
					..Default::default()
				})
			);
			// e.g. it cannot reserve more than 500 that it has free from the total 2000
			assert_noop!(Etp::reserve(&3, 501), EtpError::LiquidityRestrictions);
			assert_ok!(Etp::reserve(&3, 409));
		});
}

#[test]
fn blocking_and_kicking_works() {
	ExtBuilder::default()
		.minimum_validator_count(1)
		.validator_count(4)
		.nominate(true)
		.num_validators(3)
		.build()
		.execute_with(|| {
			// block validator 10/11
			assert_ok!(Staking::validate(
				Origin::signed(10),
				ValidatorPrefs {
					blocked: true,
					..Default::default()
				}
			));
			// attempt to nominate from 100/101...
			assert_ok!(Staking::nominate(Origin::signed(100), vec![11]));
			// should have worked since we're already nominated them
			assert_eq!(<Nominators<Test>>::get(&101).unwrap().targets, vec![11]);
			// kick the nominator
			assert_ok!(Staking::kick(Origin::signed(10), vec![101]));
			// should have been kicked now
			assert!(<Nominators<Test>>::get(&101).unwrap().targets.is_empty());
			// attempt to nominate from 100/101...
			assert_noop!(
				Staking::nominate(Origin::signed(100), vec![11]),
				StakingError::BadTarget
			);
		});
}

#[test]
fn less_than_needed_candidates_works() {
	ExtBuilder::default()
		.minimum_validator_count(1)
		.validator_count(4)
		.nominate(false)
		.num_validators(3)
		.build()
		.execute_with(|| {
			assert_eq!(Staking::validator_count(), 4);
			assert_eq!(Staking::minimum_validator_count(), 1);
			assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

			start_active_era(1);

			// Previous set is selected. NO election algorithm is even executed.
			assert_eq_uvec!(validator_controllers(), vec![30, 20, 10]);

			// But the exposure is updated in a simple way. No external votes exists.
			// This is purely self-vote.
			assert!(<ErasStakers<Test>>::iter_prefix_values(active_era())
				.all(|exposure| exposure.others.is_empty()));
		});
}

#[test]
fn no_candidate_emergency_condition() {
	ExtBuilder::default()
		.minimum_validator_count(1)
		.validator_count(15)
		.num_validators(4)
		.validator_pool(true)
		.nominate(false)
		.build()
		.execute_with(|| {
			// initial validators
			assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
			let prefs = ValidatorPrefs {
				commission: Perbill::one(),
				..Default::default()
			};
			<Staking as Store>::Validators::insert(11, prefs.clone());

			// set the minimum validator count.
			<Staking as Store>::MinimumValidatorCount::put(10);

			let _ = Staking::chill(Origin::signed(10));

			// trigger era
			start_active_era(1);

			// Previous ones are elected. chill is invalidates. TODO: #2494
			assert_eq_uvec!(validator_controllers(), vec![10, 20, 30, 40]);
			// Though the validator preferences has been removed.
			assert!(Staking::validators(11) != prefs);
		});
}

#[test]
fn nominating_and_rewards_should_work() {
	ExtBuilder::default()
		.nominate(false)
		.validator_pool(true)
		.build()
		.execute_with(|| {
			// initial validators -- everyone is actually even.
			assert_eq_uvec!(validator_controllers(), vec![40, 30]);

			// Set payee to controller
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Controller
			));
			assert_ok!(Staking::set_payee(
				Origin::signed(20),
				RewardDestination::Controller
			));
			assert_ok!(Staking::set_payee(
				Origin::signed(30),
				RewardDestination::Controller
			));
			assert_ok!(Staking::set_payee(
				Origin::signed(40),
				RewardDestination::Controller
			));

			// give the man some money
			let initial_balance = 1000;
			for i in [1, 2, 3, 4, 5, 10, 11, 20, 21].iter() {
				let _ = Etp::make_free_balance_be(i, initial_balance);
			}

			// bond two account pairs and state interest in nomination.
			// 2 will nominate for 10, 20, 30
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 21, 31]));
			// 4 will nominate for 10, 20, 40
			assert_ok!(Staking::bond(
				Origin::signed(3),
				4,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::nominate(Origin::signed(4), vec![11, 21, 41]));

			// the total reward for era 0
			let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
			Staking::reward_by_ids(vec![(41, 1)]);
			Staking::reward_by_ids(vec![(31, 1)]);

			start_active_era(1);

			// 10 and 20 have more votes, they will be chosen.
			assert_eq_uvec!(validator_controllers(), vec![20, 10]);

			// OLD validators must have already received some rewards.
			make_all_reward_payment(0);
			assert_eq!(Etp::free_balance(&40), 1 + total_payout_0 / 2);
			assert_eq!(Etp::free_balance(&30), 1 + total_payout_0 / 2);

			// ------ check the staked value of all parties.

			// 30 and 40 are not chosen anymore
			assert_eq!(
				<ErasStakers<Test>>::iter_prefix_values(active_era()).count(),
				2,
			);
			assert_eq!(
				Staking::eras_stakers(active_era(), 11),
				Exposure {
					own_etp_balance: 1000,
					own_dna_balance: 0,
					own_power: Staking::currency_to_power(1000, Staking::etp_pool()),
					total_power: Staking::currency_to_power(1000, Staking::etp_pool())
						+ Staking::currency_to_power(800, Staking::etp_pool()),
					others: vec![
						IndividualExposure {
							who: 3,
							etp_balance: 400,
							dna_balance: 0,
							power: Staking::currency_to_power(400, Staking::etp_pool()),
						},
						IndividualExposure {
							who: 1,
							etp_balance: 400,
							dna_balance: 0,
							power: Staking::currency_to_power(400, Staking::etp_pool()),
						},
					],
				},
			);
			assert_eq!(
				Staking::eras_stakers(active_era(), 21),
				Exposure {
					own_etp_balance: 1000,
					own_dna_balance: 0,
					own_power: Staking::currency_to_power(1000, Staking::etp_pool()),
					total_power: Staking::currency_to_power(1000, Staking::etp_pool())
						+ Staking::currency_to_power(1200, Staking::etp_pool()),
					others: vec![
						IndividualExposure {
							who: 3,
							// expect 600, error rate 1
							etp_balance: 599,
							dna_balance: 0,
							power: Staking::currency_to_power(600, Staking::etp_pool()),
						},
						IndividualExposure {
							who: 1,
							// expect 600, error rate 1
							etp_balance: 599,
							dna_balance: 0,
							power: Staking::currency_to_power(600, Staking::etp_pool()),
						},
					],
				},
			);
			// the total reward for era 1
			let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
			Staking::reward_by_ids(vec![(21, 2)]);
			Staking::reward_by_ids(vec![(11, 1)]);

			start_active_era(2);

			// nothing else will happen, era ends and rewards are paid again,
			// it is expected that nominators will also be paid. See below

			make_all_reward_payment(1);
			let payout_for_10 = total_payout_1 / 3;
			let payout_for_20 = 2 * total_payout_1 / 3;
			// Nominator 2: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
			assert_eq_error_rate!(
				Etp::free_balance(&2),
				initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
				MICRO,
			);
			// Nominator 4: has [400/1800 ~ 2/9 from 10] + [600/2200 ~ 3/11 from 20]'s reward. ==> 2/9 + 3/11
			assert_eq_error_rate!(
				Etp::free_balance(&4),
				initial_balance + (2 * payout_for_10 / 9 + 3 * payout_for_20 / 11),
				MICRO,
			);

			// Validator 10: got 800 / 1800 external stake => 8/18 =? 4/9 => Validator's share = 5/9
			assert_eq_error_rate!(
				Etp::free_balance(&10),
				initial_balance + 5 * payout_for_10 / 9,
				MICRO
			);
			// Validator 20: got 1200 / 2200 external stake => 12/22 =? 6/11 => Validator's share = 5/11
			assert_eq_error_rate!(
				Etp::free_balance(&20),
				initial_balance + 5 * payout_for_20 / 11,
				MICRO,
			);
		});
}

#[test]
fn nominators_also_get_slashed_pro_rata() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);

		let slash_percent = Perbill::from_percent(5);
		let initial_exposure = Staking::eras_stakers(active_era(), 11);
		// 101 is a nominator for 11
		assert_eq!(initial_exposure.others.first().unwrap().who, 101);

		// staked values;
		let nominator_stake = Staking::ledger(100).unwrap().active_etp;
		let nominator_balance = etp_balances(&101).0;
		let validator_stake = Staking::ledger(10).unwrap().active_etp;
		let validator_balance = etp_balances(&11).0;
		let exposed_stake = initial_exposure.total_power;
		let exposed_validator = initial_exposure.own_power;
		let exposed_nominator = initial_exposure.others.first().unwrap().power;

		// 11 goes offline
		on_offence_now(
			&[OffenceDetails {
				offender: (11, initial_exposure.clone()),
				reporters: vec![],
			}],
			&[slash_percent],
		);

		// both stakes must have been decreased.
		assert!(Staking::ledger(100).unwrap().active_etp < nominator_stake);
		assert!(Staking::ledger(10).unwrap().active_etp < validator_stake);

		let slash_amount = slash_percent * exposed_stake;
		let validator_share =
			Perbill::from_rational_approximation(exposed_validator, exposed_stake) * slash_amount;
		let nominator_share =
			Perbill::from_rational_approximation(exposed_nominator, exposed_stake) * slash_amount;

		// both slash amounts need to be positive for the test to make sense.
		assert!(validator_share > 0);
		assert!(nominator_share > 0);

		// both stakes must have been decreased pro-rata.
		let nominator_share =
			Perbill::from_rational_approximation(nominator_share, exposed_stake) * nominator_stake;
		let validator_share =
			Perbill::from_rational_approximation(validator_share, exposed_stake) * validator_stake;

		assert_eq_error_rate!(
			Staking::ledger(100).unwrap().active_etp,
			nominator_stake - nominator_share,
			10,
		);
		assert_eq_error_rate!(
			Staking::ledger(10).unwrap().active_etp,
			validator_stake - validator_share,
			10,
		);
		assert_eq_error_rate!(
			etp_balances(&101).0, // free balance
			nominator_balance - nominator_share,
			10,
		);
		assert_eq_error_rate!(
			etp_balances(&11).0, // free balance
			validator_balance - validator_share,
			10,
		);
		// Because slashing happened.
		assert!(is_disabled(10));
	});
}

#[test]
fn double_staking_should_fail() {
	// should test (in the same order):
	// * an account already bonded as stash cannot be be stashed again.
	// * an account already bonded as stash cannot nominate.
	// * an account already bonded as controller can nominate.
	ExtBuilder::default().build().execute_with(|| {
		let arbitrary_value = 5;
		// 2 = controller, 1 stashed => ok
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::EtpBalance(arbitrary_value),
			RewardDestination::default(),
			0,
		));
		// 4 = not used so far, 1 stashed => not allowed.
		assert_noop!(
			Staking::bond(
				Origin::signed(1),
				4,
				StakingBalance::EtpBalance(arbitrary_value),
				RewardDestination::default(),
				0,
			),
			StakingError::AlreadyBonded,
		);
		// 1 = stashed => attempting to nominate should fail.
		assert_noop!(
			Staking::nominate(Origin::signed(1), vec![1]),
			StakingError::NotController,
		);
		// 2 = controller  => nominating should work.
		assert_ok!(Staking::nominate(Origin::signed(2), vec![1]));
	});
}

#[test]
fn double_controlling_should_fail() {
	// should test (in the same order):
	// * an account already bonded as controller CANNOT be reused as the controller of another account.
	ExtBuilder::default().build_and_execute(|| {
		let arbitrary_value = 5;
		// 2 = controller, 1 stashed => ok
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::EtpBalance(arbitrary_value),
			RewardDestination::default(),
			0,
		));
		// 2 = controller, 3 stashed (Note that 2 is reused.) => no-op
		assert_noop!(
			Staking::bond(
				Origin::signed(3),
				2,
				StakingBalance::EtpBalance(arbitrary_value),
				RewardDestination::default(),
				0,
			),
			StakingError::AlreadyPaired,
		);
	});
}

#[test]
fn session_and_eras_work_simple() {
	ExtBuilder::default().period(1).build_and_execute(|| {
		assert_eq!(active_era(), 0);
		assert_eq!(current_era(), 0);
		assert_eq!(Session::current_index(), 1);
		assert_eq!(System::block_number(), 1);

		// Session 1: this is basically a noop. This has already been started.
		start_session(1);
		assert_eq!(active_era(), 0);
		assert_eq!(System::block_number(), 1);

		// Session 2: No change.
		start_session(2);
		assert_eq!(active_era(), 0);
		assert_eq!(System::block_number(), 2);

		// Session 3: Era increment.
		start_session(3);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 3);

		// Session 4: No change.
		start_session(4);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 4);

		// Session 5: No change.
		start_session(5);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 5);

		// Session 6: Era increment.
		start_session(6);
		assert_eq!(active_era(), 2);
		assert_eq!(System::block_number(), 6);
	});
}

#[test]
fn session_and_eras_work_complex() {
	ExtBuilder::default().period(5).build_and_execute(|| {
		assert_eq!(active_era(), 0);
		assert_eq!(Session::current_index(), 0);
		assert_eq!(System::block_number(), 1);

		start_session(1);
		assert_eq!(Session::current_index(), 1);
		assert_eq!(active_era(), 0);
		assert_eq!(System::block_number(), 5);

		start_session(2);
		assert_eq!(Session::current_index(), 2);
		assert_eq!(active_era(), 0);
		assert_eq!(System::block_number(), 10);

		start_session(3);
		assert_eq!(Session::current_index(), 3);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 15);

		start_session(4);
		assert_eq!(Session::current_index(), 4);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 20);

		start_session(5);
		assert_eq!(Session::current_index(), 5);
		assert_eq!(active_era(), 1);
		assert_eq!(System::block_number(), 25);

		start_session(6);
		assert_eq!(Session::current_index(), 6);
		assert_eq!(active_era(), 2);
		assert_eq!(System::block_number(), 30);
	});
}

#[test]
fn forcing_new_era_works() {
	ExtBuilder::default().build_and_execute(|| {
		// normal flow of session.
		start_session(1);
		assert_eq!(active_era(), 0);

		start_session(2);
		assert_eq!(active_era(), 0);

		start_session(3);
		assert_eq!(active_era(), 1);

		// no era change.
		ForceEra::put(Forcing::ForceNone);

		start_session(4);
		assert_eq!(active_era(), 1);

		start_session(5);
		assert_eq!(active_era(), 1);

		start_session(6);
		assert_eq!(active_era(), 1);

		start_session(7);
		assert_eq!(active_era(), 1);

		// back to normal.
		// this immediately starts a new session.
		ForceEra::put(Forcing::NotForcing);

		start_session(8);
		assert_eq!(active_era(), 1);

		start_session(9);
		assert_eq!(active_era(), 2);
		// forceful change
		ForceEra::put(Forcing::ForceAlways);

		start_session(10);
		assert_eq!(active_era(), 2);

		start_session(11);
		assert_eq!(active_era(), 3);

		start_session(12);
		assert_eq!(active_era(), 4);

		// just one forceful change
		ForceEra::put(Forcing::ForceNew);
		start_session(13);
		assert_eq!(active_era(), 5);
		assert_eq!(ForceEra::get(), Forcing::NotForcing);

		start_session(14);
		assert_eq!(active_era(), 6);

		start_session(15);
		assert_eq!(active_era(), 6);
	});
}

#[test]
fn cannot_transfer_staked_balance() {
	// Tests that a stash account cannot transfer funds
	ExtBuilder::default().nominate(false).build_and_execute(|| {
		// Confirm account 11 is stashed
		assert_eq!(Staking::bonded(&11), Some(10));
		// Confirm account 11 has some free balance
		assert_eq!(Etp::free_balance(11), 1000);
		// Confirm account 11 (via controller 10) is totally staked
		{
			let expo = Staking::eras_stakers(active_era(), &11);
			assert_eq!(
				expo.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance),
				1000,
			);
		}
		// Confirm account 11 cannot transfer as a result
		assert_noop!(
			Etp::transfer(Origin::signed(11), 20, 1),
			EtpError::LiquidityRestrictions,
		);

		// Give account 11 extra free balance
		let _ = Etp::make_free_balance_be(&11, 10000);
		// Confirm that account 11 can now transfer some balance
		assert_ok!(Etp::transfer(Origin::signed(11), 20, 1));
	});
}

#[test]
fn cannot_transfer_staked_balance_2() {
	// Tests that a stash account cannot transfer funds
	// Same test as above but with 20, and more accurate.
	// 21 has 2000 free balance but 1000 at stake
	ExtBuilder::default()
		.nominate(false)
		.fair(true)
		.build_and_execute(|| {
			// Confirm account 21 is stashed
			assert_eq!(Staking::bonded(&21), Some(20));
			// Confirm account 21 has some free balance
			assert_eq!(Etp::free_balance(21), 2000);
			// Confirm account 21 (via controller 20) is totally staked
			{
				let expo = Staking::eras_stakers(active_era(), &21);
				assert_eq!(
					expo.others
						.iter()
						.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance),
					1000,
				);
			}
			// Confirm account 21 can transfer at most 1000
			assert_noop!(
				Etp::transfer(Origin::signed(21), 20, 1001),
				EtpError::LiquidityRestrictions,
			);
			assert_ok!(Etp::transfer(Origin::signed(21), 20, 1000));
		});
}

#[test]
fn cannot_reserve_staked_balance() {
	// Checks that a bonded account cannot reserve balance from free balance
	ExtBuilder::default().build_and_execute(|| {
		// Confirm account 11 is stashed
		assert_eq!(Staking::bonded(&11), Some(10));
		// Confirm account 11 has some free balance
		assert_eq!(Etp::free_balance(11), 1000);
		// Confirm account 11 (via controller 10) is totally staked
		assert_eq!(
			Staking::eras_stakers(active_era(), &11).own_etp_balance,
			1000,
		);
		// Confirm account 11 cannot transfer as a result
		assert_noop!(Etp::reserve(&11, 1), EtpError::LiquidityRestrictions);

		// Give account 11 extra free balance
		let _ = Etp::make_free_balance_be(&11, 10000);
		// Confirm account 11 can now reserve balance
		assert_ok!(Etp::reserve(&11, 1));
	});
}

#[test]
fn reward_destination_works() {
	// Rewards go to the correct destination as determined in Payee
	ExtBuilder::default().nominate(false).build_and_execute(|| {
		// Check that account 11 is a validator
		assert!(Session::validators().contains(&11));
		// Check the balance of the validator account
		assert_eq!(Etp::free_balance(10), 1);
		// Check the balance of the stash account
		assert_eq!(Etp::free_balance(11), 1000);
		// Check how much is at stake
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000,
				etp_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![],
				},
				..Default::default()
			}),
		);

		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
		Staking::reward_by_ids(vec![(11, 1)]);

		start_active_era(1);
		make_all_reward_payment(0);

		// Check that RewardDestination is Staked (default)
		assert_eq!(Staking::payee(&11), RewardDestination::Staked);
		// Check that reward went to the stash account of validator
		assert_eq!(Etp::free_balance(11), 1000 + total_payout_0);
		// Check that amount at stake increased accordingly
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000 + total_payout_0,
				etp_staking_lock: StakingLock {
					staking_amount: 1000 + total_payout_0,
					unbondings: vec![],
				},
				claimed_rewards: vec![0],
				..Default::default()
			}),
		);

		//Change RewardDestination to Stash
		<Payee<Test>>::insert(&11, RewardDestination::Stash);

		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
		Staking::reward_by_ids(vec![(11, 1)]);

		start_active_era(2);
		make_all_reward_payment(1);

		// Check that RewardDestination is Stash
		assert_eq!(Staking::payee(&11), RewardDestination::Stash);
		// Check that reward went to the stash account
		assert_eq!(
			Etp::free_balance(11),
			1000 + total_payout_0 + total_payout_1
		);
		// Record this value
		let recorded_stash_balance = 1000 + total_payout_0 + total_payout_1;
		// Check that amount at stake is NOT increased
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000 + total_payout_0,
				etp_staking_lock: StakingLock {
					staking_amount: 1000 + total_payout_0,
					unbondings: vec![],
				},
				claimed_rewards: vec![0, 1],
				..Default::default()
			}),
		);

		// Change RewardDestination to Controller
		<Payee<Test>>::insert(&11, RewardDestination::Controller);

		// Check controller balance
		assert_eq!(Etp::free_balance(10), 1);

		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_2 = current_total_payout_for_duration(reward_time_per_era());
		Staking::reward_by_ids(vec![(11, 1)]);

		start_active_era(3);
		make_all_reward_payment(2);

		// Check that RewardDestination is Controller
		assert_eq!(Staking::payee(&11), RewardDestination::Controller);
		// Check that reward went to the controller account
		assert_eq!(Etp::free_balance(10), 1 + total_payout_2);
		// Check that amount at stake is NOT increased
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000 + total_payout_0,
				etp_staking_lock: StakingLock {
					staking_amount: 1000 + total_payout_0,
					unbondings: vec![],
				},
				claimed_rewards: vec![0, 1, 2],
				..Default::default()
			}),
		);
		// Check that amount in staked account is NOT increased.
		assert_eq!(Etp::free_balance(11), recorded_stash_balance);
	});
}

#[test]
fn validator_payment_prefs_work() {
	// Test that validator preferences are correctly honored
	// Note: unstake threshold is being directly tested in slashing tests.
	// This test will focus on validator payment.
	ExtBuilder::default().build_and_execute(|| {
		let commission = Perbill::from_percent(40);
		<Validators<Test>>::insert(
			&11,
			ValidatorPrefs {
				commission: commission.clone(),
				..Default::default()
			},
		);

		// Reward controller so staked ratio doesn't change.
		<Payee<Test>>::insert(&11, RewardDestination::Controller);
		<Payee<Test>>::insert(&101, RewardDestination::Controller);

		start_active_era(1);
		mock::make_all_reward_payment(0);

		let balance_era_1_10 = Etp::free_balance(&10);
		let balance_era_1_100 = Etp::free_balance(&100);

		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
		let exposure_1 = Staking::eras_stakers(active_era(), 11);
		Staking::reward_by_ids(vec![(11, 1)]);

		start_active_era(2);
		make_all_reward_payment(1);

		let taken_cut = commission * total_payout_1;
		let shared_cut = total_payout_1 - taken_cut;
		let reward_of_10 = shared_cut * exposure_1.own_power as Balance
			/ exposure_1.total_power as Balance
			+ taken_cut;
		let reward_of_100 =
			shared_cut * exposure_1.others[0].power as Balance / exposure_1.total_power as Balance;
		assert_eq_error_rate!(
			Etp::free_balance(&10),
			balance_era_1_10 + reward_of_10,
			MICRO
		);
		assert_eq_error_rate!(
			Etp::free_balance(&100),
			balance_era_1_100 + reward_of_100,
			MICRO
		);
	});
}

#[test]
fn bond_extra_works() {
	// Tests that extra `free_balance` in the stash can be added to stake
	// NOTE: this tests only verifies `StakingLedger` for correct updates
	// See `bond_extra_and_withdraw_unbonded_works` for more details and updates on `Exposure`.
	ExtBuilder::default().build_and_execute(|| {
		// Check that account 10 is a validator
		assert!(<Validators<Test>>::contains_key(11));
		// Check that account 10 is bonded to account 11
		assert_eq!(Staking::bonded(&11), Some(10));
		// Check how much is at stake
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000,
				etp_staking_lock: StakingLock {
					staking_amount: 1000,
					unbondings: vec![],
				},
				..Default::default()
			})
		);

		// Give account 11 some large free balance greater than total
		let _ = Etp::make_free_balance_be(&11, 1000000);

		// Call the bond_extra function from controller, add only 100
		assert_ok!(Staking::bond_extra(
			Origin::signed(11),
			StakingBalance::EtpBalance(100),
			0,
		));
		// There should be 100 more `total` and `active` in the ledger
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000 + 100,
				etp_staking_lock: StakingLock {
					staking_amount: 1000 + 100,
					unbondings: vec![],
				},
				..Default::default()
			}),
		);

		// Call the bond_extra function with a large number, should handle it
		assert_ok!(Staking::bond_extra(
			Origin::signed(11),
			StakingBalance::EtpBalance(Balance::max_value()),
			0,
		));
		// The full amount of the funds should now be in the total and active
		assert_eq!(
			Staking::ledger(&10),
			Some(StakingLedger {
				stash: 11,
				active_etp: 1000000,
				etp_staking_lock: StakingLock {
					staking_amount: 1000000,
					unbondings: vec![],
				},
				..Default::default()
			}),
		);
	});
}

// #[deprecated]
// #[test]
// fn bond_extra_and_withdraw_unbonded_works() {}

#[test]
fn too_many_unbond_calls_should_not_work() {
	ExtBuilder::default().build_and_execute(|| {
		// locked at block 1 until 16
		for _ in 0..MAX_UNLOCKING_CHUNKS - 1 {
			assert_ok!(Staking::unbond(
				Origin::signed(10),
				StakingBalance::EtpBalance(1)
			));
		}

		run_to_block(15);

		// locked at block 15 until 30
		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::EtpBalance(1)
		));
		// can't do more.
		assert_noop!(
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(1)),
			StakingError::NoMoreChunks,
		);

		run_to_block(29);

		// Can add again.
		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::EtpBalance(1)
		));
		assert_eq!(
			Staking::ledger(&10)
				.unwrap()
				.etp_staking_lock
				.unbondings
				.len(),
			2
		);
	})
}

#[test]
fn rebond_works() {
	// * Should test
	// * Given an account being bonded [and chosen as a validator](not mandatory)
	// * it can unbond a portion of its funds from the stash account.
	// * it can re-bond a portion of the funds scheduled to unlock.
	ExtBuilder::default()
		.nominate(false)
		.build()
		.execute_with(|| {
			// Set payee to controller. avoids confusion
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Controller
			));

			// Give account 11 some large free balance greater than total
			let _ = Etp::make_free_balance_be(&11, 1000000);

			// confirm that 10 is a normal validator and gets paid at the end of the era.
			start_active_era(1);

			// Initial state of 10
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				})
			);

			start_active_era(2);
			assert_eq!(active_era(), 2);

			// Try to rebond some funds. We get an error since no fund is unbonded.
			assert_noop!(
				Staking::rebond(Origin::signed(10), 500, 0),
				StakingError::NoUnlockChunk,
			);

			// Unbond almost all of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(900)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 100,
					etp_staking_lock: StakingLock {
						staking_amount: 100,
						unbondings: vec![Unbonding {
							amount: 900,
							until: System::block_number() + BondingDurationInBlockNumber::get(),
						}]
					},
					..Default::default()
				})
			);

			// Re-bond all the funds unbonded.
			Staking::rebond(Origin::signed(10), 900, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				})
			);

			// Unbond almost all of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(900)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 100,
					etp_staking_lock: StakingLock {
						staking_amount: 100,
						unbondings: vec![Unbonding {
							amount: 900,
							until: System::block_number() + BondingDurationInBlockNumber::get(),
						}]
					},
					..Default::default()
				})
			);

			// Re-bond part of the funds unbonded.
			Staking::rebond(Origin::signed(10), 500, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 600,
					etp_staking_lock: StakingLock {
						staking_amount: 600,
						unbondings: vec![Unbonding {
							amount: 400,
							until: System::block_number() + BondingDurationInBlockNumber::get(),
						}]
					},
					..Default::default()
				})
			);

			// Re-bond the remainder of the funds unbonded.
			Staking::rebond(Origin::signed(10), 500, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				})
			);

			// Unbond parts of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(300)).unwrap();
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(300)).unwrap();
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(300)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 100,
					etp_staking_lock: StakingLock {
						staking_amount: 100,
						unbondings: vec![
							Unbonding {
								amount: 300,
								until: System::block_number() + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 300,
								until: System::block_number() + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 300,
								until: System::block_number() + BondingDurationInBlockNumber::get(),
							}
						]
					},
					..Default::default()
				})
			);

			// Re-bond part of the funds unbonded.
			Staking::rebond(Origin::signed(10), 500, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 600,
					etp_staking_lock: StakingLock {
						staking_amount: 600,
						unbondings: vec![
							Unbonding {
								amount: 300,
								until: System::block_number() + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 100,
								until: System::block_number() + BondingDurationInBlockNumber::get(),
							}
						]
					},
					..Default::default()
				})
			);
		});
}

#[test]
fn rebond_is_fifo() {
	// Rebond should proceed by reversing the most recent bond operations.
	ExtBuilder::default()
		.nominate(false)
		.build()
		.execute_with(|| {
			// Set payee to controller. avoids confusion
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Controller
			));

			// Give account 11 some large free balance greater than total
			let _ = Etp::make_free_balance_be(&11, 1000000);

			run_to_block(5);

			// Initial state of 10
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				})
			);

			run_to_block(6);

			// Unbond some of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(400)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 600,
					etp_staking_lock: StakingLock {
						staking_amount: 600,
						unbondings: vec![Unbonding {
							amount: 400,
							until: 6 + bonding_duration_in_blocks(),
						}]
					},
					..Default::default()
				})
			);

			run_to_block(7);

			// Unbond more of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(300)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 300,
					etp_staking_lock: StakingLock {
						staking_amount: 300,
						unbondings: vec![
							Unbonding {
								amount: 400,
								until: 6 + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 300,
								until: 7 + BondingDurationInBlockNumber::get(),
							},
						]
					},
					..Default::default()
				})
			);

			run_to_block(8);

			// Unbond yet more of the funds in stash.
			Staking::unbond(Origin::signed(10), StakingBalance::EtpBalance(200)).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 100,
					etp_staking_lock: StakingLock {
						staking_amount: 100,
						unbondings: vec![
							Unbonding {
								amount: 400,
								until: 6 + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 300,
								until: 7 + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 200,
								until: 8 + BondingDurationInBlockNumber::get(),
							},
						]
					},
					..Default::default()
				})
			);

			// Re-bond half of the unbonding funds.
			Staking::rebond(Origin::signed(10), 400, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 500,
					etp_staking_lock: StakingLock {
						staking_amount: 500,
						unbondings: vec![
							Unbonding {
								amount: 400,
								until: 6 + BondingDurationInBlockNumber::get(),
							},
							Unbonding {
								amount: 100,
								until: 7 + BondingDurationInBlockNumber::get(),
							},
						]
					},
					..Default::default()
				})
			);
		});
}

#[test]
fn reward_to_stake_works() {
	ExtBuilder::default()
		.nominate(false)
		.fair(false)
		.build_and_execute(|| {
			// Confirm validator count is 2
			assert_eq!(Staking::validator_count(), 2);
			// Confirm account 10 and 20 are validators
			assert!(<Validators<Test>>::contains_key(&11) && <Validators<Test>>::contains_key(&21));

			{
				let expo = Staking::eras_stakers(active_era(), 11);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 1000);
			}
			{
				let expo = Staking::eras_stakers(active_era(), 21);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 2000);
			}

			// Give the man some money.
			let _ = Etp::make_free_balance_be(&10, 1000);
			let _ = Etp::make_free_balance_be(&20, 1000);

			// Bypass logic and change current exposure
			<ErasStakers<Test>>::insert(
				0,
				21,
				Exposure {
					own_etp_balance: 69,
					own_dna_balance: 0,
					total_power: Staking::currency_to_power(69, Staking::etp_pool()),
					own_power: Staking::currency_to_power(69, Staking::etp_pool()),
					others: vec![],
				},
			);

			// Now lets lower account 20 stake
			{
				let expo = Staking::eras_stakers(active_era(), 21);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 69);
			}
			<Ledger<Test>>::insert(
				&20,
				StakingLedger {
					stash: 21,
					active_etp: 69,
					..Default::default()
				},
			);

			// Compute total payout now for whole duration as other parameter won't change
			let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());
			Staking::reward_by_ids(vec![(11, 1)]);
			Staking::reward_by_ids(vec![(21, 1)]);

			// New era --> rewards are paid --> stakes are changed
			start_active_era(1);
			make_all_reward_payment(0);

			{
				let expo = Staking::eras_stakers(active_era(), 11);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 1000);
			}
			{
				let expo = Staking::eras_stakers(active_era(), 21);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 69);
			}

			let _11_balance = Etp::free_balance(&11);
			assert_eq!(_11_balance, 1000 + total_payout_0 / 2);

			// Trigger another new era as the info are frozen before the era start.
			start_active_era(2);

			// -- new infos
			{
				let expo = Staking::eras_stakers(active_era(), 11);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 1000 + total_payout_0 / 2);
			}
			{
				let expo = Staking::eras_stakers(active_era(), 21);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 69 + total_payout_0 / 2);
			}
		});
}

#[test]
fn on_free_balance_zero_stash_removes_validator() {
	// Tests that validator storage items are cleaned up when stash is empty
	// Tests that storage items are untouched when controller is empty
	ExtBuilder::default()
		.existential_deposit(10)
		.build_and_execute(|| {
			// Check the balance of the validator account
			assert_eq!(Etp::free_balance(10), 256);
			// Check the balance of the stash account
			assert_eq!(Etp::free_balance(11), 256000);
			// Check these two accounts are bonded
			assert_eq!(Staking::bonded(&11), Some(10));

			// Set some storage items which we expect to be cleaned up
			// Set payee information
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Stash
			));

			// Check storage items that should be cleaned up
			assert!(<Ledger<Test>>::contains_key(&10));
			assert!(<Bonded<Test>>::contains_key(&11));
			assert!(<Validators<Test>>::contains_key(&11));
			assert!(<Payee<Test>>::contains_key(&11));

			// Reduce free_balance of controller to 0
			let _ = Etp::slash(&10, Balance::max_value());

			// Check the balance of the stash account has not been touched
			assert_eq!(Etp::free_balance(11), 256000);
			// Check these two accounts are still bonded
			assert_eq!(Staking::bonded(&11), Some(10));

			// Check storage items have not changed
			assert!(<Ledger<Test>>::contains_key(&10));
			assert!(<Bonded<Test>>::contains_key(&11));
			assert!(<Validators<Test>>::contains_key(&11));
			assert!(<Payee<Test>>::contains_key(&11));

			// Reduce free_balance of stash to 0
			let _ = Etp::slash(&11, Balance::max_value());
			// Check total balance of stash
			assert_eq!(Etp::free_balance(&11), 10);

			// Reap the stash
			assert_ok!(Staking::reap_stash(Origin::none(), 11, 0));

			// Check storage items do not exist
			assert!(!<Ledger<Test>>::contains_key(&10));
			assert!(!<Bonded<Test>>::contains_key(&11));
			assert!(!<Validators<Test>>::contains_key(&11));
			assert!(!<Nominators<Test>>::contains_key(&11));
			assert!(!<Payee<Test>>::contains_key(&11));
		});
}

#[test]
fn on_free_balance_zero_stash_removes_nominator() {
	// Tests that nominator storage items are cleaned up when stash is empty
	// Tests that storage items are untouched when controller is empty
	ExtBuilder::default()
		.existential_deposit(10)
		.build_and_execute(|| {
			// Make 10 a nominator
			assert_ok!(Staking::nominate(Origin::signed(10), vec![20]));
			// Check that account 10 is a nominator
			assert!(<Nominators<Test>>::contains_key(11));
			// Check the balance of the nominator account
			assert_eq!(Etp::free_balance(10), 256);
			// Check the balance of the stash account
			assert_eq!(Etp::free_balance(11), 256000);

			// Set payee information
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Stash
			));

			// Check storage items that should be cleaned up
			assert!(<Ledger<Test>>::contains_key(&10));
			assert!(<Bonded<Test>>::contains_key(&11));
			assert!(<Nominators<Test>>::contains_key(&11));
			assert!(<Payee<Test>>::contains_key(&11));

			// Reduce free_balance of controller to 0
			let _ = Etp::slash(&10, Balance::max_value());
			// Check total balance of account 10
			assert_eq!(Etp::free_balance(&10), 0);

			// Check the balance of the stash account has not been touched
			assert_eq!(Etp::free_balance(11), 256000);
			// Check these two accounts are still bonded
			assert_eq!(Staking::bonded(&11), Some(10));

			// Check storage items have not changed
			assert!(<Ledger<Test>>::contains_key(&10));
			assert!(<Bonded<Test>>::contains_key(&11));
			assert!(<Nominators<Test>>::contains_key(&11));
			assert!(<Payee<Test>>::contains_key(&11));

			// Reduce free_balance of stash to 0
			let _ = Etp::slash(&11, Balance::max_value());
			// Check total balance of stash
			assert_eq!(Etp::free_balance(&11), 10);

			// Reap the stash
			assert_ok!(Staking::reap_stash(Origin::none(), 11, 0));

			// Check storage items do not exist
			assert!(!<Ledger<Test>>::contains_key(&10));
			assert!(!<Bonded<Test>>::contains_key(&11));
			assert!(!<Validators<Test>>::contains_key(&11));
			assert!(!<Nominators<Test>>::contains_key(&11));
			assert!(!<Payee<Test>>::contains_key(&11));
		});
}

#[test]
fn switching_roles() {
	// Test that it should be possible to switch between roles (nominator, validator, idle) with minimal overhead.
	ExtBuilder::default().nominate(false).build_and_execute(|| {
		// Reset reward destination
		for i in &[10, 20] {
			assert_ok!(Staking::set_payee(
				Origin::signed(*i),
				RewardDestination::Controller
			));
		}

		assert_eq_uvec!(validator_controllers(), vec![20, 10]);

		// put some money in account that we'll use.
		for i in 1..7 {
			let _ = Etp::deposit_creating(&i, 5000);
		}

		// add 2 nominators
		assert_ok!(Staking::bond(
			Origin::signed(1),
			2,
			StakingBalance::EtpBalance(2000),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::nominate(Origin::signed(2), vec![11, 5]));

		assert_ok!(Staking::bond(
			Origin::signed(3),
			4,
			StakingBalance::EtpBalance(500),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::nominate(Origin::signed(4), vec![21, 1]));

		// add a new validator candidate
		assert_ok!(Staking::bond(
			Origin::signed(5),
			6,
			StakingBalance::EtpBalance(1000),
			RewardDestination::Controller,
			0,
		));
		assert_ok!(Staking::validate(
			Origin::signed(6),
			ValidatorPrefs::default()
		));

		start_active_era(1);

		// with current nominators 10 and 5 have the most stake
		assert_eq_uvec!(validator_controllers(), vec![6, 10]);

		// 2 decides to be a validator. Consequences:
		assert_ok!(Staking::validate(
			Origin::signed(2),
			ValidatorPrefs::default()
		));
		// new stakes:
		// 10: 1000 self vote
		// 20: 1000 self vote + 250 vote
		// 6 : 1000 self vote
		// 2 : 2000 self vote + 250 vote.
		// Winners: 20 and 2

		start_active_era(2);

		assert_eq_uvec!(validator_controllers(), vec![2, 20]);
	});
}

#[test]
fn wrong_vote_is_null() {
	ExtBuilder::default()
		.nominate(false)
		.validator_pool(true)
		.build_and_execute(|| {
			assert_eq_uvec!(validator_controllers(), vec![40, 30]);

			// put some money in account that we'll use.
			for i in 1..3 {
				let _ = Etp::deposit_creating(&i, 5000);
			}

			// add 1 nominators
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(2000),
				RewardDestination::default(),
				0,
			));
			assert_ok!(Staking::nominate(
				Origin::signed(2),
				vec![
					11, 21, // good votes
					1, 2, 15, 1000, 25 // crap votes. No effect.
				]
			));

			// new block
			start_active_era(1);

			assert_eq_uvec!(validator_controllers(), vec![20, 10]);
		});
}

#[test]
fn bond_with_no_staked_value() {
	// Behavior when someone bonds with no staked value.
	// Particularly when she votes and the candidate is elected.
	ExtBuilder::default()
		.validator_count(3)
		.existential_deposit(5)
		.nominate(false)
		.minimum_validator_count(1)
		.build()
		.execute_with(|| {
			// Can't bond with 1
			assert_noop!(
				Staking::bond(
					Origin::signed(1),
					2,
					StakingBalance::EtpBalance(1),
					RewardDestination::Controller,
					0,
				),
				StakingError::InsufficientValue,
			);
			// bonded with absolute minimum value possible.
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(5),
				RewardDestination::Controller,
				0,
			));
			assert_eq!(
				Etp::locks(&1)[0].locked_amount(Some(System::block_number())),
				5
			);

			// unbonding even 1 will cause all to be unbonded.
			assert_ok!(Staking::unbond(
				Origin::signed(2),
				StakingBalance::EtpBalance(1)
			));
			// FIXME: https://github.com/new-mvs/darwinia-common/issues/121
			// assert!(Staking::ledger(2).is_none());
		});
}

#[test]
fn bond_with_little_staked_value_bounded() {
	ExtBuilder::default()
		.validator_count(3)
		.nominate(false)
		.minimum_validator_count(1)
		.build()
		.execute_with(|| {
			// setup
			assert_ok!(Staking::chill(Origin::signed(30)));
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Controller
			));
			let init_balance_2 = Etp::free_balance(&2);
			let init_balance_10 = Etp::free_balance(&10);

			// Stingy validator.
			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(1),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::validate(
				Origin::signed(2),
				ValidatorPrefs::default()
			));

			// 1 era worth of reward. BUT, we set the timestamp after on_initialize, so outdated by
			// one block.
			let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());

			reward_all_elected();
			start_active_era(1);
			make_all_reward_payment(0);

			// 2 is elected.
			assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
			{
				let expo = Staking::eras_stakers(active_era(), 2);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 0);
			}

			// Old ones are rewarded.
			assert_eq_error_rate!(
				Etp::free_balance(&10),
				init_balance_10 + total_payout_0 / 3,
				MICRO
			);
			// no rewards paid to 2. This was initial election.
			assert_eq!(Etp::free_balance(&2), init_balance_2);

			// reward era 2
			let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
			reward_all_elected();
			start_active_era(2);
			make_all_reward_payment(1);

			assert_eq_uvec!(validator_controllers(), vec![20, 10, 2]);
			{
				let expo = Staking::eras_stakers(active_era(), 2);
				let total_etp = expo
					.others
					.iter()
					.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
				assert_eq!(total_etp, 0);
			}

			// 2 is now rewarded.
			assert_eq_error_rate!(
				Etp::free_balance(&2),
				init_balance_2 + total_payout_1 / 3,
				MICRO
			);
			assert_eq_error_rate!(
				Etp::free_balance(&10),
				init_balance_10 + total_payout_0 / 3 + total_payout_1 / 3,
				MICRO,
			);
		});
}

#[test]
fn bond_with_duplicate_vote_should_be_ignored_by_npos_election() {
	ExtBuilder::default()
		.validator_count(2)
		.nominate(false)
		.minimum_validator_count(1)
		.build()
		.execute_with(|| {
			// disable the nominator
			assert_ok!(Staking::chill(Origin::signed(100)));
			// make stakes equal.
			assert_ok!(Staking::bond_extra(
				Origin::signed(31),
				StakingBalance::EtpBalance(999),
				0
			));

			assert_eq!(
				<Validators<Test>>::iter()
					.map(|(v, _)| (v, Staking::ledger(v - 1).unwrap().active_etp))
					.collect::<Vec<_>>(),
				vec![(31, 1000), (21, 1000), (11, 1000)],
			);
			assert!(<Nominators<Test>>::iter()
				.map(|(n, _)| n)
				.collect::<Vec<_>>()
				.is_empty());

			// give the man some money
			let initial_balance = 1000;
			for i in [1, 2, 3, 4].iter() {
				let _ = Etp::make_free_balance_be(i, initial_balance);
			}

			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0
			));
			assert_ok!(Staking::nominate(
				Origin::signed(2),
				vec![11, 11, 11, 21, 31,]
			));

			assert_ok!(Staking::bond(
				Origin::signed(3),
				4,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0
			));
			assert_ok!(Staking::nominate(Origin::signed(4), vec![21, 31]));

			// winners should be 21 and 31. Otherwise this election is taking duplicates into account.

			let sp_npos_elections::ElectionResult {
				winners,
				assignments,
			} = Staking::do_phragmen::<Perbill>(0).unwrap();
			let winners = sp_npos_elections::to_without_backing(winners);

			assert_eq!(winners, vec![31, 21]);
			// only distribution to 21 and 31.
			assert_eq!(
				assignments
					.iter()
					.find(|a| a.who == 1)
					.unwrap()
					.distribution
					.len(),
				2
			);
		});
}

#[test]
fn bond_with_duplicate_vote_should_be_ignored_by_npos_election_elected() {
	// same as above but ensures that even when the duple is being elected, everything is sane.
	ExtBuilder::default()
		.validator_count(2)
		.nominate(false)
		.minimum_validator_count(1)
		.build()
		.execute_with(|| {
			// disable the nominator
			assert_ok!(Staking::chill(Origin::signed(100)));
			// make stakes equal.
			assert_ok!(Staking::bond_extra(
				Origin::signed(31),
				StakingBalance::EtpBalance(99),
				0
			));

			assert_eq!(
				<Validators<Test>>::iter()
					.map(|(v, _)| (v, Staking::ledger(v - 1).unwrap().active_etp))
					.collect::<Vec<_>>(),
				vec![(31, 100), (21, 1000), (11, 1000)],
			);
			assert!(<Nominators<Test>>::iter()
				.map(|(n, _)| n)
				.collect::<Vec<_>>()
				.is_empty());

			// give the man some money
			let initial_balance = 1000;
			for i in [1, 2, 3, 4].iter() {
				let _ = Etp::make_free_balance_be(i, initial_balance);
			}

			assert_ok!(Staking::bond(
				Origin::signed(1),
				2,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0
			));
			assert_ok!(Staking::nominate(
				Origin::signed(2),
				vec![11, 11, 11, 21, 31,]
			));

			assert_ok!(Staking::bond(
				Origin::signed(3),
				4,
				StakingBalance::EtpBalance(1000),
				RewardDestination::Controller,
				0
			));
			assert_ok!(Staking::nominate(Origin::signed(4), vec![21, 31]));

			// winners should be 21 and 31. Otherwise this election is taking duplicates into account.

			let sp_npos_elections::ElectionResult {
				winners,
				assignments,
			} = Staking::do_phragmen::<Perbill>(0).unwrap();

			let winners = sp_npos_elections::to_without_backing(winners);
			assert_eq!(winners, vec![21, 11]);
			// only distribution to 21 and 31.
			assert_eq!(
				assignments
					.iter()
					.find(|a| a.who == 1)
					.unwrap()
					.distribution
					.len(),
				2
			);
		});
}

#[test]
fn new_era_elects_correct_number_of_validators() {
	ExtBuilder::default()
		.nominate(true)
		.validator_pool(true)
		.fair(true)
		.validator_count(1)
		.build()
		.execute_with(|| {
			assert_eq!(Staking::validator_count(), 1);
			assert_eq!(validator_controllers().len(), 1);

			Session::on_initialize(System::block_number());

			assert_eq!(validator_controllers().len(), 1);
		})
}

#[test]
fn phragmen_should_not_overflow() {
	ExtBuilder::default().nominate(false).build_and_execute(|| {
		let _ = Staking::chill(Origin::signed(10));
		let _ = Staking::chill(Origin::signed(20));

		bond_validator(3, 2, StakingBalance::EtpBalance(CAP));
		bond_validator(5, 4, StakingBalance::DnaBalance(Balance::max_value() - 1));

		bond_nominator(7, 6, StakingBalance::EtpBalance(1), vec![3, 5]);
		bond_nominator(9, 8, StakingBalance::DnaBalance(1), vec![3, 5]);

		start_active_era(1);

		assert_eq_uvec!(validator_controllers(), vec![4, 2]);

		// Saturate.
		assert_eq!(
			Staking::eras_stakers(active_era(), 3).total_power,
			TOTAL_POWER / 2,
		);
		assert_eq!(
			Staking::eras_stakers(active_era(), 5).total_power,
			TOTAL_POWER / 2,
		);
	})
}

#[test]
fn reward_validator_slashing_validator_does_not_overflow() {
	ExtBuilder::default().build_and_execute(|| {
		let stake = u64::max_value() as Balance * 2;
		let reward_slash = u64::max_value() as Balance * 2;

		// Assert multiplication overflows in balance arithmetic.
		assert!(stake.checked_mul(reward_slash).is_none());

		// Set staker
		let _ = Etp::make_free_balance_be(&11, stake);

		let exposure = Exposure::<AccountId, Balance, Balance> {
			own_etp_balance: stake,
			own_dna_balance: 0,
			own_power: Staking::currency_to_power(stake, Staking::etp_pool()),
			total_power: Staking::currency_to_power(stake, Staking::etp_pool()),
			others: vec![],
		};
		let reward = EraRewardPoints::<AccountId> {
			total: 1,
			individual: vec![(11, 1)].into_iter().collect(),
		};

		// Check reward
		<ErasRewardPoints<Test>>::insert(0, reward);
		<ErasStakers<Test>>::insert(0, 11, &exposure);
		<ErasStakersClipped<Test>>::insert(0, 11, exposure);
		<ErasValidatorReward<Test>>::insert(0, stake);
		let _ = Etp::deposit_creating(&Staking::account_id(), stake);
		assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 0));
		assert_eq!(Etp::free_balance(&11), stake * 2);

		// Set staker
		let _ = Etp::make_free_balance_be(&11, stake);
		let _ = Etp::make_free_balance_be(&2, stake);

		// only slashes out of bonded stake are applied. without this line,
		// it is 0.
		Staking::bond(
			Origin::signed(2),
			20000,
			StakingBalance::EtpBalance(stake - 1),
			RewardDestination::default(),
			0,
		)
		.unwrap();
		// Override exposure of 11
		<ErasStakers<Test>>::insert(
			0,
			&11,
			Exposure {
				own_etp_balance: 1,
				total_power: Staking::currency_to_power(stake, Staking::etp_pool()),
				own_dna_balance: 0,
				own_power: Staking::currency_to_power(1, Staking::etp_pool()),
				others: vec![IndividualExposure {
					who: 2,
					etp_balance: stake - 1,
					dna_balance: 0,
					power: Staking::currency_to_power(stake - 1, Staking::etp_pool()),
				}],
			},
		);

		// Check slashing
		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(100)],
		);

		assert_eq_error_rate!(Etp::free_balance(&11), stake - 1000, MICRO);
		assert_eq!(Etp::free_balance(&2), 1);
	})
}

#[test]
fn reward_from_authorship_event_handler_works() {
	ExtBuilder::default().build_and_execute(|| {
		use pallet_authorship::EventHandler;

		assert_eq!(<pallet_authorship::Module<Test>>::author(), 11);

		Staking::note_author(11);
		Staking::note_uncle(21, 1);

		// Rewarding the same two times works.
		Staking::note_uncle(11, 1);

		// Not mandatory but must be coherent with rewards
		assert_eq_uvec!(Session::validators(), vec![11, 21]);

		// 21 is rewarded as an uncle producer
		// 11 is rewarded as a block producer and uncle referencer and uncle producer
		assert_eq!(
			<ErasRewardPoints<Test>>::get(active_era()),
			EraRewardPoints {
				individual: vec![(11, 20 + 2 * 2 + 1), (21, 1)].into_iter().collect(),
				total: 26,
			},
		);
	})
}

#[test]
fn add_reward_points_fns_works() {
	ExtBuilder::default().build_and_execute(|| {
		// Not mandatory but must be coherent with rewards
		assert_eq!(Session::validators(), vec![21, 11]);

		Staking::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

		Staking::reward_by_ids(vec![(21, 1), (11, 1), (11, 1)]);

		assert_eq!(
			<ErasRewardPoints<Test>>::get(active_era()),
			EraRewardPoints {
				individual: vec![(11, 4), (21, 2)].into_iter().collect(),
				total: 6,
			},
		);
	})
}

#[test]
fn unbonded_balance_is_not_slashable() {
	ExtBuilder::default().build_and_execute(|| {
		// total amount staked is slashable.
		assert_eq!(Staking::stake_of(&11).0, 1000);

		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::EtpBalance(800)
		));

		// only the active portion.
		assert_eq!(Staking::stake_of(&11).0, 200);
	})
}

#[test]
fn era_is_always_same_length() {
	// This ensures that the sessions is always of the same length if there is no forcing no
	// session changes.
	ExtBuilder::default().build_and_execute(|| {
		let sessions_per_era = <SessionsPerEra as Get<SessionIndex>>::get();

		start_active_era(1);
		assert_eq!(
			Staking::eras_start_session_index(current_era()).unwrap(),
			sessions_per_era
		);

		start_active_era(2);
		assert_eq!(
			Staking::eras_start_session_index(current_era()).unwrap(),
			sessions_per_era * 2u32
		);

		let session = Session::current_index();
		ForceEra::put(Forcing::ForceNew);
		advance_session();
		advance_session();
		assert_eq!(current_era(), 3);
		assert_eq!(
			Staking::eras_start_session_index(current_era()).unwrap(),
			session + 2
		);

		start_active_era(4);
		assert_eq!(
			Staking::eras_start_session_index(active_era()).unwrap(),
			session + 2u32 + sessions_per_era
		);
	});
}

#[test]
fn offence_forces_new_era() {
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(Staking::force_era(), Forcing::NotForcing);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(5)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceNew);
	});
}

#[test]
fn offence_ensures_new_era_without_clobbering() {
	ExtBuilder::default().build_and_execute(|| {
		assert_ok!(Staking::force_new_era_always(Origin::root()));
		assert_eq!(Staking::force_era(), Forcing::ForceAlways);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(5)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceAlways);
	});
}

#[test]
fn offence_deselects_validator_even_when_slash_is_zero() {
	ExtBuilder::default().build_and_execute(|| {
		assert!(Session::validators().contains(&11));
		assert!(<Validators<Test>>::contains_key(11));

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(0)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceNew);
		assert!(!<Validators<Test>>::contains_key(11));

		start_active_era(1);

		assert!(!Session::validators().contains(&11));
		assert!(!<Validators<Test>>::contains_key(11));
	});
}

#[test]
fn slashing_performed_according_exposure() {
	// This test checks that slashing is performed according the exposure (or more precisely,
	// historical exposure), not the current balance.
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(
			Staking::eras_stakers(active_era(), 11).own_etp_balance,
			1000,
		);

		// Handle an offence with a historical exposure.
		on_offence_now(
			&[OffenceDetails {
				offender: (
					11,
					Exposure {
						total_power: Staking::currency_to_power(500, Staking::etp_pool()),
						own_etp_balance: 500,
						own_dna_balance: 0,
						own_power: 0,
						others: vec![],
					},
				),
				reporters: vec![],
			}],
			&[Perbill::from_percent(50)],
		);

		// The stash account should be slashed for 250 (50% of 500).
		assert_eq!(Etp::free_balance(11), 1000 - 250);
	});
}

#[test]
fn slash_in_old_span_does_not_deselect() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);

		assert!(<Validators<Test>>::contains_key(11));
		assert!(Session::validators().contains(&11));

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(0)],
		);

		assert_eq!(Staking::force_era(), Forcing::ForceNew);
		assert!(!<Validators<Test>>::contains_key(11));

		start_active_era(2);

		Staking::validate(Origin::signed(10), Default::default()).unwrap();
		assert_eq!(Staking::force_era(), Forcing::NotForcing);
		assert!(<Validators<Test>>::contains_key(11));
		assert!(!Session::validators().contains(&11));

		start_active_era(3);

		// this staker is in a new slashing span now, having re-registered after
		// their prior slash.

		on_offence_in_era(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(0)],
			1,
		);

		// not forcing for zero-slash and previous span.
		assert_eq!(Staking::force_era(), Forcing::NotForcing);
		assert!(<Validators<Test>>::contains_key(11));
		assert!(Session::validators().contains(&11));

		on_offence_in_era(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			// NOTE: A 100% slash here would clean up the account, causing de-registration.
			&[Perbill::from_percent(95)],
			1,
		);

		// or non-zero.
		assert_eq!(Staking::force_era(), Forcing::NotForcing);
		assert!(<Validators<Test>>::contains_key(11));
		assert!(Session::validators().contains(&11));
	});
}

#[test]
fn reporters_receive_their_slice() {
	// This test verifies that the reporters of the offence receive their slice from the slashed
	// amount, but less and less if they submit multiple reports in one span.
	ExtBuilder::default().build_and_execute(|| {
		// The reporters' reward is calculated from the total exposure.
		let initial_balance = 1125;

		{
			let expo = Staking::eras_stakers(active_era(), 11);
			let total_etp = expo
				.others
				.iter()
				.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
			assert_eq!(total_etp, initial_balance);
		}

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![1, 2],
			}],
			&[Perbill::from_percent(50)],
		);

		// F1 * (reward_proportion * slash - 0)
		// 50% * (10% * initial_balance / 2)
		let reward = (initial_balance / 20) / 2;
		let reward_each = reward / 2; // split into two pieces.
		assert_eq!(Etp::free_balance(&1), 10 + reward_each);
		assert_eq!(Etp::free_balance(&2), 20 + reward_each);
	});
}

#[test]
fn subsequent_reports_in_same_span_pay_out_less() {
	// This test verifies that the reporters of the offence receive their slice from the slashed
	// amount.
	ExtBuilder::default().build_and_execute(|| {
		// The reporters' reward is calculated from the total exposure.
		let initial_balance = 1125;

		{
			let expo = Staking::eras_stakers(active_era(), 11);
			let total_etp = expo
				.others
				.iter()
				.fold(expo.own_etp_balance, |acc, i| acc + i.etp_balance);
			assert_eq!(total_etp, initial_balance);
		}

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![1],
			}],
			&[Perbill::from_percent(20)],
		);

		// F1 * (reward_proportion * slash - 0)
		// 50% * (10% * initial_balance * 20%)
		let reward = (initial_balance / 5) / 20;
		assert_eq!(Etp::free_balance(&1), 10 + reward);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![1],
			}],
			&[Perbill::from_percent(50)],
		);

		let prior_payout = reward;

		// F1 * (reward_proportion * slash - prior_payout)
		// 50% * (10% * (initial_balance / 2) - prior_payout)
		let reward = ((initial_balance / 20) - prior_payout) / 2;
		assert_eq!(Etp::free_balance(&1), 10 + prior_payout + reward);
	});
}

#[test]
fn invulnerables_are_not_slashed() {
	// For invulnerable validators no slashing is performed.
	ExtBuilder::default()
		.invulnerables(vec![11])
		.build_and_execute(|| {
			assert_eq!(Etp::free_balance(&11), 1000);
			assert_eq!(Etp::free_balance(&21), 2000);

			let exposure = Staking::eras_stakers(active_era(), 21);
			let initial_balance = Staking::stake_of(&21).0;

			let nominator_balances: Vec<_> = exposure
				.others
				.iter()
				.map(|o| Etp::free_balance(&o.who))
				.collect();

			on_offence_now(
				&[
					OffenceDetails {
						offender: (11, Staking::eras_stakers(active_era(), 11)),
						reporters: vec![],
					},
					OffenceDetails {
						offender: (21, Staking::eras_stakers(active_era(), 21)),
						reporters: vec![],
					},
				],
				&[Perbill::from_percent(50), Perbill::from_percent(20)],
			);

			// The validator 11 hasn't been slashed, but 21 has been.
			assert_eq!(Etp::free_balance(&11), 1000);
			// 2000 - (0.2 * initial_balance)
			assert_eq!(Etp::free_balance(&21), 2000 - (2 * initial_balance / 10));

			// ensure that nominators were slashed as well.
			for (initial_balance, other) in nominator_balances.into_iter().zip(exposure.others) {
				assert_eq_error_rate!(
					Etp::free_balance(&other.who),
					initial_balance - (2 * other.etp_balance / 10),
					1
				);
			}
		});
}

#[test]
fn do_not_slash_if_fraction_is_zero() {
	// Don't slash if the fraction is zero.
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(Etp::free_balance(11), 1000);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(0)],
		);

		// The validator hasn't been slashed. The new era is not forced.
		assert_eq!(Etp::free_balance(11), 1000);
		assert_eq!(Staking::force_era(), Forcing::ForceNew);
	});
}

#[test]
fn only_slash_for_max_in_era() {
	// multiple slashes within one era are only applied if it is more than any previous slash in the
	// same era.
	ExtBuilder::default().build_and_execute(|| {
		assert_eq!(Etp::free_balance(11), 1000);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(50)],
		);

		// The validator has been slashed and has been force-chilled.
		assert_eq!(Etp::free_balance(11), 500);
		assert_eq!(Staking::force_era(), Forcing::ForceNew);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(25)],
		);

		// The validator has not been slashed additionally.
		assert_eq!(Etp::free_balance(11), 500);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(60)],
		);

		// The validator got slashed 10% more.
		assert_eq!(Etp::free_balance(11), 400);
	})
}

#[test]
fn garbage_collection_after_slashing() {
	// ensures that `SlashingSpans` and `SpanSlash` of an account is removed after reaping.
	ExtBuilder::default()
		.existential_deposit(2)
		.build_and_execute(|| {
			assert_eq!(Etp::free_balance(11), 256_000);

			on_offence_now(
				&[OffenceDetails {
					offender: (11, Staking::eras_stakers(active_era(), 11)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(10)],
			);

			assert_eq!(Etp::free_balance(11), 256_000 - 25_600);
			assert!(<Staking as Store>::SlashingSpans::get(&11).is_some());
			assert_eq!(
				<Staking as Store>::SpanSlash::get(&(11, 0))
					.amount_slashed()
					.r,
				25_600,
			);

			on_offence_now(
				&[OffenceDetails {
					offender: (11, Staking::eras_stakers(active_era(), 11)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(100)],
			);

			// validator and nominator slash in era are garbage-collected by era change,
			// so we don't test those here.

			assert_eq!(Etp::free_balance(11), 2);
			assert_eq!(Etp::total_balance(&11), 2);

			let slashing_spans = <Staking as crate::Store>::SlashingSpans::get(&11).unwrap();
			assert_eq!(slashing_spans.iter().count(), 2);

			// reap_stash respects num_slashing_spans so that weight is accurate
			assert_noop!(
				Staking::reap_stash(Origin::none(), 11, 0),
				StakingError::IncorrectSlashingSpans
			);
			assert_ok!(Staking::reap_stash(Origin::none(), 11, 2));

			assert!(<Staking as Store>::SlashingSpans::get(&11).is_none());
			assert_eq!(
				<Staking as Store>::SpanSlash::get(&(11, 0)).amount_slashed(),
				&Zero::zero(),
			);
		})
}

#[test]
fn garbage_collection_on_window_pruning() {
	// ensures that `ValidatorSlashInEra` and `NominatorSlashInEra` are cleared after
	// `BondingDuration`.
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);

		assert_eq!(Etp::free_balance(11), 1000);
		let now = active_era();

		let exposure = Staking::eras_stakers(now, 11);
		assert_eq!(Etp::free_balance(101), 2000);
		let nominated_value = exposure
			.others
			.iter()
			.find(|o| o.who == 101)
			.unwrap()
			.etp_balance;

		on_offence_now(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(now, 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(10)],
		);

		assert_eq!(Etp::free_balance(11), 900);
		assert_eq!(Etp::free_balance(101), 2000 - (nominated_value / 10));

		assert!(<Staking as Store>::ValidatorSlashInEra::get(&now, &11).is_some());
		assert!(<Staking as Store>::NominatorSlashInEra::get(&now, &101).is_some());

		// + 1 because we have to exit the bonding window.
		for era in (0..(BondingDurationInEra::get() + 1)).map(|offset| offset + now + 1) {
			assert!(<Staking as Store>::ValidatorSlashInEra::get(&now, &11).is_some());
			assert!(<Staking as Store>::NominatorSlashInEra::get(&now, &101).is_some());

			start_active_era(era);
		}

		assert!(<Staking as Store>::ValidatorSlashInEra::get(&now, &11).is_none());
		assert!(<Staking as Store>::NominatorSlashInEra::get(&now, &101).is_none());
	})
}

#[test]
fn slashing_nominators_by_span_max() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);
		start_active_era(2);
		start_active_era(3);

		assert_eq!(Etp::free_balance(11), 1000);
		assert_eq!(Etp::free_balance(21), 2000);
		assert_eq!(Etp::free_balance(101), 2000);
		assert_eq!(Staking::stake_of(&21).0, 1000);

		let exposure_11 = Staking::eras_stakers(active_era(), 11);
		let exposure_21 = Staking::eras_stakers(active_era(), 21);
		let nominated_value_11 = exposure_11
			.others
			.iter()
			.find(|o| o.who == 101)
			.unwrap()
			.etp_balance;
		let nominated_value_21 = exposure_21
			.others
			.iter()
			.find(|o| o.who == 101)
			.unwrap()
			.etp_balance;

		on_offence_in_era(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(10)],
			2,
		);

		assert_eq!(Etp::free_balance(11), 900);

		let slash_1_amount = Perbill::from_percent(10) * nominated_value_11;
		assert_eq!(Etp::free_balance(101), 2000 - slash_1_amount);

		let expected_spans = vec![
			slashing::SlashingSpan {
				index: 1,
				start: 4,
				length: None,
			},
			slashing::SlashingSpan {
				index: 0,
				start: 0,
				length: Some(4),
			},
		];

		let get_span = |account| <Staking as Store>::SlashingSpans::get(&account).unwrap();

		assert_eq!(get_span(11).iter().collect::<Vec<_>>(), expected_spans,);

		assert_eq!(get_span(101).iter().collect::<Vec<_>>(), expected_spans,);

		// second slash: higher era, higher value, same span.
		on_offence_in_era(
			&[OffenceDetails {
				offender: (21, Staking::eras_stakers(active_era(), 21)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(30)],
			3,
		);

		// 11 was not further slashed, but 21 and 101 were.
		assert_eq!(Etp::free_balance(11), 900);
		assert_eq!(Etp::free_balance(21), 1700);

		let slash_2_amount = Perbill::from_percent(30) * nominated_value_21;
		assert!(slash_2_amount > slash_1_amount);

		// only the maximum slash in a single span is taken.
		assert_eq!(Etp::free_balance(101), 2000 - slash_2_amount);

		// third slash: in same era and on same validator as first, higher
		// in-era value, but lower slash value than slash 2.
		on_offence_in_era(
			&[OffenceDetails {
				offender: (11, Staking::eras_stakers(active_era(), 11)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(20)],
			2,
		);

		// 11 was further slashed, but 21 and 101 were not.
		assert_eq!(Etp::free_balance(11), 800);
		assert_eq!(Etp::free_balance(21), 1700);

		let slash_3_amount = Perbill::from_percent(20) * nominated_value_21;
		assert!(slash_3_amount < slash_2_amount);
		assert!(slash_3_amount > slash_1_amount);

		// only the maximum slash in a single span is taken.
		assert_eq!(Etp::free_balance(101), 2000 - slash_2_amount);
	});
}

#[test]
fn slashes_are_summed_across_spans() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);
		start_active_era(2);
		start_active_era(3);

		assert_eq!(Etp::free_balance(21), 2000);
		assert_eq!(Staking::stake_of(&21).0, 1000);

		let get_span = |account| <Staking as Store>::SlashingSpans::get(&account).unwrap();

		on_offence_now(
			&[OffenceDetails {
				offender: (21, Staking::eras_stakers(active_era(), 21)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(10)],
		);

		let expected_spans = vec![
			slashing::SlashingSpan {
				index: 1,
				start: 4,
				length: None,
			},
			slashing::SlashingSpan {
				index: 0,
				start: 0,
				length: Some(4),
			},
		];

		assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
		assert_eq!(Etp::free_balance(21), 1900);

		// 21 has been force-chilled. re-signal intent to validate.
		Staking::validate(Origin::signed(20), Default::default()).unwrap();

		start_active_era(4);

		assert_eq!(Staking::stake_of(&21).0, 900);

		on_offence_now(
			&[OffenceDetails {
				offender: (21, Staking::eras_stakers(active_era(), 21)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(10)],
		);

		let expected_spans = vec![
			slashing::SlashingSpan {
				index: 2,
				start: 5,
				length: None,
			},
			slashing::SlashingSpan {
				index: 1,
				start: 4,
				length: Some(1),
			},
			slashing::SlashingSpan {
				index: 0,
				start: 0,
				length: Some(4),
			},
		];

		assert_eq!(get_span(21).iter().collect::<Vec<_>>(), expected_spans);
		assert_eq!(Etp::free_balance(21), 1810);
	});
}

#[test]
fn deferred_slashes_are_deferred() {
	ExtBuilder::default()
		.slash_defer_duration(2)
		.build_and_execute(|| {
			start_active_era(1);

			assert_eq!(Etp::free_balance(11), 1000);

			let exposure = Staking::eras_stakers(active_era(), 11);
			assert_eq!(Etp::free_balance(101), 2000);
			let nominated_value = exposure
				.others
				.iter()
				.find(|o| o.who == 101)
				.unwrap()
				.etp_balance;

			on_offence_now(
				&[OffenceDetails {
					offender: (11, Staking::eras_stakers(active_era(), 11)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(10)],
			);

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			start_active_era(2);

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			start_active_era(3);

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			// at the start of era 4, slashes from era 1 are processed,
			// after being deferred for at least 2 full eras.
			start_active_era(4);

			assert_eq!(Etp::free_balance(11), 900);
			assert_eq!(Etp::free_balance(101), 2000 - (nominated_value / 10));
		})
}

#[test]
fn remove_deferred() {
	ExtBuilder::default()
		.slash_defer_duration(2)
		.build_and_execute(|| {
			start_active_era(1);

			assert_eq!(Etp::free_balance(11), 1000);

			let exposure = Staking::eras_stakers(active_era(), 11);
			assert_eq!(Etp::free_balance(101), 2000);
			let nominated_value = exposure
				.others
				.iter()
				.find(|o| o.who == 101)
				.unwrap()
				.etp_balance;

			on_offence_now(
				&[OffenceDetails {
					offender: (11, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(10)],
			);

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			start_active_era(2);

			on_offence_in_era(
				&[OffenceDetails {
					offender: (11, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(15)],
				1,
			);

			// fails if empty
			assert_noop!(
				Staking::cancel_deferred_slash(Origin::root(), 1, vec![]),
				StakingError::EmptyTargets
			);

			assert_ok!(Staking::cancel_deferred_slash(Origin::root(), 1, vec![0]));

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			start_active_era(3);

			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			// at the start of era 4, slashes from era 1 are processed,
			// after being deferred for at least 2 full eras.
			start_active_era(4);

			// the first slash for 10% was cancelled, so no effect.
			assert_eq!(Etp::free_balance(11), 1000);
			assert_eq!(Etp::free_balance(101), 2000);

			start_active_era(5);

			let slash_10 = Perbill::from_percent(10);
			let slash_15 = Perbill::from_percent(15);
			let initial_slash = slash_10 * nominated_value;

			let total_slash = slash_15 * nominated_value;
			let actual_slash = total_slash - initial_slash;

			// 5% slash (15 - 10) processed now.
			assert_eq!(Etp::free_balance(11), 950);
			assert_eq!(Etp::free_balance(101), 2000 - actual_slash);
		})
}

#[test]
fn remove_multi_deferred() {
	ExtBuilder::default()
		.slash_defer_duration(2)
		.build_and_execute(|| {
			start_active_era(1);

			assert_eq!(Etp::free_balance(11), 1000);

			let exposure = Staking::eras_stakers(active_era(), 11);
			assert_eq!(Etp::free_balance(101), 2000);

			on_offence_now(
				&[OffenceDetails {
					offender: (11, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(10)],
			);

			on_offence_now(
				&[OffenceDetails {
					offender: (21, Staking::eras_stakers(active_era(), 21)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(10)],
			);

			on_offence_now(
				&[OffenceDetails {
					offender: (11, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(25)],
			);

			on_offence_now(
				&[OffenceDetails {
					offender: (42, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(25)],
			);

			on_offence_now(
				&[OffenceDetails {
					offender: (69, exposure.clone()),
					reporters: vec![],
				}],
				&[Perbill::from_percent(25)],
			);

			assert_eq!(<Staking as Store>::UnappliedSlashes::get(&1).len(), 5);

			// fails if list is not sorted
			assert_noop!(
				Staking::cancel_deferred_slash(Origin::root(), 1, vec![2, 0, 4]),
				StakingError::NotSortedAndUnique,
			);
			// fails if list is not unique
			assert_noop!(
				Staking::cancel_deferred_slash(Origin::root(), 1, vec![0, 2, 2]),
				StakingError::NotSortedAndUnique,
			);
			// fails if bad index
			assert_noop!(
				Staking::cancel_deferred_slash(Origin::root(), 1, vec![1, 2, 3, 4, 5]),
				StakingError::InvalidSlashIndex,
			);

			assert_ok!(Staking::cancel_deferred_slash(
				Origin::root(),
				1,
				vec![0, 2, 4]
			));

			let slashes = <Staking as Store>::UnappliedSlashes::get(&1);
			assert_eq!(slashes.len(), 2);
			assert_eq!(slashes[0].validator, 21);
			assert_eq!(slashes[1].validator, 42);
		})
}

#[test]
fn slash_kicks_validators_not_nominators_and_disables_nominator_for_kicked_validator() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);
		assert_eq_uvec!(Session::validators(), vec![11, 21]);

		// pre-slash balance
		assert_eq!(Etp::free_balance(11), 1000);
		assert_eq!(Etp::free_balance(101), 2000);

		// 11 and 21 both have the support of 100
		let exposure_11 = Staking::eras_stakers(active_era(), &11);
		let exposure_21 = Staking::eras_stakers(active_era(), &21);

		assert_eq_error_rate!(
			exposure_11.total_power,
			Staking::currency_to_power(1000 + 125, Staking::etp_pool()),
			1
		);
		assert_eq_error_rate!(
			exposure_21.total_power,
			Staking::currency_to_power(1000 + 375, Staking::etp_pool()),
			1
		);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, exposure_11.clone()),
				reporters: vec![],
			}],
			&[Perbill::from_percent(10)],
		);

		// post-slash balance
		let nominator_slash_amount_11 = 125 / 10;
		assert_eq!(Etp::free_balance(11), 900);
		assert_eq!(Etp::free_balance(101), 2000 - nominator_slash_amount_11);

		// This is the best way to check that the validator was chilled; `get` will
		// return default value.
		for (stash, _) in <Staking as Store>::Validators::iter() {
			assert!(stash != 11);
		}

		let nominations = <Staking as Store>::Nominators::get(&101).unwrap();

		// and make sure that the vote will be ignored even if the validator
		// re-registers.
		let last_slash = <Staking as Store>::SlashingSpans::get(&11)
			.unwrap()
			.last_nonzero_slash();
		assert!(nominations.submitted_in < last_slash);

		// actually re-bond the slashed validator
		assert_ok!(Staking::validate(Origin::signed(10), Default::default()));

		start_active_era(2);
		let exposure_11 = Staking::eras_stakers(active_era(), &11);
		let exposure_21 = Staking::eras_stakers(active_era(), &21);

		// 10 is re-elected, but without the support of 100
		assert_eq!(
			exposure_11.total_power,
			Staking::currency_to_power(900, Staking::etp_pool())
		);

		// 20 is re-elected, with the (almost) entire support of 100
		assert_eq_error_rate!(
			exposure_21.total_power,
			Staking::currency_to_power(
				1000 + 500 - nominator_slash_amount_11,
				Staking::etp_pool()
			),
			10
		);
	});
}

#[test]
fn claim_reward_at_the_last_era_and_no_double_claim_and_invalid_claim() {
	// should check that:
	// * rewards get paid until history_depth for both validators and nominators
	// * an invalid era to claim doesn't update last_reward
	// * double claim of one era fails
	ExtBuilder::default().nominate(true).build_and_execute(|| {
		let init_balance_10 = Etp::free_balance(&10);
		let init_balance_100 = Etp::free_balance(&100);

		let part_for_10 = Perbill::from_rational_approximation::<u32>(1000, 1125);
		let part_for_100 = Perbill::from_rational_approximation::<u32>(125, 1125);

		// Check state
		<Payee<Test>>::insert(11, RewardDestination::Controller);
		<Payee<Test>>::insert(101, RewardDestination::Controller);

		Staking::reward_by_ids(vec![(11, 1)]);
		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_0 = current_total_payout_for_duration(reward_time_per_era());

		start_active_era(1);

		Staking::reward_by_ids(vec![(11, 1)]);
		// Change total issuance in order to modify total payout
		let _ = Etp::deposit_creating(&999, 1_000_000_000);
		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_1 = current_total_payout_for_duration(reward_time_per_era());
		assert!(total_payout_1 != total_payout_0);

		start_active_era(2);

		Staking::reward_by_ids(vec![(11, 1)]);
		// Change total issuance in order to modify total payout
		let _ = Etp::deposit_creating(&999, 1_000_000_000);
		// Compute total payout now for whole duration as other parameter won't change
		let total_payout_2 = current_total_payout_for_duration(reward_time_per_era());
		assert!(total_payout_2 != total_payout_0);
		assert!(total_payout_2 != total_payout_1);

		start_active_era(Staking::history_depth() + 1);

		let active_era = active_era();

		// This is the latest planned era in staking, not the active era
		let current_era = Staking::current_era().unwrap();

		// Last kept is 1:
		assert!(current_era - Staking::history_depth() == 1);
		assert_noop!(
			Staking::payout_stakers(Origin::signed(1337), 11, 0),
			// Fail: Era out of history
			StakingError::InvalidEraToReward,
		);
		assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));
		assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 2));
		assert_noop!(
			Staking::payout_stakers(Origin::signed(1337), 11, 2),
			// Fail: Double claim
			StakingError::AlreadyClaimed
		);
		assert_noop!(
			Staking::payout_stakers(Origin::signed(1337), 11, active_era),
			// Fail: Era not finished yet
			StakingError::InvalidEraToReward,
		);

		// Era 0 can't be rewarded anymore and current era can't be rewarded yet
		// only era 1 and 2 can be rewarded.

		assert_eq_error_rate!(
			Etp::free_balance(&10),
			init_balance_10 + part_for_10 * (total_payout_1 + total_payout_2),
			MICRO,
		);
		assert_eq_error_rate!(
			Etp::free_balance(&100),
			init_balance_100 + part_for_100 * (total_payout_1 + total_payout_2),
			MICRO,
		);
	});
}

#[test]
fn zero_slash_keeps_nominators() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(1);

		assert_eq!(Etp::free_balance(11), 1000);

		let exposure = Staking::eras_stakers(active_era(), 11);
		assert_eq!(Etp::free_balance(101), 2000);

		on_offence_now(
			&[OffenceDetails {
				offender: (11, exposure.clone()),
				reporters: vec![],
			}],
			&[Perbill::from_percent(0)],
		);

		assert_eq!(Etp::free_balance(11), 1000);
		assert_eq!(Etp::free_balance(101), 2000);

		// This is the best way to check that the validator was chilled; `get` will
		// return default value.
		for (stash, _) in <Staking as Store>::Validators::iter() {
			assert!(stash != 11);
		}

		let nominations = <Staking as Store>::Nominators::get(&101).unwrap();

		// and make sure that the vote will not be ignored, because the slash was
		// zero.
		let last_slash = <Staking as Store>::SlashingSpans::get(&11)
			.unwrap()
			.last_nonzero_slash();
		assert!(nominations.submitted_in >= last_slash);
	});
}

#[test]
fn six_session_delay() {
	ExtBuilder::default()
		.initialize_first_session(false)
		.build_and_execute(|| {
			use pallet_session::SessionManager;

			let val_set = Session::validators();
			let init_session = Session::current_index();
			let init_active_era = active_era();
			// pallet-session is delaying session by one, thus the next session to plan is +2.
			assert_eq!(
				<Staking as SessionManager<_>>::new_session(init_session + 2),
				None
			);
			assert_eq!(
				<Staking as SessionManager<_>>::new_session(init_session + 3),
				Some(val_set.clone())
			);
			assert_eq!(
				<Staking as SessionManager<_>>::new_session(init_session + 4),
				None
			);
			assert_eq!(
				<Staking as SessionManager<_>>::new_session(init_session + 5),
				None
			);
			assert_eq!(
				<Staking as SessionManager<_>>::new_session(init_session + 6),
				Some(val_set.clone())
			);

			<Staking as SessionManager<_>>::end_session(init_session);
			<Staking as SessionManager<_>>::start_session(init_session + 1);
			assert_eq!(active_era(), init_active_era);
			<Staking as SessionManager<_>>::end_session(init_session + 1);
			<Staking as SessionManager<_>>::start_session(init_session + 2);
			assert_eq!(active_era(), init_active_era);

			// Reward current era
			Staking::reward_by_ids(vec![(11, 1)]);

			// New active era is triggered here.
			<Staking as SessionManager<_>>::end_session(init_session + 2);
			<Staking as SessionManager<_>>::start_session(init_session + 3);
			assert_eq!(active_era(), init_active_era + 1);
			<Staking as SessionManager<_>>::end_session(init_session + 3);
			<Staking as SessionManager<_>>::start_session(init_session + 4);
			assert_eq!(active_era(), init_active_era + 1);
			<Staking as SessionManager<_>>::end_session(init_session + 4);
			<Staking as SessionManager<_>>::start_session(init_session + 5);
			assert_eq!(active_era(), init_active_era + 1);

			// Reward current era
			Staking::reward_by_ids(vec![(21, 2)]);

			// New active era is triggered here.
			<Staking as SessionManager<_>>::end_session(init_session + 5);
			<Staking as SessionManager<_>>::start_session(init_session + 6);
			assert_eq!(active_era(), init_active_era + 2);

			// That reward are correct
			assert_eq!(Staking::eras_reward_points(init_active_era).total, 1);
			assert_eq!(Staking::eras_reward_points(init_active_era + 1).total, 2);
		});
}

#[test]
fn test_max_nominator_rewarded_per_validator_and_cant_steal_someone_else_reward() {
	ExtBuilder::default().build_and_execute(|| {
		for i in 0..=<Test as Config>::MaxNominatorRewardedPerValidator::get() {
			let stash = 10_000 + i as AccountId;
			let controller = 20_000 + i as AccountId;
			let balance = 10_000 + i as Balance;
			Etp::make_free_balance_be(&stash, balance);
			assert_ok!(Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::EtpBalance(balance),
				RewardDestination::Stash,
				0,
			));
			assert_ok!(Staking::nominate(Origin::signed(controller), vec![11]));
		}
		start_active_era(1);

		Staking::reward_by_ids(vec![(11, 1)]);
		// compute and ensure the reward amount is greater than zero.
		let _ = current_total_payout_for_duration(reward_time_per_era());

		start_active_era(2);
		make_all_reward_payment(1);

		// Assert only nominators from 1 to Max are rewarded
		for i in 0..=<Test as Config>::MaxNominatorRewardedPerValidator::get() {
			let stash = 10_000 + i as AccountId;
			let balance = 10_000 + i as Balance;
			if stash == 10_000 {
				assert!(Etp::free_balance(&stash) == balance);
			} else {
				assert!(Etp::free_balance(&stash) > balance);
			}
		}
	});
}

#[test]
fn set_history_depth_works() {
	ExtBuilder::default().build_and_execute(|| {
		start_active_era(10);
		Staking::set_history_depth(Origin::root(), 20, 0).unwrap();
		assert!(<Staking as Store>::ErasTotalStake::contains_key(10 - 4));
		assert!(<Staking as Store>::ErasTotalStake::contains_key(10 - 5));
		Staking::set_history_depth(Origin::root(), 4, 0).unwrap();
		assert!(<Staking as Store>::ErasTotalStake::contains_key(10 - 4));
		assert!(!<Staking as Store>::ErasTotalStake::contains_key(10 - 5));
		Staking::set_history_depth(Origin::root(), 3, 0).unwrap();
		assert!(!<Staking as Store>::ErasTotalStake::contains_key(10 - 4));
		assert!(!<Staking as Store>::ErasTotalStake::contains_key(10 - 5));
		Staking::set_history_depth(Origin::root(), 8, 0).unwrap();
		assert!(!<Staking as Store>::ErasTotalStake::contains_key(10 - 4));
		assert!(!<Staking as Store>::ErasTotalStake::contains_key(10 - 5));
	});
}

#[test]
fn test_payout_stakers() {
	// Here we will test validator can set `max_nominators_payout` and it works.
	// We also test that `payout_extra_nominators` works.
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			let balance = 1000;
			// Create a validators:
			bond_validator(11, 10, StakingBalance::EtpBalance(balance)); // Default(64)

			// Create nominators, targeting stash of validators
			for i in 0..100 {
				bond_nominator(
					1000 + i,
					100 + i,
					StakingBalance::EtpBalance(balance + i as Balance),
					vec![11],
				);
			}

			start_active_era(1);
			Staking::reward_by_ids(vec![(11, 1)]);

			// compute and ensure the reward amount is greater than zero.
			let _ = current_total_payout_for_duration(reward_time_per_era());

			start_active_era(2);
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));

			// Top 64 nominators of validator 11 automatically paid out, including the validator
			// Validator payout goes to controller.
			assert!(Etp::free_balance(&10) > balance);
			for i in 36..100 {
				assert!(Etp::free_balance(&(100 + i)) > balance + i as Balance);
			}
			// The bottom 36 do not
			for i in 0..36 {
				assert_eq!(Etp::free_balance(&(100 + i)), balance + i as Balance);
			}

			// We track rewards in `claimed_rewards` vec
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: vec![1],
					..Default::default()
				})
			);

			for i in 3..16 {
				Staking::reward_by_ids(vec![(11, 1)]);

				// compute and ensure the reward amount is greater than zero.
				let _ = current_total_payout_for_duration(reward_time_per_era());

				start_active_era(i);
				assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, i - 1));
			}

			// We track rewards in `claimed_rewards` vec
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: (1..=14).collect(),
					..Default::default()
				})
			);

			for i in 16..100 {
				Staking::reward_by_ids(vec![(11, 1)]);
				// compute and ensure the reward amount is greater than zero.
				let _ = current_total_payout_for_duration(reward_time_per_era());
				start_active_era(i);
			}

			// We clean it up as history passes
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 15));
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 98));
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: vec![15, 98],
					..Default::default()
				})
			);

			// Out of order claims works.
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 69));
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 23));
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 42));
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: vec![15, 23, 42, 69, 98],
					..Default::default()
				})
			);
		});
}

#[test]
fn payout_stakers_handles_basic_errors() {
	// Here we will test payouts handle all errors.
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			// Same setup as the test above
			let balance = 1000;
			bond_validator(11, 10, StakingBalance::EtpBalance(balance)); // Default(64)

			// Create nominators, targeting stash
			for i in 0..100 {
				bond_nominator(
					1000 + i,
					100 + i,
					StakingBalance::EtpBalance(balance + i as Balance),
					vec![11],
				);
			}

			start_active_era(1);
			Staking::reward_by_ids(vec![(11, 1)]);

			// compute and ensure the reward amount is greater than zero.
			let _ = current_total_payout_for_duration(reward_time_per_era());

			start_active_era(2);

			// Wrong Era, too big
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 11, 2),
				StakingError::InvalidEraToReward
			);
			// Wrong Staker
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 10, 1),
				StakingError::NotStash
			);

			for i in 3..100 {
				Staking::reward_by_ids(vec![(11, 1)]);
				// compute and ensure the reward amount is greater than zero.
				let _ = current_total_payout_for_duration(reward_time_per_era());
				start_active_era(i);
			}
			// We are at era 99, with history depth of 84
			// We should be able to payout era 15 through 98 (84 total eras), but not 14 or 99.
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 11, 14),
				StakingError::InvalidEraToReward
			);
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 11, 99),
				StakingError::InvalidEraToReward
			);
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 15));
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 98));

			// Can't claim again
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 11, 15),
				StakingError::AlreadyClaimed
			);
			assert_noop!(
				Staking::payout_stakers(Origin::signed(1337), 11, 98),
				StakingError::AlreadyClaimed
			);
		});
}

#[test]
fn bond_duetp_era_correctly_populates_claimed_rewards() {
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			// Era = None
			bond_validator(9, 8, StakingBalance::EtpBalance(1000));
			assert_eq!(
				Staking::ledger(&8),
				Some(StakingLedger {
					stash: 9,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				})
			);
			start_active_era(5);
			bond_validator(11, 10, StakingBalance::EtpBalance(1000));
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: (0..5).collect(),
					..Default::default()
				})
			);
			start_active_era(99);
			bond_validator(13, 12, StakingBalance::EtpBalance(1000));
			assert_eq!(
				Staking::ledger(&12),
				Some(StakingLedger {
					stash: 13,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					claimed_rewards: (15..99).collect(),
					..Default::default()
				})
			);
		});
}

#[test]
fn offences_weight_calculated_correctly() {
	ExtBuilder::default().nominate(true).build_and_execute(|| {
		// On offence with zero offenders: 4 Reads, 1 Write
		let zero_offence_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 1);
		assert_eq!(
			Staking::on_offence(&[], &[Perbill::from_percent(50)], 0),
			Ok(zero_offence_weight)
		);

		// On Offence with N offenders, Unapplied: 4 Reads, 1 Write + 4 Reads, 5 Writes
		let n_offence_unapplied_weight = <Test as frame_system::Config>::DbWeight::get()
			.reads_writes(4, 1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 5);

		let offenders: Vec<
			OffenceDetails<
				<Test as frame_system::Config>::AccountId,
				pallet_session::historical::IdentificationTuple<Test>,
			>,
		> = (1..10)
			.map(|i| OffenceDetails {
				offender: (
					i,
					Staking::eras_stakers(active_era(), i),
				),
				reporters: vec![],
			})
			.collect();
		assert_eq!(
			Staking::on_offence(&offenders, &[Perbill::from_percent(50)], 0),
			Ok(n_offence_unapplied_weight)
		);

		// On Offence with one offenders, Applied
		let one_offender = [OffenceDetails {
			offender: (
				11,
				Staking::eras_stakers(active_era(), 11),
			),
			reporters: vec![1],
		}];

		let n = 1; // Number of offenders
		let rw = 3 + 3 * n; // rw reads and writes
		let one_offence_unapplied_weight = <Test as frame_system::Config>::DbWeight::get().reads_writes(4, 1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(rw, rw)
			// One `slash_cost`
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(6, 5)
			// `slash_cost` * nominators (1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(6, 5)
			// `reward_cost` * reporters (1)
			+ <Test as frame_system::Config>::DbWeight::get().reads_writes(2, 2);

		assert_eq!(
			Staking::on_offence(&one_offender, &[Perbill::from_percent(50)], 0),
			Ok(one_offence_unapplied_weight)
		);
	});
}

#[test]
fn on_initialize_weight_is_correct() {
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			assert_eq!(<Validators<Test>>::iter().count(), 0);
			assert_eq!(<Nominators<Test>>::iter().count(), 0);
			// When this pallet has nothing, we do 4 reads each block
			let base_weight = <Test as frame_system::Config>::DbWeight::get().reads(4);
			assert_eq!(base_weight, Staking::on_initialize(0));
		});

	ExtBuilder::default()
		.offchain_election_ext()
		.validator_count(4)
		.has_stakers(false)
		.build()
		.execute_with(|| {
			crate::substrate_tests::offchain_election::build_offchain_election_test_ext();
			run_to_block(11);
			Staking::on_finalize(System::block_number());
			System::set_block_number((System::block_number() + 1).into());
			Timestamp::set_timestamp(System::block_number() * 1000 + INIT_TIMESTAMP);
			Session::on_initialize(System::block_number());

			assert_eq!(<Validators<Test>>::iter().count(), 4);
			assert_eq!(<Nominators<Test>>::iter().count(), 5);
			// With 4 validators and 5 nominator, we should increase weight by:
			// - (4 + 5) reads
			// - 3 Writes
			let final_weight =
				<Test as frame_system::Config>::DbWeight::get().reads_writes(4 + 9, 3);
			assert_eq!(final_weight, Staking::on_initialize(System::block_number()));
		});
}

#[test]
fn payout_creates_controller() {
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			let balance = StakingBalance::EtpBalance(1000);
			// Create a validator:
			bond_validator(11, 10, balance);

			// Create a stash/controller pair
			bond_nominator(1234, 1337, StakingBalance::EtpBalance(100), vec![11]);

			// kill controller
			assert_ok!(Etp::transfer(Origin::signed(1337), 1234, 100));
			assert_eq!(Etp::free_balance(1337), 0);

			start_active_era(1);
			Staking::reward_by_ids(vec![(11, 1)]);
			// compute and ensure the reward amount is greater than zero.
			let _ = current_total_payout_for_duration(reward_time_per_era());
			start_active_era(2);
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));

			// Controller is created
			assert!(Etp::free_balance(1337) > 0);
		})
}

#[test]
fn payout_to_any_account_works() {
	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			let balance = StakingBalance::EtpBalance(1000);
			// Create a validator:
			bond_validator(11, 10, balance); // Default(64)

			// Create a stash/controller pair
			bond_nominator(1234, 1337, StakingBalance::EtpBalance(100), vec![11]);

			// Update payout location
			assert_ok!(Staking::set_payee(
				Origin::signed(1337),
				RewardDestination::Account(42)
			));

			// Reward Destination account doesn't exist
			assert_eq!(Etp::free_balance(42), 0);

			start_active_era(1);
			Staking::reward_by_ids(vec![(11, 1)]);
			// compute and ensure the reward amount is greater than zero.
			let _ = current_total_payout_for_duration(reward_time_per_era());
			start_active_era(2);
			assert_ok!(Staking::payout_stakers(Origin::signed(1337), 11, 1));

			// Payment is successful
			assert!(Etp::free_balance(42) > 0);
		})
}

#[test]
fn session_buffeetp_with_offset() {
	// similar to live-chains, have some offset for the first session
	ExtBuilder::default()
		.offset(2)
		.period(5)
		.sessions_per_era(5)
		.build_and_execute(|| {
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 0);

			start_session(1);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 1);
			assert_eq!(System::block_number(), 2);

			start_session(2);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 2);
			assert_eq!(System::block_number(), 7);

			start_session(3);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 3);
			assert_eq!(System::block_number(), 12);

			// active era is lagging behind by one session, because of how session module works.
			start_session(4);
			assert_eq!(current_era(), 1);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 4);
			assert_eq!(System::block_number(), 17);

			start_session(5);
			assert_eq!(current_era(), 1);
			assert_eq!(active_era(), 1);
			assert_eq!(Session::current_index(), 5);
			assert_eq!(System::block_number(), 22);

			// go all the way to active 2.
			start_active_era(2);
			assert_eq!(current_era(), 2);
			assert_eq!(active_era(), 2);
			assert_eq!(Session::current_index(), 10);
		});
}

#[test]
fn session_buffeetp_no_offset() {
	// no offset, first session starts immediately
	ExtBuilder::default()
		.offset(0)
		.period(5)
		.sessions_per_era(5)
		.build_and_execute(|| {
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 0);

			start_session(1);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 1);
			assert_eq!(System::block_number(), 5);

			start_session(2);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 2);
			assert_eq!(System::block_number(), 10);

			start_session(3);
			assert_eq!(current_era(), 0);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 3);
			assert_eq!(System::block_number(), 15);

			// active era is lagging behind by one session, because of how session module works.
			start_session(4);
			assert_eq!(current_era(), 1);
			assert_eq!(active_era(), 0);
			assert_eq!(Session::current_index(), 4);
			assert_eq!(System::block_number(), 20);

			start_session(5);
			assert_eq!(current_era(), 1);
			assert_eq!(active_era(), 1);
			assert_eq!(Session::current_index(), 5);
			assert_eq!(System::block_number(), 25);

			// go all the way to active 2.
			start_active_era(2);
			assert_eq!(current_era(), 2);
			assert_eq!(active_era(), 2);
			assert_eq!(Session::current_index(), 10);
		});
}

#[test]
fn cannot_rebond_to_lower_than_ed() {
	ExtBuilder::default()
		.existential_deposit(10)
		.build_and_execute(|| {
			// stash must have more balance than bonded for this to work.
			assert_eq!(Etp::free_balance(&21), 512_000);

			// initial stuff.
			assert_eq!(
				Staking::ledger(&20).unwrap(),
				StakingLedger {
					stash: 21,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				}
			);

			// unbond all of it.
			assert_ok!(Staking::unbond(
				Origin::signed(20),
				StakingBalance::EtpBalance(1000)
			));
			assert_eq!(
				Staking::ledger(&20).unwrap(),
				StakingLedger {
					stash: 21,
					active_etp: 0,
					etp_staking_lock: StakingLock {
						staking_amount: 0,
						unbondings: vec![Unbonding {
							amount: 1000,
							until: System::block_number() + bonding_duration_in_blocks()
						}]
					},
					..Default::default()
				}
			);

			// now bond a wee bit more
			assert_noop!(
				Staking::rebond(Origin::signed(20), 5, 0),
				StakingError::InsufficientValue,
			);
		})
}

#[test]
fn cannot_bond_to_lower_than_ed() {
	ExtBuilder::default()
		.existential_deposit(10)
		.build_and_execute(|| {
			// stash must have more balance than bonded for this to work.
			assert_eq!(Etp::free_balance(&21), 512_000);

			// initial stuff.
			assert_eq!(
				Staking::ledger(&20).unwrap(),
				StakingLedger {
					stash: 21,
					active_etp: 1000,
					etp_staking_lock: StakingLock {
						staking_amount: 1000,
						unbondings: vec![]
					},
					..Default::default()
				}
			);

			// unbond all of it.
			assert_ok!(Staking::unbond(
				Origin::signed(20),
				StakingBalance::EtpBalance(1000)
			));
			assert_eq!(
				Staking::ledger(&20).unwrap(),
				StakingLedger {
					stash: 21,
					active_etp: 0,
					etp_staking_lock: StakingLock {
						staking_amount: 0,
						unbondings: vec![Unbonding {
							amount: 1000,
							until: System::block_number() + bonding_duration_in_blocks()
						}]
					},
					..Default::default()
				}
			);

			// now bond a wee bit more
			assert_noop!(
				Staking::rebond(Origin::signed(20), 5, 0),
				StakingError::InsufficientValue,
			);
		})
}
