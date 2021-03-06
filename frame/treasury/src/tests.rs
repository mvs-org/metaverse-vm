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

//! Tests for treasury.

// --- substrate ---
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use sp_runtime::traits::BlakeTwo256;
// --- hyperspace ---
use crate::{mock::*, *};

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Treasury::pot::<Etp>(), 0);
		assert_eq!(Treasury::proposal_count(), 0);
	});
}

#[test]
fn tip_new_cannot_be_used_twice() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::tip_new(
			Origin::signed(10),
			b"awesome.hyperspace".to_vec(),
			3,
			10
		));
		assert_noop!(
			Treasury::tip_new(Origin::signed(11), b"awesome.hyperspace".to_vec(), 3, 10),
			<Error<Test, _>>::AlreadyKnown
		);
	});
}

#[test]
fn report_awesome_and_tip_works() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::report_awesome(
			Origin::signed(0),
			b"awesome.hyperspace".to_vec(),
			3
		));
		assert_eq!(Etp::reserved_balance(0), 17);
		assert_eq!(Etp::free_balance(0), 83);

		// other reports don't count.
		assert_noop!(
			Treasury::report_awesome(Origin::signed(1), b"awesome.hyperspace".to_vec(), 3),
			<Error<Test, _>>::AlreadyKnown
		);

		let h = tip_hash();
		assert_ok!(Treasury::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(Treasury::tip(Origin::signed(9), h.clone(), 10), BadOrigin);
		System::set_block_number(2);
		assert_ok!(Treasury::close_tip(Origin::signed(100), h.into()));
		assert_eq!(Etp::reserved_balance(0), 0);
		assert_eq!(Etp::free_balance(0), 102);
		assert_eq!(Etp::free_balance(3), 8);
	});
}

#[test]
fn report_awesome_from_beneficiary_and_tip_works() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::report_awesome(
			Origin::signed(0),
			b"awesome.hyperspace".to_vec(),
			0
		));
		assert_eq!(Etp::reserved_balance(0), 17);
		assert_eq!(Etp::free_balance(0), 83);
		let h = BlakeTwo256::hash_of(&(BlakeTwo256::hash(b"awesome.hyperspace"), 0u128));
		assert_ok!(Treasury::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10));
		System::set_block_number(2);
		assert_ok!(Treasury::close_tip(Origin::signed(100), h.into()));
		assert_eq!(Etp::reserved_balance(0), 0);
		assert_eq!(Etp::free_balance(0), 110);
	});
}

#[test]
fn close_tip_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);

		assert_ok!(Treasury::tip_new(
			Origin::signed(10),
			b"awesome.hyperspace".to_vec(),
			3,
			10
		));

		let h = tip_hash();

		assert_eq!(last_event(), RawEvent::NewTip(h));

		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));

		assert_noop!(
			Treasury::close_tip(Origin::signed(0), h.into()),
			<Error<Test, _>>::StillOpen
		);

		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10));

		assert_eq!(last_event(), RawEvent::TipClosing(h));

		assert_noop!(
			Treasury::close_tip(Origin::signed(0), h.into()),
			<Error<Test, _>>::Premature
		);

		System::set_block_number(2);
		assert_noop!(Treasury::close_tip(Origin::none(), h.into()), BadOrigin);
		assert_ok!(Treasury::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Etp::free_balance(3), 10);

		assert_eq!(last_event(), RawEvent::TipClosed(h, 3, 10));

		assert_noop!(
			Treasury::close_tip(Origin::signed(100), h.into()),
			<Error<Test, _>>::UnknownTip
		);
	});
}

