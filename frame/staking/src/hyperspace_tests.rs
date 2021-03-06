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

// --- substrate ---
use frame_support::{assert_err, assert_ok};
use substrate_test_utils::assert_eq_uvec;
// --- hyperspace ---
use crate::{mock::*, *};

/// gen_paired_account!(a(1), b(2), m(12));
/// will create stash `a` and controller `b`
/// `a` has 100 Etp and 100 Dna
/// promise for `m` month with 50 Etp and 50 Dna
///
/// `m` can be ignore, this won't create variable `m`
/// ```rust
/// gen_paired_account!(a(1), b(2), 12);
/// ```
///
/// `m(12)` can be ignore, and it won't perform `bond` action
/// ```rust
/// gen_paired_account!(a(1), b(2));
/// ```
macro_rules! gen_paired_account {
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $promise_month:ident($how_long:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Etp::deposit_creating(&$stash, 100 * COIN);
		let _ = Dna::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Etp::deposit_creating(&$controller, COIN);
		#[allow(non_snake_case, unused)]
		let $promise_month = $how_long;
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::EtpBalance(50 * COIN),
			RewardDestination::Stash,
			$how_long,
		));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::DnaBalance(50 * COIN),
			$how_long
		));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr), $how_long:expr) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Etp::deposit_creating(&$stash, 100 * COIN);
		let _ = Dna::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Etp::deposit_creating(&$controller, COIN);
		assert_ok!(Staking::bond(
			Origin::signed($stash),
			$controller,
			StakingBalance::EtpBalance(50 * COIN),
			RewardDestination::Stash,
			$how_long,
		));
		assert_ok!(Staking::bond_extra(
			Origin::signed($stash),
			StakingBalance::DnaBalance(50 * COIN),
			$how_long,
		));
	};
	($stash:ident($stash_id:expr), $controller:ident($controller_id:expr)) => {
		#[allow(non_snake_case, unused)]
		let $stash = $stash_id;
		let _ = Etp::deposit_creating(&$stash, 100 * COIN);
		let _ = Dna::deposit_creating(&$stash, 100 * COIN);
		#[allow(non_snake_case, unused)]
		let $controller = $controller_id;
		let _ = Etp::deposit_creating(&$controller, COIN);
	};
}

#[test]
fn slash_ledger_should_work() {
	ExtBuilder::default()
		.nominate(false)
		.validator_count(1)
		.build()
		.execute_with(|| {
			start_active_era(0);

			assert_eq_uvec!(validator_controllers(), vec![20]);

			let (account_id, bond) = (777, COIN);
			let _ = Etp::deposit_creating(&account_id, bond);

			assert_ok!(Staking::bond(
				Origin::signed(account_id),
				account_id,
				StakingBalance::EtpBalance(bond),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::deposit_extra(
				Origin::signed(account_id),
				COIN * 80 / 100,
				36
			));
			assert_ok!(Staking::validate(
				Origin::signed(account_id),
				ValidatorPrefs::default()
			));

			start_active_era(1);

			assert_eq_uvec!(validator_controllers(), vec![777]);

			on_offence_now(
				&[OffenceDetails {
					offender: (account_id, Staking::eras_stakers(active_era(), account_id)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(90)],
			);

			{
				let total = bond;
				let normal = total * (100 - 80) / 100;
				let deposit = total * 80 / 100;

				assert!(normal + deposit == total);
				let total_slashed = bond * 90 / 100;

				assert!(total_slashed > normal);
				let normal_slashed = normal;
				let deposit_slashed = total_slashed - normal_slashed;

				assert_eq!(
					Staking::ledger(&account_id).unwrap(),
					StakingLedger {
						stash: account_id,
						active_etp: total - total_slashed,
						active_deposit_etp: deposit - deposit_slashed,
						deposit_items: vec![TimeDepositItem {
							value: deposit - deposit_slashed,
							start_time: 30000,
							expire_time: 93312030000,
						}],
						etp_staking_lock: StakingLock {
							staking_amount: deposit - deposit_slashed,
							unbondings: vec![],
						},
						..Default::default()
					},
				);
			}

			let ledger = Staking::ledger(&account_id).unwrap();

			// Should not overflow here
			assert_ok!(Staking::unbond(
				Origin::signed(account_id),
				StakingBalance::EtpBalance(1)
			));

			assert_eq!(ledger, Staking::ledger(&account_id).unwrap());
		});
}

#[test]
fn dna_should_reward_even_does_not_own_dna_before() {
	// Tests that validator storage items are cleaned up when stash is empty
	// Tests that storage items are untouched when controller is empty
	ExtBuilder::default()
		.has_stakers(false)
		.build()
		.execute_with(|| {
			let account_id = 777;
			let _ = Etp::deposit_creating(&account_id, 10000);

			assert!(Dna::free_balance(&account_id).is_zero());
			assert_ok!(Staking::bond(
				Origin::signed(account_id),
				account_id,
				StakingBalance::EtpBalance(10000),
				RewardDestination::Stash,
				36,
			));
			assert_eq!(Dna::free_balance(&account_id), 3);
		});
}

#[test]
fn bond_zero_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(s(123), c(456));
		assert_err!(
			Staking::bond(
				Origin::signed(s),
				c,
				StakingBalance::EtpBalance(0),
				RewardDestination::Stash,
				0,
			),
			StakingError::InsufficientValue
		);

		gen_paired_account!(s(234), c(567));
		assert_err!(
			Staking::bond(
				Origin::signed(s),
				c,
				StakingBalance::DnaBalance(0),
				RewardDestination::Stash,
				0,
			),
			StakingError::InsufficientValue
		);
	});
}

