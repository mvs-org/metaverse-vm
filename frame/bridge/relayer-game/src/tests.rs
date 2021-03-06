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

// --- substrate ---
use frame_support::{assert_err, assert_ok};
// --- hyperspace ---
use crate::{
	mock::{mock_relay::*, BlockNumber, *},
	*,
};
use hyperspace_support::balance::lock::*;

// #[test]
// fn events_should_work() {
// 	ExtBuilder::default()
// 		.confirmed_period(3)
// 		.build()
// 		.execute_with(|| {
// 			run_to_block(1);

// 			let relayer_a = 1;
// 			let relayer_b = 2;
// 			let relay_header_parcels_a =
// 				MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
// 			let relay_header_parcels_b =
// 				MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
// 			let game_id = relay_header_parcels_a.len() as _;
// 			let round_index = relay_header_parcels_a.len() as _;

// 			assert_ok!(RelayerGame::affirm(
// 				&relayer_a,
// 				relay_header_parcels_a[0].clone(),
// 				Some(())
// 			));
// 			assert_eq!(
// 				relayer_game_events(),
// 				vec![Event::relayer_game(RawEvent::Affirmed(
// 					game_id,
// 					0,
// 					0,
// 					relayer_a
// 				))]
// 			);

// 			assert_ok!(RelayerGame::dispute_and_affirm(
// 				&relayer_b,
// 				relay_header_parcels_b[0].clone(),
// 				Some(())
// 			));
// 			assert_eq!(
// 				relayer_game_events(),
// 				vec![
// 					Event::relayer_game(RawEvent::Disputed(game_id)),
// 					Event::relayer_game(RawEvent::Affirmed(game_id, 0, 1, relayer_b))
// 				]
// 			);

// 			run_to_block(challenge_time() * 1 as BlockNumber + 2);

// 			assert_eq!(
// 				relayer_game_events(),
// 				vec![Event::relayer_game(RawEvent::NewRound(5, vec![4]))]
// 			);

// 			for i in 1..round_index {
// 				assert_ok!(RelayerGame::extend_affirmation(
// 					&relayer_a,
// 					RelayAffirmationId {
// 						game_id,
// 						round: i - 1,
// 						index: 0
// 					},
// 					vec![relay_header_parcels_a[i as usize].clone()],
// 					Some(vec![()])
// 				));
// 				assert_eq!(
// 					relayer_game_events(),
// 					vec![
// 						Event::relayer_game(RawEvent::Extended(game_id)),
// 						Event::relayer_game(RawEvent::Affirmed(game_id, i, 0, relayer_a,))
// 					]
// 				);

// 				assert_ok!(RelayerGame::extend_affirmation(
// 					&relayer_b,
// 					RelayAffirmationId {
// 						game_id,
// 						round: i - 1,
// 						index: 1
// 					},
// 					vec![relay_header_parcels_b[i as usize].clone()],
// 					Some(vec![()])
// 				));
// 				assert_eq!(
// 					relayer_game_events(),
// 					vec![
// 						Event::relayer_game(RawEvent::Extended(game_id)),
// 						Event::relayer_game(RawEvent::Affirmed(game_id, i, 1, relayer_b))
// 					]
// 				);

// 				run_to_block(challenge_time() * (i as BlockNumber + 1) + 2);

// 				if i == round_index - 1 {
// 					assert_eq!(
// 						relayer_game_events(),
// 						vec![
// 							Event::relayer_game(RawEvent::GameOver(game_id)),
// 							Event::relayer_game(RawEvent::Pended(game_id))
// 						]
// 					);
// 				} else {
// 					assert_eq!(
// 						relayer_game_events(),
// 						vec![Event::relayer_game(RawEvent::NewRound(
// 							game_id,
// 							vec![round_index - i - 1]
// 						))]
// 					);
// 				}
// 			}

// 			run_to_block(challenge_time() * (round_index as BlockNumber + 1) + 2);