#[test]
fn retract_tip_works() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::report_awesome(
			Origin::signed(0),
			b"awesome.hyperspace".to_vec(),
			3
		));
		let h = tip_hash();
		assert_ok!(Treasury::tip(Origin::signed(10), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(
			Treasury::retract_tip(Origin::signed(10), h.clone()),
			<Error<Test, _>>::NotFinder
		);
		assert_ok!(Treasury::retract_tip(Origin::signed(0), h.clone()));
		System::set_block_number(2);
		assert_noop!(
			Treasury::close_tip(Origin::signed(0), h.into()),
			<Error<Test, _>>::UnknownTip
		);

		// with tip new
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::tip_new(
			Origin::signed(10),
			b"awesome.hyperspace".to_vec(),
			3,
			10
		));
		let h = tip_hash();
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10));
		assert_noop!(
			Treasury::retract_tip(Origin::signed(0), h.clone()),
			<Error<Test, _>>::NotFinder
		);
		assert_ok!(Treasury::retract_tip(Origin::signed(10), h.clone()));
		System::set_block_number(2);
		assert_noop!(
			Treasury::close_tip(Origin::signed(10), h.into()),
			<Error<Test, _>>::UnknownTip
		);
	});
}

#[test]
fn tip_median_calculation_works() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::tip_new(
			Origin::signed(10),
			b"awesome.hyperspace".to_vec(),
			3,
			0
		));
		let h = tip_hash();
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 1000000));
		System::set_block_number(2);
		assert_ok!(Treasury::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Etp::free_balance(3), 10);
	});
}

#[test]
fn tip_changing_works() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::tip_new(
			Origin::signed(10),
			b"awesome.hyperspace".to_vec(),
			3,
			10000
		));
		let h = tip_hash();
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 10000));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 10000));
		assert_ok!(Treasury::tip(Origin::signed(13), h.clone(), 0));
		assert_ok!(Treasury::tip(Origin::signed(14), h.clone(), 0));
		assert_ok!(Treasury::tip(Origin::signed(12), h.clone(), 1000));
		assert_ok!(Treasury::tip(Origin::signed(11), h.clone(), 100));
		assert_ok!(Treasury::tip(Origin::signed(10), h.clone(), 10));
		System::set_block_number(2);
		assert_ok!(Treasury::close_tip(Origin::signed(0), h.into()));
		assert_eq!(Etp::free_balance(3), 10);
	});
}

#[test]
fn minting_works() {
	new_test_ext().execute_with(|| {
		// Check that accumulate works when we have Some value in Dummy already.
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
	});
}

#[test]
fn spend_proposal_takes_min_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 1, 0, 3));
		assert_eq!(Etp::free_balance(0), 99);
		assert_eq!(Etp::reserved_balance(0), 1);
	});
}

#[test]
fn spend_proposal_takes_proportional_deposit() {
	new_test_ext().execute_with(|| {
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 0, 3));
		assert_eq!(Etp::free_balance(0), 95);
		assert_eq!(Etp::reserved_balance(0), 5);
	});
}

#[test]
fn spend_proposal_fails_when_proposer_poor() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Treasury::propose_spend(Origin::signed(2), 100, 0, 3),
			<Error<Test, _>>::InsufficientProposersBalance,
		);
	});
}

#[test]
fn accepted_spend_proposal_ignored_outside_spend_period() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);

		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 0, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));

		<Treasury as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Etp::free_balance(3), 0);
		assert_eq!(Treasury::pot::<Etp>(), 100);
	});
}

#[test]
fn unused_pot_should_diminish() {
	new_test_ext().execute_with(|| {
		let init_total_issuance = Etp::total_issuance();
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Etp::total_issuance(), init_total_issuance + 100);

		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Treasury::pot::<Etp>(), 50);
		assert_eq!(Etp::total_issuance(), init_total_issuance + 50);
	});
}

#[test]
fn rejected_spend_proposal_ignored_on_spend_period() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 0, 3));
		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Etp::free_balance(3), 0);
		assert_eq!(Treasury::pot::<Etp>(), 50);
	});
}

#[test]
fn reject_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 0, 3));
		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
		assert_noop!(
			Treasury::reject_proposal(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);
	});
}

#[test]
fn reject_non_existent_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Treasury::reject_proposal(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);
	});
}

#[test]
fn accept_non_existent_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Treasury::approve_proposal(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);
	});
}

#[test]
fn accept_already_rejected_spend_proposal_fails() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 100, 3));
		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));
		assert_noop!(
			Treasury::approve_proposal(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);
	});
}