#[test]
fn normal_dna_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		{
			let (stash, controller) = (1001, 1000);

			let _ = Dna::deposit_creating(&stash, 10 * COIN);
			assert_ok!(Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::DnaBalance(10 * COIN),
				RewardDestination::Stash,
				0,
			));
			assert_eq!(
				Staking::ledger(controller).unwrap(),
				StakingLedger {
					stash,
					active_dna: 10 * COIN,
					dna_staking_lock: StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					},
					..Default::default()
				}
			);
			assert_eq!(
				Dna::locks(&stash),
				vec![BalanceLock {
					id: STAKING_ID,
					lock_for: LockFor::Staking(StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					}),
					lock_reasons: LockReasons::All
				}]
			);
		}

		{
			let (stash, controller) = (2001, 2000);

			// promise_month should not work for dna
			let _ = Dna::deposit_creating(&stash, 10 * COIN);
			assert_ok!(Staking::bond(
				Origin::signed(stash),
				controller,
				StakingBalance::DnaBalance(10 * COIN),
				RewardDestination::Stash,
				12,
			));
			assert_eq!(
				Staking::ledger(controller).unwrap(),
				StakingLedger {
					stash,
					active_dna: 10 * COIN,
					dna_staking_lock: StakingLock {
						staking_amount: 10 * COIN,
						unbondings: vec![],
					},
					..Default::default()
				}
			);
		}
	});
}

#[test]
fn time_deposit_etp_unbond_and_withdraw_automatically_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (11, 10);

		let start = System::block_number();
		let unbond_value = 10;

		// unbond 10 for the first time
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(unbond_value),
		));

		// check the lock
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1000 - unbond_value,
					unbondings: vec![Unbonding {
						amount: unbond_value,
						until: BondingDurationInBlockNumber::get() + start,
					}],
				}),
				lock_reasons: LockReasons::All,
			}],
		);

		// check the ledger
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash,
				active_etp: 1000 - unbond_value,
				active_deposit_etp: 0,
				active_dna: 0,
				deposit_items: vec![],
				etp_staking_lock: StakingLock {
					staking_amount: 1000 - unbond_value,
					unbondings: vec![Unbonding {
						amount: unbond_value,
						until: BondingDurationInBlockNumber::get() + start,
					}],
				},
				dna_staking_lock: Default::default(),
				claimed_rewards: vec![]
			},
		);

		let unbond_start = BondingDurationInBlockNumber::get() + start - 1;
		System::set_block_number(unbond_start);

		// unbond for the second time
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(90)
		));

		// check the locks
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 900,
					unbondings: vec![
						Unbonding {
							amount: unbond_value,
							until: BondingDurationInBlockNumber::get() + start,
						},
						Unbonding {
							amount: 90,
							until: BondingDurationInBlockNumber::get() + unbond_start,
						},
					],
				}),
				lock_reasons: LockReasons::All,
			}],
		);

		// We can't transfer current now.
		assert_err!(
			Etp::transfer(Origin::signed(stash), controller, 1),
			EtpError::LiquidityRestrictions
		);

		let unbond_start_2 = BondingDurationInBlockNumber::get() + unbond_start + 1;
		System::set_block_number(unbond_start_2);

		// stash account can transfer again!
		assert_ok!(Etp::transfer(Origin::signed(stash), controller, 1));

		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 900,
					unbondings: vec![
						Unbonding {
							amount: unbond_value,
							until: BondingDurationInBlockNumber::get() + start,
						},
						Unbonding {
							amount: 90,
							until: BondingDurationInBlockNumber::get() + unbond_start,
						},
					],
				}),
				lock_reasons: LockReasons::All,
			}],
		);

		// Unbond all
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(COIN)
		));

		assert_eq!(Etp::locks(&stash).len(), 1);

		System::set_block_number(BondingDurationInBlockNumber::get() + unbond_start_2 + 1);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(10)
		));

		// TODO: clean dust ledger
		// check the ledger, it will be empty because we have
		// just unbonded all balances, the ledger is drained.
		// assert!(Staking::ledger(controller).is_none());

		// check the ledger
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash,
				..Default::default()
			},
		);
	});
}