// 			assert_eq!(
// 				relayer_game_events(),
// 				vec![Event::relayer_game(
// 					RawEvent::PendingRelayHeaderParcelApproved(
// 						game_id,
// 						b"Not Enough Technical Member Online, Approved By System".to_vec()
// 					)
// 				),]
// 			);
// 		});
// }

#[test]
fn insufficient_bond_should_fail() {
	ExtBuilder::default()
		.estimate_stake(101)
		.build()
		.execute_with(|| {
			let relay_header_parcels = MockRelayHeader::gen_continous(1, vec![1, 1], true);

			{
				let poor_man = 0;

				assert_err!(
					RelayerGame::affirm(&poor_man, relay_header_parcels[0].clone(), None),
					RelayerGameError::StakeIns
				);
			}

			assert_err!(
				RelayerGame::affirm(&1, relay_header_parcels[0].clone(), None),
				RelayerGameError::StakeIns
			);
			assert_ok!(RelayerGame::affirm(
				&2,
				MockRelayHeader::gen(2, 0, 1),
				Some(())
			));
			assert_ok!(RelayerGame::dispute_and_affirm(
				&3,
				relay_header_parcels[0].clone(),
				Some(())
			));

			run_to_block(4);

			assert_err!(
				RelayerGame::affirm(&2, relay_header_parcels[1].clone(), None),
				RelayerGameError::StakeIns
			);
			assert_ok!(RelayerGame::affirm(
				&3,
				relay_header_parcels[1].clone(),
				None
			));
		});
}

#[test]
fn some_affirm_cases_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let relay_header_parcel_a = MockRelayHeader::gen(1, 0, 1);
		let relay_header_parcel_b = MockRelayHeader::gen(1, 0, 1);

		assert_err!(
			RelayerGame::dispute_and_affirm(&1, relay_header_parcel_a.clone(), None),
			RelayerGameError::GameAtThisRoundC
		);
		assert_ok!(RelayerGame::affirm(&1, relay_header_parcel_a, None));
		assert_err!(
			RelayerGame::affirm(&1, relay_header_parcel_b, None),
			RelayerGameError::ExistedAffirmationsFoundC
		);
	});
}

#[test]
fn already_confirmed_should_fail() {
	let relay_header_parcels = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);

	ExtBuilder::default()
		.headers(relay_header_parcels.clone())
		.build()
		.execute_with(|| {
			let relayer = 1;

			for relay_header_parcel in relay_header_parcels {
				assert_err!(
					RelayerGame::affirm(&relayer, relay_header_parcel, None),
					RelayerGameError::RelayParcelAR
				);
			}
		});
}

#[test]
fn duplicate_game_should_fail() {
	ExtBuilder::default().build().execute_with(|| {
		let relay_header_parcel = MockRelayHeader::gen(1, 0, 1);

		assert_ok!(RelayerGame::affirm(&1, relay_header_parcel.clone(), None));
		assert_err!(
			RelayerGame::dispute_and_affirm(&2, relay_header_parcel, None),
			RelayerGameError::RelayAffirmationDup
		);
	});
}

// #[test]
// fn jump_round_should_fail() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		let proposal = MockTcHeader::mock_proposal(vec![1, 1, 1, 1, 1], true);

// 		assert_ok!(RelayerGame::affirm(
// 			1,
// 			proposal[..1].to_vec()
// 		));

// 		for i in 2..=5 {
// 			assert_err!(
// 				RelayerGame::affirm(&1, proposal[..i].to_vec()),
// 				RelayerGameError::RoundMis
// 			);
// 		}
// 	});
// }

#[test]
fn challenge_time_should_work() {
	for &challenge_time in [4, 6, 8].iter() {
		ExtBuilder::default()
			.challenge_time(challenge_time)
			.build()
			.execute_with(|| {
				let relay_header_parcel = MockRelayHeader::gen(1, 0, 1);

				assert_ok!(RelayerGame::affirm(&1, relay_header_parcel.clone(), None));

				for block in 0..=challenge_time {
					run_to_block(block);

					assert_eq!(
						RelayerGame::affirmations_of_game_at(relay_header_parcel.number, 0).len(),
						1
					);
					assert!(Relay::confirmed_header_of(relay_header_parcel.number).is_none());
				}

				run_to_block(challenge_time + 1);

				assert!(
					RelayerGame::affirmations_of_game_at(relay_header_parcel.number, 1).is_empty()
				);
				assert_eq!(
					Relay::confirmed_header_of(relay_header_parcel.number),
					Some(relay_header_parcel)
				);
			});
	}
}