#[test]
fn accepted_spend_proposal_enacted_on_spend_period() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 0, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));

		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Etp::free_balance(3), 100);
		assert_eq!(Treasury::pot::<Etp>(), 0);
	});
}

#[test]
fn pot_underflow_should_not_diminish() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 150, 0, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));

		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Treasury::pot::<Etp>(), 100); // Pot hasn't changed

		let _ = Etp::deposit_into_existing(&Treasury::account_id(), 100).unwrap();
		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Etp::free_balance(3), 150); // Fund has been spent
		assert_eq!(Treasury::pot::<Etp>(), 25); // Pot has finally changed
	});
}

// Treasury account doesn't get deleted if amount approved to spend is all its free balance.
// i.e. pot should not include existential deposit needed for account survival.
#[test]
fn treasury_account_doesnt_get_deleted() {
	new_test_ext().execute_with(|| {
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);

		let treasury_balance = Etp::free_balance(&Treasury::account_id());
		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			treasury_balance,
			0,
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));

		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Treasury::pot::<Etp>(), 100); // Pot hasn't changed

		assert_ok!(Treasury::propose_spend(
			Origin::signed(0),
			Treasury::pot::<Etp>(),
			0,
			3
		));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 1));

		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Treasury::pot::<Etp>(), 0); // Pot is emptied
		assert_eq!(Etp::free_balance(Treasury::account_id()), 1); // but the account is still there
	});
}

// In case treasury account is not existing then it works fine.
// This is useful for chain that will just update runtime.
#[test]
fn inexistent_account_works() {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	EtpConfig {
		balances: vec![(0, 100), (1, 99), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	DnaConfig {
		balances: vec![(0, 100), (1, 99), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// GenesisConfig::default().assimilate_storage::<Test>(&mut t).unwrap();
	// Treasury genesis config is not build thus treasury account does not exist
	let mut t: sp_io::TestExternalities = t.into();

	t.execute_with(|| {
		assert_eq!(Etp::free_balance(Treasury::account_id()), 0); // Account does not exist
		assert_eq!(Treasury::pot::<Etp>(), 0); // Pot is empty
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 99, 0, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0));
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 1, 0, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 1));
		<Treasury as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Treasury::pot::<Etp>(), 0); // Pot hasn't changed
		assert_eq!(Etp::free_balance(3), 0); // Balance of `3` hasn't changed

		Etp::make_free_balance_be(&Treasury::account_id(), 100);
		assert_eq!(Treasury::pot::<Etp>(), 99); // Pot now contains funds
		assert_eq!(Etp::free_balance(Treasury::account_id()), 100); // Account does exist
		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Treasury::pot::<Etp>(), 0); // Pot has changed
		assert_eq!(Etp::free_balance(3), 99); // Balance of `3` has changed
	});
}

#[test]
fn propose_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);

		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			10,
			b"1234567890".to_vec()
		));

		assert_eq!(last_event(), RawEvent::BountyProposed(0));

		let deposit: u64 = 85 + 5;
		assert_eq!(Etp::reserved_balance(0), deposit);
		assert_eq!(Etp::free_balance(0), 100 - deposit);

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 0,
				curator_deposit: 0,
				value: 10,
				bond: deposit,
				status: BountyStatus::Proposed,
			}
		);

		assert_eq!(
			Treasury::bounty_descriptions(0).unwrap(),
			b"1234567890".to_vec()
		);

		assert_eq!(Treasury::bounty_count(), 1);
	});
}

#[test]
fn propose_bounty_validation_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);

		assert_noop!(
			Treasury::propose_bounty(Origin::signed(1), 0, [0; 17_000].to_vec()),
			<Error<Test, _>>::ReasonTooBig
		);

		assert_noop!(
			Treasury::propose_bounty(Origin::signed(1), 10, b"12345678901234567890".to_vec()),
			<Error<Test, _>>::InsufficientProposersBalance
		);

		assert_noop!(
			Treasury::propose_bounty(Origin::signed(1), 0, b"12345678901234567890".to_vec()),
			<Error<Test, _>>::InvalidValue
		);
	});
}