#[test]
fn normal_unbond_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (11, 10);
		let value = 200 * COIN;
		let promise_month: u64 = 12;
		let _ = Etp::deposit_creating(&stash, 1000 * COIN);
		let start = System::block_number();

		{
			let mut ledger = Staking::ledger(controller).unwrap();

			assert_ok!(Staking::bond_extra(
				Origin::signed(stash),
				StakingBalance::EtpBalance(value),
				promise_month as u8,
			));
			ledger.active_etp += value;
			ledger.active_deposit_etp += value;
			ledger.deposit_items.push(TimeDepositItem {
				value,
				start_time: INIT_TIMESTAMP,
				expire_time: INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS,
			});
			ledger.etp_staking_lock.staking_amount += value;
			assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		}

		{
			let dna_free_balance = Dna::free_balance(&stash);
			let mut ledger = Staking::ledger(controller).unwrap();

			assert_ok!(Staking::bond_extra(
				Origin::signed(stash),
				StakingBalance::DnaBalance(COIN),
				0,
			));
			ledger.active_dna += dna_free_balance;
			ledger.dna_staking_lock.staking_amount += dna_free_balance;
			assert_eq!(Staking::ledger(controller).unwrap(), ledger);

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::DnaBalance(dna_free_balance)
			));
			ledger.active_dna = 0;
			ledger.dna_staking_lock.staking_amount = 0;
			ledger.dna_staking_lock.unbondings.push(Unbonding {
				amount: dna_free_balance,
				until: BondingDurationInBlockNumber::get() + start,
			});

			assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		}
	});
}

#[test]
fn punished_claim_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let promise_month = 36;
		let bond_value = 10;
		let _ = Etp::deposit_creating(&stash, 1000);
		let mut ledger = StakingLedger {
			stash,
			active_etp: bond_value,
			active_deposit_etp: bond_value,
			deposit_items: vec![TimeDepositItem {
				value: bond_value,
				start_time: INIT_TIMESTAMP,
				expire_time: INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS,
			}],
			etp_staking_lock: StakingLock {
				staking_amount: bond_value,
				unbondings: vec![],
			},
			..Default::default()
		};

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(bond_value),
			RewardDestination::Stash,
			promise_month as u8,
		));
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		// Dna is 0, skip `unbond_with_punish`.
		assert_ok!(Staking::try_claim_deposits_with_punish(
			Origin::signed(controller),
			INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS,
		));
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		// Set more dna balance to make it work.
		let _ = Dna::deposit_creating(&stash, COIN);
		assert_ok!(Staking::try_claim_deposits_with_punish(
			Origin::signed(controller),
			INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS,
		));
		ledger.active_deposit_etp -= bond_value;
		ledger.deposit_items.clear();
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		assert_eq!(Dna::free_balance(&stash), COIN - 3);
	});
}

#[test]
fn deposit_zero_should_do_nothing() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let _ = Etp::deposit_creating(&stash, COIN);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(COIN),
			RewardDestination::Stash,
			0,
		));

		for m in 0..=36 {
			// NO-OP
			assert_ok!(Staking::deposit_extra(Origin::signed(stash), 0, m));
		}

		assert!(Staking::ledger(&controller)
			.unwrap()
			.deposit_items
			.is_empty());

		// Deposit succeeded.
		assert_ok!(Staking::deposit_extra(Origin::signed(stash), COIN, 1));
		assert_eq!(Staking::ledger(&controller).unwrap().deposit_items.len(), 1);

		// NO-OP
		assert_ok!(Staking::deposit_extra(Origin::signed(stash), COIN, 1));
		assert_eq!(Staking::ledger(&controller).unwrap().deposit_items.len(), 1);
	})
}

#[test]
fn transform_to_deposited_etp_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let _ = Etp::deposit_creating(&stash, COIN);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(COIN),
			RewardDestination::Stash,
			0,
		));
		let dna_free_balance = Dna::free_balance(&stash);
		let mut ledger = Staking::ledger(controller).unwrap();

		assert_ok!(Staking::deposit_extra(Origin::signed(stash), COIN, 12));
		ledger.active_deposit_etp += COIN;
		ledger.deposit_items.push(TimeDepositItem {
			value: COIN,
			start_time: INIT_TIMESTAMP,
			expire_time: INIT_TIMESTAMP + 12 * MONTH_IN_MILLISECONDS,
		});
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
		assert_eq!(
			Dna::free_balance(&stash),
			dna_free_balance + (COIN / 10000)
		);
	});
}

#[test]
fn expired_etp_should_capable_to_promise_again() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (1001, 1000);
		let _ = Etp::deposit_creating(&stash, 10);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(10),
			RewardDestination::Stash,
			12,
		));
		let mut ledger = Staking::ledger(controller).unwrap();
		let ts = 13 * MONTH_IN_MILLISECONDS;
		let promise_extra_value = 5;

		Timestamp::set_timestamp(ts);

		assert_ok!(Staking::deposit_extra(
			Origin::signed(stash),
			promise_extra_value,
			13,
		));
		ledger.active_deposit_etp = promise_extra_value;

		// old deposit_item with 12 months promised removed
		ledger.deposit_items = vec![TimeDepositItem {
			value: promise_extra_value,
			start_time: ts,
			expire_time: 2 * ts,
		}];
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
	});
}