#[test]
fn extend_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let relayer_a = 1;
		let relayer_b = 2;
		let relay_header_parcels_a = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
		let relay_header_parcels_b = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
		let game_id = relay_header_parcels_a.len() as _;
		let round_index = relay_header_parcels_a.len() as _;

		assert_ok!(RelayerGame::affirm(
			&relayer_a,
			relay_header_parcels_a[0].clone(),
			Some(())
		));
		assert_ok!(RelayerGame::dispute_and_affirm(
			&relayer_b,
			relay_header_parcels_b[0].clone(),
			Some(())
		));

		// println_game(3);

		for i in 1..round_index {
			run_to_block(challenge_time() * i as BlockNumber + 1);

			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_a,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 0
				},
				vec![relay_header_parcels_a[i as usize].clone()],
				Some(vec![()])
			));
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_b,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 1
				},
				vec![relay_header_parcels_b[i as usize].clone()],
				Some(vec![()])
			));
		}
	});
}

#[test]
fn lock_should_work() {
	for estimate_stake in 1..=3 {
		ExtBuilder::default()
			.estimate_stake(estimate_stake)
			.build()
			.execute_with(|| {
				let relayer_a = 1;
				let relayer_b = 2;
				let relay_header_parcels_a =
					MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
				let relay_header_parcels_b =
					MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
				let game_id = relay_header_parcels_a.len() as _;
				let round_index = relay_header_parcels_a.len() as _;
				let submit_then_assert = |relayer, relay_parcel, round, index, stakes| {
					assert_ok!(RelayerGame::extend_affirmation(
						relayer,
						RelayAffirmationId {
							game_id,
							round,
							index,
						},
						vec![relay_parcel],
						Some(vec![()])
					));
					assert_eq!(RelayerGame::stakes_of(relayer), stakes);
					assert_eq!(
						Etp::locks(relayer),
						vec![BalanceLock {
							id: <Test as Config>::LockId::get(),
							lock_for: LockFor::Common { amount: stakes },
							lock_reasons: LockReasons::All
						}]
					);
				};

				assert_ok!(RelayerGame::affirm(
					&relayer_a,
					relay_header_parcels_a[0].clone(),
					Some(())
				));
				assert_ok!(RelayerGame::dispute_and_affirm(
					&relayer_b,
					relay_header_parcels_b[0].clone(),
					Some(())
				));

				run_to_block(challenge_time() * 1 + 1);

				let mut stakes = estimate_stake;

				for i in 1..round_index {
					stakes += estimate_stake;

					submit_then_assert(
						&relayer_a,
						relay_header_parcels_a[i as usize].clone(),
						i - 1,
						0,
						stakes,
					);
					submit_then_assert(
						&relayer_b,
						relay_header_parcels_b[i as usize].clone(),
						i - 1,
						1,
						stakes,
					);

					run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);
				}

				assert_eq!(RelayerGame::stakes_of(relayer_a), 0);
				assert!(Etp::locks(1).is_empty());

				assert_eq!(RelayerGame::stakes_of(relayer_b), 0);
				assert!(Etp::locks(2).is_empty());
			});
	}
}