#[test]
fn close_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_noop!(
			Treasury::close_bounty(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);

		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			10,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::close_bounty(Origin::root(), 0));

		let deposit: u64 = 80 + 5;

		assert_eq!(last_event(), RawEvent::BountyRejected(0, deposit));

		assert_eq!(Etp::reserved_balance(0), 0);
		assert_eq!(Etp::free_balance(0), 100 - deposit);

		assert_eq!(Treasury::bounties(0), None);
		assert!(!Bounties::<Test>::contains_key(0));
		assert_eq!(Treasury::bounty_descriptions(0), None);
	});
}

#[test]
fn approve_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_noop!(
			Treasury::approve_bounty(Origin::root(), 0),
			<Error<Test, _>>::InvalidIndex
		);

		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		let deposit: u64 = 80 + 5;

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 0,
				value: 50,
				curator_deposit: 0,
				bond: deposit,
				status: BountyStatus::Approved,
			}
		);
		assert_eq!(Treasury::bounty_approvals(), vec![0]);

		assert_noop!(
			Treasury::close_bounty(Origin::root(), 0),
			<Error<Test, _>>::UnexpectedStatus
		);

		// deposit not returned yet
		assert_eq!(Etp::reserved_balance(0), deposit);
		assert_eq!(Etp::free_balance(0), 100 - deposit);

		<Treasury as OnInitialize<u64>>::on_initialize(2);

		// return deposit
		assert_eq!(Etp::reserved_balance(0), 0);
		assert_eq!(Etp::free_balance(0), 100);

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 0,
				curator_deposit: 0,
				value: 50,
				bond: deposit,
				status: BountyStatus::Funded,
			}
		);
		assert_eq!(Treasury::pot::<Etp>(), 100 - 50 - 25); // burn 25
		assert_eq!(Etp::free_balance(Treasury::bounty_account_id(0)), 50);
	});
}

#[test]
fn assign_curator_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);

		assert_noop!(
			Treasury::propose_curator(Origin::root(), 0, 4, 4),
			<Error<Test, _>>::InvalidIndex
		);

		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_noop!(
			Treasury::propose_curator(Origin::root(), 0, 4, 50),
			<Error<Test, _>>::InvalidFee
		);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 4));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 4,
				curator_deposit: 0,
				value: 50,
				bond: 85,
				status: BountyStatus::CuratorProposed { curator: 4 },
			}
		);

		assert_noop!(
			Treasury::accept_curator(Origin::signed(1), 0),
			<Error<Test, _>>::RequireCurator
		);
		assert_noop!(
			Treasury::accept_curator(Origin::signed(4), 0),
			<hyperspace_balances::Error<Test, EtpInstance>>::InsufficientBalance
		);

		Etp::make_free_balance_be(&4, 10);

		assert_ok!(Treasury::accept_curator(Origin::signed(4), 0));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 4,
				curator_deposit: 2,
				value: 50,
				bond: 85,
				status: BountyStatus::Active {
					curator: 4,
					update_due: 22,
				},
			}
		);

		assert_eq!(Etp::free_balance(&4), 8);
		assert_eq!(Etp::reserved_balance(&4), 2);
	});
}

#[test]
fn unassign_curator_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 4));

		assert_noop!(Treasury::unassign_curator(Origin::signed(1), 0), BadOrigin);

		assert_ok!(Treasury::unassign_curator(Origin::signed(4), 0));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 4,
				curator_deposit: 0,
				value: 50,
				bond: 85,
				status: BountyStatus::Funded,
			}
		);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 4));

		Etp::make_free_balance_be(&4, 10);

		assert_ok!(Treasury::accept_curator(Origin::signed(4), 0));

		assert_ok!(Treasury::unassign_curator(Origin::root(), 0));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 4,
				curator_deposit: 0,
				value: 50,
				bond: 85,
				status: BountyStatus::Funded,
			}
		);

		assert_eq!(Etp::free_balance(&4), 8);
		assert_eq!(Etp::reserved_balance(&4), 0); // slashed 2
	});
}