#[test]
fn inflation_should_be_correct() {
	ExtBuilder::default().build().execute_with(|| {
		let initial_issuance = 1_200_000_000 * COIN;
		let surplus_needed = initial_issuance - Etp::total_issuance();
		let _ = Etp::deposit_into_existing(&11, surplus_needed);

		assert_eq!(Etp::total_issuance(), initial_issuance);
	});

	// breakpoint test
	// ExtBuilder::default().build().execute_with(|| {
	// 	gen_paired_account!(validator_1_stash(123), validator_1_controller(456), 0);
	// 	gen_paired_account!(validator_2_stash(234), validator_2_controller(567), 0);
	// 	gen_paired_account!(nominator_stash(345), nominator_controller(678), 0);
	//
	// 	assert_ok!(Staking::validate(
	// 		Origin::signed(validator_1_controller),
	// 		ValidatorPrefs::default(),
	// 	));
	// 	assert_ok!(Staking::validate(
	// 		Origin::signed(validator_2_controller),
	// 		ValidatorPrefs::default(),
	// 	));
	// 	assert_ok!(Staking::nominate(
	// 		Origin::signed(nominator_controller),
	// 		vec![validator_1_stash, validator_2_stash],
	// 	));
	//
	// 	Timestamp::set_timestamp(1_575_448_345_000 - 12_000);
	// 	// breakpoint here
	// 	Staking::new_era(1);
	//
	// 	Timestamp::set_timestamp(1_575_448_345_000);
	// 	// breakpoint here
	// 	Staking::new_era(2);
	//
	// 	// breakpoint here
	//     inflation::compute_total_payout::<Test>(11_999, 1_295_225_000, 9_987_999_900_000_000_000);
	//
	// 	loop {}
	// });
}

#[test]
fn slash_also_slash_unbondings() {
	ExtBuilder::default()
		.validator_count(1)
		.build()
		.execute_with(|| {
			start_active_era(0);

			let (account_id, bond) = (777, COIN);
			let _ = Etp::deposit_creating(&account_id, bond);

			assert_ok!(Staking::bond(
				Origin::signed(account_id),
				account_id,
				StakingBalance::EtpBalance(bond),
				RewardDestination::Controller,
				0,
			));
			assert_ok!(Staking::validate(
				Origin::signed(account_id),
				ValidatorPrefs::default()
			));

			let mut etp_staking_lock = Staking::ledger(account_id)
				.unwrap()
				.etp_staking_lock
				.clone();

			start_active_era(1);

			assert_ok!(Staking::unbond(
				Origin::signed(account_id),
				StakingBalance::EtpBalance(COIN / 2)
			));

			assert_eq_uvec!(validator_controllers(), vec![777]);

			on_offence_now(
				&[OffenceDetails {
					offender: (account_id, Staking::eras_stakers(active_era(), account_id)),
					reporters: vec![],
				}],
				&[Perbill::from_percent(100)],
			);

			etp_staking_lock.staking_amount = 0;
			etp_staking_lock.unbondings.clear();

			assert_eq!(
				Staking::ledger(account_id).unwrap().etp_staking_lock,
				etp_staking_lock
			);
		});
}

#[test]
fn check_stash_already_bonded_and_controller_already_paired() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(unpaired_stash(123), unpaired_controller(456));

		assert_err!(
			Staking::bond(
				Origin::signed(11),
				unpaired_controller,
				StakingBalance::EtpBalance(COIN),
				RewardDestination::Stash,
				0,
			),
			StakingError::AlreadyBonded
		);
		assert_err!(
			Staking::bond(
				Origin::signed(unpaired_stash),
				10,
				StakingBalance::EtpBalance(COIN),
				RewardDestination::Stash,
				0,
			),
			StakingError::AlreadyPaired
		);
	});
}

