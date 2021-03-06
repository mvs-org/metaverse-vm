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
pub use frame_support::traits::{LockIdentifier, VestingSchedule, WithdrawReasons};

// --- core ---
use core::fmt::Debug;
// --- crates ---
use codec::FullCodec;
use impl_trait_for_tuples::impl_for_tuples;
// --- substrate ---
use frame_support::traits::{Currency, Get, TryDrop};
use sp_runtime::{DispatchError, DispatchResult};
use sp_std::prelude::*;
// --- hyperspace ---
use crate::balance::{
	lock::{LockFor, LockReasons},
	FrozenBalance,
};
use ethereum_primitives::receipt::EthereumTransactionIndex;

pub trait BalanceInfo<Balance, Module> {
	fn free(&self) -> Balance;
	fn set_free(&mut self, new_free: Balance);

	fn reserved(&self) -> Balance;
	fn set_reserved(&mut self, new_reserved: Balance);

	/// The total balance in this account including any that is reserved and ignoring any frozen.
	fn total(&self) -> Balance;

	/// How much this account's balance can be reduced for the given `reasons`.
	fn usable(&self, reasons: LockReasons, frozen_balance: FrozenBalance<Balance>) -> Balance;
}

/// A currency whose accounts can have liquidity restrictions.
pub trait LockableCurrency<AccountId>: Currency<AccountId> {
	/// The quantity used to denote time; usually just a `BlockNumber`.
	type Moment;

	/// The maximum number of locks a user should have on their account.
	type MaxLocks: Get<u32>;

	/// Create a new balance lock on account `who`.
	///
	/// If the new lock is valid (i.e. not already expired), it will push the struct to
	/// the `Locks` vec in storage. Note that you can lock more funds than a user has.
	///
	/// If the lock `id` already exists, this will update it.
	fn set_lock(
		id: LockIdentifier,
		who: &AccountId,
		lock_for: LockFor<Self::Balance, Self::Moment>,
		reasons: WithdrawReasons,
	);

	/// Changes a balance lock (selected by `id`) so that it becomes less liquid in all
	/// parameters or creates a new one if it does not exist.
	///
	/// Calling `extend_lock` on an existing lock `id` differs from `set_lock` in that it
	/// applies the most severe constraints of the two, while `set_lock` replaces the lock
	/// with the new parameters. As in, `extend_lock` will set:
	/// - maximum `amount`
	/// - bitwise mask of all `reasons`
	fn extend_lock(
		id: LockIdentifier,
		who: &AccountId,
		amount: Self::Balance,
		reasons: WithdrawReasons,
	) -> DispatchResult;

	/// Remove an existing lock.
	fn remove_lock(id: LockIdentifier, who: &AccountId);

	/// Get the balance of an account that can be used for transfers, reservations, or any other
	/// non-locking, non-transaction-fee activity. Will be at most `free_balance`.
	fn usable_balance(who: &AccountId) -> Self::Balance;

	/// Get the balance of an account that can be used for paying transaction fees (not tipping,
	/// or any other kind of fees, though). Will be at most `free_balance`.
	fn usable_balance_for_fees(who: &AccountId) -> Self::Balance;
}

pub trait DustCollector<AccountId> {
	fn is_dust(who: &AccountId) -> bool;

	fn collect(who: &AccountId);
}
#[impl_for_tuples(30)]
impl<AccountId> DustCollector<AccountId> for Currencies {
	fn is_dust(who: &AccountId) -> bool {
		for_tuples!( #(
			if !Currencies::is_dust(who) {
				return false;
			}
		)* );

		true
	}

	fn collect(who: &AccountId) {
		for_tuples!( #( Currencies::collect(who); )* );
	}
}

/// Callback on ethereum-backing module
pub trait OnDepositRedeem<AccountId, Balance> {
	fn on_deposit_redeem(
		backing: &AccountId,
		stash: &AccountId,
		amount: Balance,
		start_at: u64,
		months: u8,
	) -> DispatchResult;
}

// FIXME: Ugly hack due to https://github.com/rust-lang/rust/issues/31844#issuecomment-557918823
/// Handler for when some currency "account" decreased in balance for
/// some reason.
///
/// The only reason at present for an increase would be for validator rewards, but
/// there may be other reasons in the future or for other chains.
///
/// Reasons for decreases include:
///
/// - Someone got slashed.
/// - Someone paid for a transaction to be included.
pub trait OnUnbalancedDna<Imbalance: TryDrop> {
	/// Handler for some imbalances. The different imbalances might have different origins or
	/// meanings, dependent on the context. Will default to simply calling on_unbalanced for all
	/// of them. Infallible.
	fn on_unbalanceds<B>(amounts: impl Iterator<Item = Imbalance>)
	where
		Imbalance: frame_support::traits::Imbalance<B>,
	{
		Self::on_unbalanced(amounts.fold(Imbalance::zero(), |i, x| x.merge(i)))
	}

	/// Handler for some imbalance. Infallible.
	fn on_unbalanced(amount: Imbalance) {
		amount
			.try_drop()
			.unwrap_or_else(Self::on_nonzero_unbalanced)
	}

	/// Actually handle a non-zero imbalance. You probably want to implement this rather than
	/// `on_unbalanced`.
	fn on_nonzero_unbalanced(amount: Imbalance) {
		drop(amount);
	}
}
impl<Imbalance: TryDrop> OnUnbalancedDna<Imbalance> for () {}

pub trait EthereumReceipt<AccountId, Balance> {
	type EthereumReceiptProofThing: Clone + Debug + PartialEq + FullCodec;

	fn account_id() -> AccountId;

	fn receipt_verify_fee() -> Balance;

	fn verify_receipt(
		proof: &Self::EthereumReceiptProofThing,
	) -> Result<ethereum_primitives::receipt::EthereumReceipt, DispatchError>;

	fn gen_receipt_index(proof: &Self::EthereumReceiptProofThing) -> EthereumTransactionIndex;
}