#[test]
fn award_and_claim_bounty_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Etp::make_free_balance_be(&4, 10);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 4));
		assert_ok!(Treasury::accept_curator(Origin::signed(4), 0));

		assert_eq!(Etp::free_balance(4), 8); // inital 10 - 2 deposit

		assert_noop!(
			Treasury::award_bounty(Origin::signed(1), 0, 3),
			<Error<Test, _>>::RequireCurator
		);

		assert_ok!(Treasury::award_bounty(Origin::signed(4), 0, 3));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 4,
				curator_deposit: 2,
				value: 50,
				bond: 85,
				status: BountyStatus::PendingPayout {
					curator: 4,
					beneficiary: 3,
					unlock_at: 5
				},
			}
		);

		assert_noop!(
			Treasury::claim_bounty(Origin::signed(1), 0),
			<Error<Test, _>>::Premature
		);

		System::set_block_number(5);
		<Treasury as OnInitialize<u64>>::on_initialize(5);

		assert_ok!(Etp::transfer(
			Origin::signed(0),
			Treasury::bounty_account_id(0),
			10
		));

		assert_ok!(Treasury::claim_bounty(Origin::signed(1), 0));

		assert_eq!(last_event(), RawEvent::BountyClaimed(0, 56, 3));

		assert_eq!(Etp::free_balance(4), 14); // initial 10 + fee 4
		assert_eq!(Etp::free_balance(3), 56);
		assert_eq!(Etp::free_balance(Treasury::bounty_account_id(0)), 0);

		assert_eq!(Treasury::bounties(0), None);
		assert_eq!(Treasury::bounty_descriptions(0), None);
	});
}

#[test]
fn claim_handles_high_fee() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Etp::make_free_balance_be(&4, 30);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 49));
		assert_ok!(Treasury::accept_curator(Origin::signed(4), 0));

		assert_ok!(Treasury::award_bounty(Origin::signed(4), 0, 3));

		System::set_block_number(5);
		<Treasury as OnInitialize<u64>>::on_initialize(5);

		// make fee > balance
		let _ = Etp::slash(&Treasury::bounty_account_id(0), 10);

		assert_ok!(Treasury::claim_bounty(Origin::signed(1), 0));

		assert_eq!(last_event(), RawEvent::BountyClaimed(0, 0, 3));

		assert_eq!(Etp::free_balance(4), 70); // 30 + 50 - 10
		assert_eq!(Etp::free_balance(3), 0);
		assert_eq!(Etp::free_balance(Treasury::bounty_account_id(0)), 0);

		assert_eq!(Treasury::bounties(0), None);
		assert_eq!(Treasury::bounty_descriptions(0), None);
	});
}

#[test]
fn cancel_and_refund() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Etp::transfer(
			Origin::signed(0),
			Treasury::bounty_account_id(0),
			10
		));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 0,
				curator_deposit: 0,
				value: 50,
				bond: 85,
				status: BountyStatus::Funded,
			}
		);

		assert_eq!(Etp::free_balance(Treasury::bounty_account_id(0)), 60);

		assert_noop!(Treasury::close_bounty(Origin::signed(0), 0), BadOrigin);

		assert_ok!(Treasury::close_bounty(Origin::root(), 0));

		assert_eq!(Treasury::pot::<Etp>(), 85); // - 25 + 10
	});
}

#[test]
fn award_and_cancel() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 0, 10));
		assert_ok!(Treasury::accept_curator(Origin::signed(0), 0));

		assert_eq!(Etp::free_balance(0), 95);
		assert_eq!(Etp::reserved_balance(0), 5);

		assert_ok!(Treasury::award_bounty(Origin::signed(0), 0, 3));

		// Cannot close bounty directly when payout is happening...
		assert_noop!(
			Treasury::close_bounty(Origin::root(), 0),
			<Error<Test, _>>::PendingPayout
		);

		// Instead unassign the curator to slash them and then close.
		assert_ok!(Treasury::unassign_curator(Origin::root(), 0));
		assert_ok!(Treasury::close_bounty(Origin::root(), 0));

		assert_eq!(last_event(), RawEvent::BountyCanceled(0));

		assert_eq!(Etp::free_balance(Treasury::bounty_account_id(0)), 0);
		// Slashed.
		assert_eq!(Etp::free_balance(0), 95);
		assert_eq!(Etp::reserved_balance(0), 0);

		assert_eq!(Treasury::bounties(0), None);
		assert_eq!(Treasury::bounty_descriptions(0), None);
	});
}