#[test]
fn pool_should_be_increased_and_decreased_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		start_active_era(0);

		let mut etp_pool = Staking::etp_pool();
		let mut dna_pool = Staking::dna_pool();

		// bond: 100COIN
		gen_paired_account!(stash_1(111), controller_1(222), 0);
		gen_paired_account!(stash_2(333), controller_2(444), promise_month(12));
		etp_pool += 100 * COIN;
		dna_pool += 100 * COIN;
		assert_eq!(Staking::etp_pool(), etp_pool);
		assert_eq!(Staking::dna_pool(), dna_pool);

		// unbond: 50Etp 25Dna
		assert_ok!(Staking::unbond(
			Origin::signed(controller_1),
			StakingBalance::EtpBalance(50 * COIN)
		));
		assert_ok!(Staking::unbond(
			Origin::signed(controller_1),
			StakingBalance::DnaBalance(25 * COIN)
		));
		// not yet expired: promise for 12 months
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::EtpBalance(50 * COIN)
		));
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::DnaBalance(25 * COIN)
		));
		etp_pool -= 50 * COIN;
		dna_pool -= 50 * COIN;
		assert_eq!(Staking::etp_pool(), etp_pool);
		assert_eq!(Staking::dna_pool(), dna_pool);

		// claim: 50Etp
		assert_ok!(Staking::try_claim_deposits_with_punish(
			Origin::signed(controller_2),
			promise_month * MONTH_IN_MILLISECONDS,
		));
		// unbond deposit items: 12.5Etp
		let backup_ts = Timestamp::now();
		Timestamp::set_timestamp(INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS);
		assert_ok!(Staking::unbond(
			Origin::signed(controller_2),
			StakingBalance::EtpBalance(125 * COIN / 10),
		));
		etp_pool -= 125 * COIN / 10;
		assert_eq!(Staking::etp_pool(), etp_pool);

		Timestamp::set_timestamp(backup_ts);
		assert_ok!(Staking::validate(
			Origin::signed(controller_1),
			ValidatorPrefs::default()
		));
		assert_ok!(Staking::validate(
			Origin::signed(controller_2),
			ValidatorPrefs::default()
		));

		start_active_era(1);

		assert_eq_uvec!(validator_controllers(), vec![controller_1, controller_2]);

		// slash: 37.5Etp 50Dna
		on_offence_now(
			&[OffenceDetails {
				offender: (stash_1, Staking::eras_stakers(active_era(), stash_1)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(100)],
		);
		on_offence_now(
			&[OffenceDetails {
				offender: (stash_2, Staking::eras_stakers(active_era(), stash_2)),
				reporters: vec![],
			}],
			&[Perbill::from_percent(100)],
		);

		etp_pool -= 375 * COIN / 10;
		dna_pool -= 50 * COIN;
		assert_eq!(Staking::etp_pool(), etp_pool);
		assert_eq!(Staking::dna_pool(), dna_pool);
	});

	ExtBuilder::default()
		.has_stakers(false)
		.build_and_execute(|| {
			bond_validator(11, 10, StakingBalance::EtpBalance(1000));
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Staked
			));

			start_active_era(1);

			Staking::reward_by_ids(vec![(11, 1)]);
			let payout = current_total_payout_for_duration(reward_time_per_era());
			assert!(payout > 100);

			start_active_era(2);

			let etp_pool = Staking::etp_pool();
			assert_ok!(Staking::payout_stakers(Origin::signed(10), 11, 1));
			assert_eq!(Staking::etp_pool(), payout + etp_pool);
		});
}

#[test]
fn unbond_over_max_unbondings_chunks_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456));
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(COIN),
			RewardDestination::Stash,
			0,
		));

		for ts in 0..MAX_UNLOCKING_CHUNKS {
			Timestamp::set_timestamp(ts as u64);

			assert_ok!(Staking::unbond(
				Origin::signed(controller),
				StakingBalance::EtpBalance(1)
			));
		}

		assert_err!(
			Staking::unbond(Origin::signed(controller), StakingBalance::EtpBalance(1)),
			StakingError::NoMoreChunks
		);
	});
}

#[test]
fn promise_extra_should_not_remove_unexpired_items() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456), promise_month(12));
		let expired_items_len = 3;
		let expiry_date = INIT_TIMESTAMP + promise_month * MONTH_IN_MILLISECONDS;

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(5 * COIN),
			0,
		));
		for _ in 0..expired_items_len {
			assert_ok!(Staking::deposit_extra(
				Origin::signed(stash),
				COIN,
				promise_month as u8
			));
		}

		Timestamp::set_timestamp(expiry_date - 1);
		assert_ok!(Staking::deposit_extra(
			Origin::signed(stash),
			2 * COIN,
			promise_month as u8,
		));
		assert_eq!(
			Staking::ledger(controller).unwrap().deposit_items.len(),
			2 + expired_items_len,
		);

		Timestamp::set_timestamp(expiry_date);
		assert_ok!(Staking::deposit_extra(
			Origin::signed(stash),
			2 * COIN,
			promise_month as u8,
		));
		assert_eq!(Staking::ledger(controller).unwrap().deposit_items.len(), 2);
	});
}

#[test]
fn unbond_zero() {
	ExtBuilder::default().build().execute_with(|| {
		gen_paired_account!(stash(123), controller(456), promise_month(12));
		let ledger = Staking::ledger(controller).unwrap();

		Timestamp::set_timestamp(promise_month * MONTH_IN_MILLISECONDS);
		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::EtpBalance(0)
		));
		assert_ok!(Staking::unbond(
			Origin::signed(10),
			StakingBalance::DnaBalance(0)
		));
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
	});
}