#[test]
fn slash_and_reward_should_work() {
	for estimate_stake in vec![1, 5, 10, 20, 50, 100] {
		ExtBuilder::default()
			.estimate_stake(estimate_stake)
			.build()
			.execute_with(|| {
				let relayer_a = 10;
				let relayer_b = 20;
				let relay_header_parcels_a =
					MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
				let relay_header_parcels_b =
					MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
				let game_id = relay_header_parcels_a.len() as _;
				let round_index = relay_header_parcels_a.len() as _;
				let mut stakes = estimate_stake;

				assert_eq!(Etp::usable_balance(&relayer_a), 1000);
				assert_eq!(Etp::usable_balance(&relayer_b), 2000);

				assert_ok!(RelayerGame::affirm(
					&relayer_a,
					relay_header_parcels_a[0].clone(),
					Some(())
				));
				assert_ok!(RelayerGame::dispute_and_affirm(
					&relayer_b,
					relay_header_parcels_b[0].clone(),
					Some(())
				));

				run_to_block(challenge_time() * 1 + 1);

				for i in 1..round_index {
					assert_ok!(RelayerGame::extend_affirmation(
						&relayer_a,
						RelayAffirmationId {
							game_id,
							round: i - 1,
							index: 0
						},
						vec![relay_header_parcels_a[i as usize].clone()],
						Some(vec![()])
					));
					assert_ok!(RelayerGame::extend_affirmation(
						&relayer_b,
						RelayAffirmationId {
							game_id,
							round: i - 1,
							index: 1
						},
						vec![relay_header_parcels_b[i as usize].clone()],
						Some(vec![()])
					));

					run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);

					stakes += estimate_stake;
				}

				assert_eq!(Etp::usable_balance(&relayer_a), 1000 + stakes);
				assert!(Etp::locks(relayer_a).is_empty());

				assert_eq!(Etp::usable_balance(&relayer_b), 2000 - stakes);
				assert!(Etp::locks(relayer_b).is_empty());
			});
	}
}

#[test]
fn settle_without_challenge_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		for (relay_header_parcel, i) in MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true)
			.into_iter()
			.rev()
			.zip(1..)
		{
			assert_ok!(RelayerGame::affirm(&1, relay_header_parcel.clone(), None));
			assert!(Etp::usable_balance(&1) < 100);
			assert!(!Etp::locks(1).is_empty());

			run_to_block(7 * i);

			assert_eq!(
				Relay::confirmed_header_of(relay_header_parcel.number),
				Some(relay_header_parcel)
			);
			assert_eq!(Etp::usable_balance(&1), 100);
			assert!(Etp::locks(1).is_empty());
		}
	})
}

#[test]
fn settle_with_challenge_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let relayer_a = 1;
		let relayer_b = 2;
		let relay_header_parcels_a = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
		let relay_header_parcels_b = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
		let game_id = relay_header_parcels_a.len() as _;
		let round_index = relay_header_parcels_a.len() as u32 - 1;

		assert_ok!(RelayerGame::affirm(
			&relayer_a,
			relay_header_parcels_a[0].clone(),
			Some(())
		));
		assert_ok!(RelayerGame::dispute_and_affirm(
			&relayer_b,
			relay_header_parcels_b[0].clone(),
			Some(())
		));

		run_to_block(challenge_time() * 1 + 1);

		for i in 1..round_index {
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_a,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 0
				},
				vec![relay_header_parcels_a[i as usize].clone()],
				Some(vec![()])
			));
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_b,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 1
				},
				vec![relay_header_parcels_b[i as usize].clone()],
				Some(vec![()])
			));

			run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);
		}

		assert_ok!(RelayerGame::extend_affirmation(
			&relayer_a,
			RelayAffirmationId {
				game_id,
				round: round_index - 1,
				index: 0
			},
			vec![relay_header_parcels_a[round_index as usize].clone()],
			Some(vec![()])
		));

		run_to_block(challenge_time() * 5 + 1);

		let relay_header_parcel = relay_header_parcels_a[0].clone();

		assert_eq!(
			Relay::confirmed_header_of(relay_header_parcel.number),
			Some(relay_header_parcel)
		);
	});
}