#[test]
fn expire_and_unassign() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 1, 10));
		assert_ok!(Treasury::accept_curator(Origin::signed(1), 0));

		assert_eq!(Etp::free_balance(1), 93);
		assert_eq!(Etp::reserved_balance(1), 5);

		System::set_block_number(22);
		<Treasury as OnInitialize<u64>>::on_initialize(22);

		assert_noop!(
			Treasury::unassign_curator(Origin::signed(0), 0),
			<Error<Test, _>>::Premature
		);

		System::set_block_number(23);
		<Treasury as OnInitialize<u64>>::on_initialize(23);

		assert_ok!(Treasury::unassign_curator(Origin::signed(0), 0));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 10,
				curator_deposit: 0,
				value: 50,
				bond: 85,
				status: BountyStatus::Funded,
			}
		);

		assert_eq!(Etp::free_balance(1), 93);
		assert_eq!(Etp::reserved_balance(1), 0); // slashed
	});
}

#[test]
fn extend_expiry() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Etp::make_free_balance_be(&4, 10);
		assert_ok!(Treasury::propose_bounty(
			Origin::signed(0),
			50,
			b"12345".to_vec()
		));

		assert_ok!(Treasury::approve_bounty(Origin::root(), 0));

		assert_noop!(
			Treasury::extend_bounty_expiry(Origin::signed(1), 0, Vec::new()),
			<Error<Test, _>>::UnexpectedStatus
		);

		System::set_block_number(2);
		<Treasury as OnInitialize<u64>>::on_initialize(2);

		assert_ok!(Treasury::propose_curator(Origin::root(), 0, 4, 10));
		assert_ok!(Treasury::accept_curator(Origin::signed(4), 0));

		assert_eq!(Etp::free_balance(4), 5);
		assert_eq!(Etp::reserved_balance(4), 5);

		System::set_block_number(10);
		<Treasury as OnInitialize<u64>>::on_initialize(10);

		assert_noop!(
			Treasury::extend_bounty_expiry(Origin::signed(0), 0, Vec::new()),
			<Error<Test, _>>::RequireCurator
		);
		assert_ok!(Treasury::extend_bounty_expiry(
			Origin::signed(4),
			0,
			Vec::new()
		));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 10,
				curator_deposit: 5,
				value: 50,
				bond: 85,
				status: BountyStatus::Active {
					curator: 4,
					update_due: 30
				},
			}
		);

		assert_ok!(Treasury::extend_bounty_expiry(
			Origin::signed(4),
			0,
			Vec::new()
		));

		assert_eq!(
			Treasury::bounties(0).unwrap(),
			Bounty {
				proposer: 0,
				fee: 10,
				curator_deposit: 5,
				value: 50,
				bond: 85,
				status: BountyStatus::Active {
					curator: 4,
					update_due: 30
				}, // still the same
			}
		);

		System::set_block_number(25);
		<Treasury as OnInitialize<u64>>::on_initialize(25);

		assert_noop!(
			Treasury::unassign_curator(Origin::signed(0), 0),
			<Error<Test, _>>::Premature
		);
		assert_ok!(Treasury::unassign_curator(Origin::signed(4), 0));

		assert_eq!(Etp::free_balance(4), 10); // not slashed
		assert_eq!(Etp::reserved_balance(4), 0);
	});
}