#[test]
fn on_deposit_redeem_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let deposit_amount = 1;
		let deposit_start_at = 1;
		let deposit_months = 3;
		let backing_account = 1;
		let deposit_item = TimeDepositItem {
			value: deposit_amount,
			start_time: deposit_start_at * 1000,
			expire_time: deposit_start_at * 1000 + deposit_months as TsInMs * MONTH_IN_MILLISECONDS,
		};

		// Not bond yet
		{
			let unbonded_account = 123;
			let etp_pool = Staking::etp_pool();

			assert_eq!(Etp::free_balance(unbonded_account), 0);
			assert!(Etp::locks(unbonded_account).is_empty());
			assert!(Staking::bonded(unbonded_account).is_none());
			assert_eq!(
				Staking::payee(unbonded_account),
				RewardDestination::default(),
			);
			assert!(Staking::ledger(unbonded_account).is_none());
			assert!(System::account(unbonded_account).providers == 0);

			assert_ok!(Staking::on_deposit_redeem(
				&backing_account,
				&unbonded_account,
				deposit_amount,
				deposit_start_at,
				deposit_months
			));

			assert_eq!(Etp::free_balance(unbonded_account), deposit_amount);
			assert_eq!(
				Etp::locks(unbonded_account),
				vec![BalanceLock {
					id: STAKING_ID,
					lock_for: LockFor::Staking(StakingLock {
						staking_amount: deposit_amount,
						unbondings: vec![],
					}),
					lock_reasons: LockReasons::All,
				}]
			);
			assert_eq!(Staking::bonded(unbonded_account).unwrap(), unbonded_account);
			assert_eq!(Staking::payee(unbonded_account), RewardDestination::Stash);
			assert_eq!(
				Staking::ledger(unbonded_account).unwrap(),
				StakingLedger {
					stash: unbonded_account,
					active_etp: deposit_amount,
					active_deposit_etp: deposit_amount,
					deposit_items: vec![deposit_item.clone()],
					etp_staking_lock: StakingLock {
						staking_amount: deposit_amount,
						unbondings: vec![]
					},
					..Default::default()
				}
			);
			assert_eq!(Staking::etp_pool(), etp_pool + deposit_amount);
			assert!(System::account(unbonded_account).providers != 0);
		}

		// Already bonded
		{
			gen_paired_account!(bonded_account(456), bonded_account(456), 0);

			let etp_pool = Staking::etp_pool();
			let mut ledger = Staking::ledger(bonded_account).unwrap();

			assert_eq!(Etp::free_balance(bonded_account), 101 * COIN);
			assert_eq!(Etp::locks(bonded_account).len(), 1);
			assert_eq!(Staking::bonded(bonded_account).unwrap(), bonded_account);

			assert_ok!(Staking::on_deposit_redeem(
				&backing_account,
				&bonded_account,
				deposit_amount,
				deposit_start_at,
				deposit_months
			));

			ledger.active_etp += deposit_amount;
			ledger.active_deposit_etp += deposit_amount;
			ledger.deposit_items.push(deposit_item);
			ledger.etp_staking_lock.staking_amount += deposit_amount;

			assert_eq!(
				Etp::free_balance(bonded_account),
				101 * COIN + deposit_amount
			);
			assert_eq!(
				Etp::locks(bonded_account),
				vec![BalanceLock {
					id: STAKING_ID,
					lock_for: LockFor::Staking(StakingLock {
						staking_amount: 50 * COIN + deposit_amount,
						unbondings: vec![],
					}),
					lock_reasons: LockReasons::All,
				}]
			);
			assert_eq!(Staking::ledger(bonded_account).unwrap(), ledger);
			assert_eq!(Staking::etp_pool(), etp_pool + deposit_amount);
		}
	});
}

// Origin test case name is `yakio_q1`
// bond 10_000 Etp for 12 months, gain 1 Dna
// bond extra 10_000 Etp for 36 months, gain 3 Dna
// bond extra 1 Dna
// nominate
// unlock the 12 months deposit item with punish
// lost 3 Dna and 10_000 Etp's power for nominate
#[test]
fn two_different_bond_then_unbond_specific_one() {
	ExtBuilder::default().build().execute_with(|| {
		let (stash, controller) = (777, 888);
		let _ = Etp::deposit_creating(&stash, 20_000);

		// Earn 1 Dna with bond 10_000 Etp 12 months
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(10_000),
			RewardDestination::Stash,
			12,
		));

		// Earn 3 Dna with bond 10_000 Etp 36 months
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(10_000),
			36,
		));

		assert_eq!(Dna::free_balance(&stash), 4);

		// Bond 1 Dna
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(1),
			36
		));
		assert_eq!(Staking::ledger(controller).unwrap().active_dna, 1);

		// Become a nominator
		assert_ok!(Staking::nominate(
			Origin::signed(controller),
			vec![controller]
		));

		// Then unbond the the first 12 months part,
		// this behavior should be punished 3 times Dna according to the remaining times
		// 3 times * 1 Dna * 12 months(remaining) / 12 months(promised)
		assert_ok!(Staking::try_claim_deposits_with_punish(
			Origin::signed(controller),
			INIT_TIMESTAMP + 12 * MONTH_IN_MILLISECONDS,
		));
		assert_eq!(Dna::free_balance(&stash), 1);

		let ledger = Staking::ledger(controller).unwrap();

		// Please Note:
		// not enough Dna to unbond, but the function will not fail
		assert_ok!(Staking::try_claim_deposits_with_punish(
			Origin::signed(controller),
			INIT_TIMESTAMP + 36 * MONTH_IN_MILLISECONDS,
		));
		assert_eq!(Staking::ledger(controller).unwrap(), ledger);
	});
}