#[test]
fn settle_abandon_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let relayer_a = 1;
		let relayer_b = 2;
		let relay_header_parcels_a = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
		let relay_header_parcels_b = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
		let game_id = relay_header_parcels_a.len() as _;
		let round_index = relay_header_parcels_a.len() as u32 - 1;

		assert_eq!(Etp::usable_balance(&relayer_a), 100);
		assert_eq!(Etp::usable_balance(&relayer_b), 200);

		assert_ok!(RelayerGame::affirm(
			&relayer_a,
			relay_header_parcels_a[0].clone(),
			Some(())
		));
		assert_ok!(RelayerGame::dispute_and_affirm(
			&relayer_b,
			relay_header_parcels_b[0].clone(),
			Some(())
		));

		run_to_block(challenge_time() * 1 + 1);

		for i in 1..round_index {
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_a,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 0
				},
				vec![relay_header_parcels_a[i as usize].clone()],
				Some(vec![()])
			));
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_b,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 1
				},
				vec![relay_header_parcels_b[i as usize].clone()],
				Some(vec![()])
			));

			run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);
		}

		run_to_block(challenge_time() * 5 + 1);

		assert_eq!(Etp::usable_balance(&relayer_a), 100 - 4);
		assert!(Etp::locks(relayer_a).is_empty());

		assert_eq!(Etp::usable_balance(&relayer_b), 200 - 4);
		assert!(Etp::locks(relayer_b).is_empty());
	});
}

#[test]
fn on_chain_arbitrate_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let relayer_a = 10;
		let relayer_b = 20;
		let relay_header_parcels_a = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], true);
		let relay_header_parcels_b = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
		let game_id = relay_header_parcels_a.len() as _;
		let round_index = relay_header_parcels_a.len() as _;

		assert_ok!(RelayerGame::affirm(
			&relayer_a,
			relay_header_parcels_a[0].clone(),
			Some(())
		));
		assert_ok!(RelayerGame::dispute_and_affirm(
			&relayer_b,
			relay_header_parcels_b[0].clone(),
			Some(())
		));

		run_to_block(challenge_time() * 1 + 1);

		for i in 1..round_index {
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_a,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 0
				},
				vec![relay_header_parcels_a[i as usize].clone()],
				Some(vec![()])
			));
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_b,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 1
				},
				vec![relay_header_parcels_b[i as usize].clone()],
				Some(vec![()])
			));

			run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);
		}

		let relay_header_parcel = relay_header_parcels_a[0].clone();

		assert_eq!(
			Relay::confirmed_header_of(relay_header_parcel.number),
			Some(relay_header_parcel)
		);
	});
}

#[test]
fn no_honesty_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		let relayer_a = 1;
		let relayer_b = 2;
		let relay_header_parcels_a = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
		let relay_header_parcels_b = MockRelayHeader::gen_continous(1, vec![1, 1, 1, 1, 1], false);
		let game_id = relay_header_parcels_a.len() as _;
		let round_index = relay_header_parcels_a.len() as _;

		assert_eq!(Etp::usable_balance(&relayer_a), 100);
		assert_eq!(Etp::usable_balance(&relayer_b), 200);

		assert_ok!(RelayerGame::affirm(
			&relayer_a,
			relay_header_parcels_a[0].clone(),
			Some(())
		));
		assert_ok!(RelayerGame::dispute_and_affirm(
			&relayer_b,
			relay_header_parcels_b[0].clone(),
			Some(())
		));

		run_to_block(challenge_time() * 1 + 1);

		for i in 1..round_index {
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_a,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 0
				},
				vec![relay_header_parcels_a[i as usize].clone()],
				Some(vec![()])
			));
			assert_ok!(RelayerGame::extend_affirmation(
				&relayer_b,
				RelayAffirmationId {
					game_id,
					round: i - 1,
					index: 1
				},
				vec![relay_header_parcels_b[i as usize].clone()],
				Some(vec![()])
			));

			run_to_block(challenge_time() * (i as BlockNumber + 1) + 1);
		}

		assert_eq!(Etp::usable_balance(&relayer_a), 100 - 5);
		assert!(Etp::locks(relayer_a).is_empty());

		assert_eq!(Etp::usable_balance(&relayer_b), 200 - 5);
		assert!(Etp::locks(relayer_b).is_empty());
	});
}