/// # Logic Tests.
///
/// + Proposal: A suggestion to allocate funds from the pot to a beneficiary.
/// + Beneficiary: An account who will receive the funds from a proposal iff the proposal is approved.
/// + Deposit: Funds that a proposer must lock when making a proposal.
/// The deposit will be returned or slashed if the proposal is approved or rejected respectively.
/// + Pot: Unspent funds accumulated by the treasury module.
#[test]
fn approve_proposal_no_keep_burning() {
	new_test_ext().execute_with(|| {
		// backtrace init configs.
		assert_eq!(Etp::free_balance(&0), 100);
		assert_eq!(Dna::free_balance(&0), 100);
		assert_eq!(Etp::free_balance(&1), 98);
		assert_eq!(Dna::free_balance(&1), 98);
		assert_eq!(Etp::free_balance(&2), 1);
		assert_eq!(Dna::free_balance(&2), 1);
		assert_eq!(Etp::free_balance(&3), 0);
		assert_eq!(Dna::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Etp>(), 0);
		assert_eq!(Treasury::pot::<Dna>(), 0);

		// Ensure an account's free balance equals some value; this will create the account if needed.
		// Returns a signed imbalance and status to indicate if the account was successfully updated
		// or update has led to killing of the account.
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Dna::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
		assert_eq!(Treasury::pot::<Dna>(), 100);

		// Put forward a suggestion for spending, burn treasury balances to AccontID-3
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 100, 3));
		assert_ok!(Treasury::approve_proposal(Origin::root(), 0)); // Accept proposal

		// @0-1: Check balances after `propose_spend`
		<Treasury as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Etp::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Dna::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 100); // No changes
		assert_eq!(Treasury::pot::<Dna>(), 100); // No changes

		// @2: On the first spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(2); // SpendPeriod: u64 = 2;
		assert_eq!(Etp::free_balance(&0), 100); // ProposalBond: Permill::from_percent(5); **return bond**
		assert_eq!(Dna::free_balance(&0), 100); // ProposalBond: Permill::from_percent(5); **return bond**
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 100); // No changes
		assert_eq!(Dna::free_balance(&3), 100); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 0); // Burn: Permill::from_percent(50); **Burn 100 if approve**
		assert_eq!(Treasury::pot::<Dna>(), 0); // Burn: Permill::from_percent(50); **Burn 100 if approve**

		// @3: Check balances on the perid after spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(3);
		assert_eq!(Etp::free_balance(&0), 100); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 100); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 100); // No changes
		assert_eq!(Dna::free_balance(&3), 100); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 0); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 0); // No changes from last perid

		// @4: The second spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Etp::free_balance(&0), 100); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 100); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 100); // No changes
		assert_eq!(Dna::free_balance(&3), 100); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 0); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 0); // No changes from last perid
	});
}

#[test]
fn reject_proposal_keep_burning() {
	new_test_ext().execute_with(|| {
		// backtrace init configs.
		assert_eq!(Etp::free_balance(&0), 100);
		assert_eq!(Dna::free_balance(&0), 100);
		assert_eq!(Etp::free_balance(&1), 98);
		assert_eq!(Dna::free_balance(&1), 98);
		assert_eq!(Etp::free_balance(&2), 1);
		assert_eq!(Dna::free_balance(&2), 1);
		assert_eq!(Etp::free_balance(&3), 0);
		assert_eq!(Dna::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Etp>(), 0);
		assert_eq!(Treasury::pot::<Dna>(), 0);

		// Ensure an account's free balance equals some value; this will create the account if needed.
		// Returns a signed imbalance and status to indicate if the account was successfully updated
		// or update has led to killing of the account.
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Dna::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
		assert_eq!(Treasury::pot::<Dna>(), 100);

		// Put forward a suggestion for spending, burn treasury balances to AccontID-3
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 100, 3));
		assert_ok!(Treasury::reject_proposal(Origin::root(), 0));

		// @0-1: Check balances after `propose_spend`
		<Treasury as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Etp::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Dna::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 100); // No changes
		assert_eq!(Treasury::pot::<Dna>(), 100); // No changes

		// @2: On the first spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(2); // SpendPeriod: u64 = 2;
		assert_eq!(Etp::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Dna::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 50); // Burn: Permill::from_percent(50); **The Burned Etp just burned?**
		assert_eq!(Treasury::pot::<Dna>(), 50); // Burn: Permill::from_percent(50); **The Burned Etp just burned?**

		// @3: Check balances on the perid after spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(3);
		assert_eq!(Etp::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 50); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 50); // No changes from last perid

		// @4: The second spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Etp::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 25); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 25); // No changes from last perid
	});
}