#[test]
fn staking_with_dna_with_unbondings() {
	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Dna::deposit_creating(&stash, 10);

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::DnaBalance(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}],
		);

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(5),
			0
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 10,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let unbond_start = System::block_number();
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::DnaBalance(9)
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					},],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_err!(
			Dna::transfer(Origin::signed(stash), controller, 1),
			DnaError::LiquidityRestrictions,
		);

		System::set_block_number(unbond_start + BondingDurationInBlockNumber::get());
		assert_ok!(Dna::transfer(Origin::signed(stash), controller, 1));
		assert_eq!(
			System::block_number(),
			unbond_start + BondingDurationInBlockNumber::get()
		);
		assert_eq!(Dna::free_balance(stash), 9);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					},],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let _ = Dna::deposit_creating(&stash, 20);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(19),
			0
		));
		assert_eq!(Dna::free_balance(stash), 29);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 20,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					},],
				}),
				lock_reasons: LockReasons::All,
			}]
		);
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_dna: 20,
				dna_staking_lock: StakingLock {
					staking_amount: 20,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					}],
				},
				..Default::default()
			}
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Etp::deposit_creating(&stash, 10);

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(5),
			0
		));
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 10,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let unbond_start = System::block_number();
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(9)
		));
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					},],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_err!(
			Etp::transfer(Origin::signed(stash), controller, 1),
			EtpError::LiquidityRestrictions,
		);

		System::set_block_number(BondingDurationInBlockNumber::get() + unbond_start);
		assert_ok!(Etp::transfer(Origin::signed(stash), controller, 1));
		assert_eq!(
			System::block_number(),
			BondingDurationInBlockNumber::get() + unbond_start
		);
		assert_eq!(Etp::free_balance(stash), 9);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					},],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let _ = Etp::deposit_creating(&stash, 20);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(19),
			0
		));
		assert_eq!(Etp::free_balance(stash), 29);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 20,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					}],
				}),
				lock_reasons: LockReasons::All,
			}]
		);
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_etp: 20,
				etp_staking_lock: StakingLock {
					staking_amount: 20,
					unbondings: vec![Unbonding {
						amount: 9,
						until: BondingDurationInBlockNumber::get() + unbond_start,
					}],
				},
				..Default::default()
			}
		);
	});
}

#[test]
fn unbound_values_in_twice() {
	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Dna::deposit_creating(&stash, 10);

		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::DnaBalance(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(4),
			0
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 9,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let (unbond_start_1, unbond_value_1) = (System::block_number(), 2);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::DnaBalance(unbond_value_1),
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 7,
					unbondings: vec![Unbonding {
						amount: 2,
						until: BondingDurationInBlockNumber::get() + unbond_start_1,
					}],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let (unbond_start_2, unbond_value_2) = (System::block_number(), 6);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::DnaBalance(6)
		));
		assert_eq!(Dna::free_balance(stash), 10);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_err!(
			Dna::transfer(Origin::signed(stash), controller, unbond_value_1),
			DnaError::LiquidityRestrictions,
		);
		assert_ok!(Dna::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_1 - 1
		));
		assert_eq!(Dna::free_balance(stash), 9);

		assert_err!(
			Dna::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
			DnaError::LiquidityRestrictions,
		);
		System::set_block_number(BondingDurationInBlockNumber::get() + unbond_start_1);
		assert_ok!(Dna::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_1
		));
		assert_eq!(
			System::block_number(),
			BondingDurationInBlockNumber::get() + unbond_start_1
		);
		assert_eq!(Dna::free_balance(stash), 7);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_ok!(Dna::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_2
		));
		assert_eq!(Dna::free_balance(stash), 1);
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let _ = Dna::deposit_creating(&stash, 1);
		assert_eq!(Dna::free_balance(stash), 2);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(1),
			0
		));
		assert_eq!(
			Dna::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 2,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);
	});

	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Etp::deposit_creating(&stash, 10);

		Timestamp::set_timestamp(1);
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(5),
			RewardDestination::Stash,
			0,
		));
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(4),
			0
		));
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 9,
					unbondings: vec![],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let (unbond_start_1, unbond_value_1) = (System::block_number(), 2);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(unbond_value_1)
		));
		assert_eq!(System::block_number(), unbond_start_1);
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 7,
					unbondings: vec![Unbonding {
						amount: 2,
						until: BondingDurationInBlockNumber::get() + unbond_start_1,
					}],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let (unbond_start_2, unbond_value_2) = (System::block_number(), 6);
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(6)
		));
		assert_eq!(System::block_number(), unbond_start_2);
		assert_eq!(Etp::free_balance(stash), 10);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		assert_err!(
			Etp::transfer(Origin::signed(stash), controller, unbond_value_1),
			EtpError::LiquidityRestrictions,
		);

		assert_ok!(Etp::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_1 - 1
		));
		assert_eq!(Etp::free_balance(stash), 9);
		assert_err!(
			Etp::transfer(Origin::signed(stash), controller, unbond_value_1 + 1),
			EtpError::LiquidityRestrictions,
		);
		System::set_block_number(BondingDurationInBlockNumber::get() + unbond_start_1);
		assert_ok!(Etp::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_1
		));
		assert_eq!(
			System::block_number(),
			BondingDurationInBlockNumber::get() + unbond_start_1
		);
		assert_eq!(Etp::free_balance(stash), 7);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);
		assert_ok!(Etp::transfer(
			Origin::signed(stash),
			controller,
			unbond_value_2
		));
		assert_eq!(
			System::block_number(),
			BondingDurationInBlockNumber::get() + unbond_start_2
		);
		assert_eq!(Etp::free_balance(stash), 1);
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 1,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);

		let _ = Etp::deposit_creating(&stash, 1);
		//		println!("Staking Ledger: {:#?}", Staking::ledger(controller).unwrap());
		assert_eq!(Etp::free_balance(stash), 2);
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(1),
			0
		));
		assert_eq!(
			Etp::locks(stash),
			vec![BalanceLock {
				id: STAKING_ID,
				lock_for: LockFor::Staking(StakingLock {
					staking_amount: 2,
					unbondings: vec![
						Unbonding {
							amount: 2,
							until: BondingDurationInBlockNumber::get() + unbond_start_1,
						},
						Unbonding {
							amount: 6,
							until: BondingDurationInBlockNumber::get() + unbond_start_2,
						}
					],
				}),
				lock_reasons: LockReasons::All,
			}]
		);
	});
}

// Original testcase name is `xavier_q3`
//
// The values(DNA, ETP) are unbond in the moment that there are values unbonding
#[test]
fn bond_values_when_some_value_unbonding() {
	// The Dna part
	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Dna::deposit_creating(&stash, 10);

		let start = System::block_number();
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::DnaBalance(5),
			RewardDestination::Stash,
			0,
		));

		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_dna: 5,
				dna_staking_lock: StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				},
				..Default::default()
			},
		);

		// all values are unbond
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::DnaBalance(5)
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				dna_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![Unbonding {
						amount: 5,
						until: start + BondingDurationInBlockNumber::get(),
					}],
				},
				..Default::default()
			},
		);

		System::set_block_number(start + BondingDurationInBlockNumber::get());
		// unbond zero to release unbondings
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::DnaBalance(0)
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				..Default::default()
			},
		);

		// bond again
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::DnaBalance(1),
			0,
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_dna: 1,
				dna_staking_lock: StakingLock {
					staking_amount: 1,
					unbondings: vec![],
				},
				..Default::default()
			},
		);
	});

	// The Etp part
	ExtBuilder::default().build().execute_with(|| {
		let stash = 123;
		let controller = 456;
		let _ = Etp::deposit_creating(&stash, 10);

		let start = System::block_number();
		assert_ok!(Staking::bond(
			Origin::signed(stash),
			controller,
			StakingBalance::EtpBalance(5),
			RewardDestination::Stash,
			0,
		));

		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_etp: 5,
				etp_staking_lock: StakingLock {
					staking_amount: 5,
					unbondings: vec![],
				},
				..Default::default()
			},
		);

		// all values are unbond
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(5),
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				etp_staking_lock: StakingLock {
					staking_amount: 0,
					unbondings: vec![Unbonding {
						amount: 5,
						until: start + BondingDurationInBlockNumber::get(),
					}],
				},
				..Default::default()
			},
		);

		System::set_block_number(start + BondingDurationInBlockNumber::get());
		// unbond zero to release unbondings
		assert_ok!(Staking::unbond(
			Origin::signed(controller),
			StakingBalance::EtpBalance(0),
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				..Default::default()
			},
		);

		// bond again
		assert_ok!(Staking::bond_extra(
			Origin::signed(stash),
			StakingBalance::EtpBalance(1),
			0,
		));
		assert_eq!(
			Staking::ledger(controller).unwrap(),
			StakingLedger {
				stash: 123,
				active_etp: 1,
				etp_staking_lock: StakingLock {
					staking_amount: 1,
					unbondings: vec![],
				},
				..Default::default()
			}
		);
	});
}

#[test]
fn rebond_event_should_work() {
	ExtBuilder::default()
		.nominate(false)
		.build()
		.execute_with(|| {
			assert_ok!(Staking::set_payee(
				Origin::signed(10),
				RewardDestination::Controller
			));

			let _ = Etp::make_free_balance_be(&11, 1000000);

			run_to_block(5);

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

			System::reset_events();

			// Re-bond half of the unbonding funds.
			Staking::rebond(Origin::signed(10), 200, 0).unwrap();
			assert_eq!(
				Staking::ledger(&10),
				Some(StakingLedger {
					stash: 11,
					active_etp: 800,
					etp_staking_lock: StakingLock {
						staking_amount: 800,
						unbondings: vec![Unbonding {
							amount: 200,
							until: 6 + BondingDurationInBlockNumber::get(),
						},]
					},
					..Default::default()
				})
			);
			assert_eq!(
				staking_events(),
				vec![RawEvent::BondEtp(200, 36000, 36000)]
			);
		});
}