#[test]
fn no_accept_no_reject_keep_burning() {
	new_test_ext().execute_with(|| {
		// backtrace init configs.
		assert_eq!(Etp::free_balance(&0), 100);
		assert_eq!(Dna::free_balance(&0), 100);
		assert_eq!(Etp::free_balance(&1), 98);
		assert_eq!(Dna::free_balance(&1), 98);
		assert_eq!(Etp::free_balance(&2), 1);
		assert_eq!(Dna::free_balance(&2), 1);
		assert_eq!(Etp::free_balance(&3), 0);
		assert_eq!(Dna::free_balance(&3), 0);
		assert_eq!(Treasury::pot::<Etp>(), 0);
		assert_eq!(Treasury::pot::<Dna>(), 0);

		// Ensure an account's free balance equals some value; this will create the account if needed.
		// Returns a signed imbalance and status to indicate if the account was successfully updated
		// or update has led to killing of the account.
		Etp::make_free_balance_be(&Treasury::account_id(), 101);
		Dna::make_free_balance_be(&Treasury::account_id(), 101);
		assert_eq!(Treasury::pot::<Etp>(), 100);
		assert_eq!(Treasury::pot::<Dna>(), 100);

		// Put forward a suggestion for spending, burn treasury balances to AccontID-3
		assert_ok!(Treasury::propose_spend(Origin::signed(0), 100, 100, 3));

		// @0-1: Check balances after `propose_spend`
		<Treasury as OnInitialize<u64>>::on_initialize(1);
		assert_eq!(Etp::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Dna::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 100); // No changes
		assert_eq!(Treasury::pot::<Dna>(), 100); // No changes

		// @2: On the first spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(2); // SpendPeriod: u64 = 2;
		assert_eq!(Etp::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Dna::free_balance(&0), 95); // ProposalBond: Permill::from_percent(5);
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 50); // Burn: Permill::from_percent(50); **The Burned Etp just burned?**
		assert_eq!(Treasury::pot::<Dna>(), 50); // Burn: Permill::from_percent(50); **The Burned Etp just burned?**

		// @3: Check balances on the perid after spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(3);
		assert_eq!(Etp::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 50); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 50); // No changes from last perid

		// @4: The second spend perid
		<Treasury as OnInitialize<u64>>::on_initialize(4);
		assert_eq!(Etp::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Dna::free_balance(&0), 95); // No changes from last perid
		assert_eq!(Etp::free_balance(&1), 98); // No changes
		assert_eq!(Dna::free_balance(&1), 98); // No changes
		assert_eq!(Etp::free_balance(&2), 1); // No changes
		assert_eq!(Dna::free_balance(&2), 1); // No changes
		assert_eq!(Etp::free_balance(&3), 0); // No changes
		assert_eq!(Dna::free_balance(&3), 0); // No changes
		assert_eq!(Treasury::pot::<Etp>(), 25); // No changes from last perid
		assert_eq!(Treasury::pot::<Dna>(), 25); // No changes from last perid
	});
}

#[test]
fn genesis_funding_works() {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let initial_funding = 100;
	EtpConfig {
		// Total issuance will be 200 with treasury account initialized with 100.
		balances: vec![(0, 100), (Treasury::account_id(), initial_funding)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	DnaConfig {
		// Total issuance will be 200 with treasury account initialized with 100.
		balances: vec![(0, 100), (Treasury::account_id(), initial_funding)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisConfig::default()
		.assimilate_storage::<Test, _>(&mut t)
		.unwrap();
	let mut t: sp_io::TestExternalities = t.into();

	t.execute_with(|| {
		assert_eq!(Etp::free_balance(Treasury::account_id()), initial_funding);
		assert_eq!(
			Treasury::pot::<Etp>(),
			initial_funding - Etp::minimum_balance()
		);
		assert_eq!(Dna::free_balance(Treasury::account_id()), initial_funding);
		assert_eq!(
			Treasury::pot::<Dna>(),
			initial_funding - Dna::minimum_balance()
		);
	});
}
