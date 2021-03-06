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

//! # Balances Module
//!
//! The Balances module provides functionality for handling accounts and balances.
//!
//! - [`balances::Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! The Balances module provides functions for:
//!
//! - Getting and setting free balances.
//! - Retrieving total, reserved and unreserved balances.
//! - Repatriating a reserved balance to a beneficiary account that exists.
//! - Transferring a balance between accounts (when not reserved).
//! - Slashing an account balance.
//! - Account creation and removal.
//! - Managing total issuance.
//! - Setting and managing locks.
//!
//! ### Terminology
//!
//! - **Existential Deposit:** The minimum balance required to create or keep an account open. This prevents
//! "dust accounts" from filling storage. When the free plus the reserved balance (i.e. the total balance)
//!   fall below this, then the account is said to be dead; and it loses its functionality as well as any
//!   prior history and all information on it is removed from the chain's state.
//!   No account should ever have a total balance that is strictly between 0 and the existential
//!   deposit (exclusive). If this ever happens, it indicates either a bug in this module or an
//!   erroneous raw mutation of storage.
//!
//! - **Total Issuance:** The total number of units in existence in a system.
//!
//! - **Reaping an account:** The act of removing an account by resetting its nonce. Happens after its
//! total balance has become zero (or, strictly speaking, less than the Existential Deposit).
//!
//! - **Free Balance:** The portion of a balance that is not reserved. The free balance is the only
//!   balance that matters for most operations.
//!
//! - **Reserved Balance:** Reserved balance still belongs to the account holder, but is suspended.
//!   Reserved balance can still be slashed, but only after all the free balance has been slashed.
//!
//! - **Imbalance:** A condition when some funds were credited or debited without equal and opposite accounting
//! (i.e. a difference between total issuance and account balances). Functions that result in an imbalance will
//! return an object of the `Imbalance` trait that can be managed within your runtime logic. (If an imbalance is
//! simply dropped, it should automatically maintain any book-keeping such as total issuance.)
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance until a specified block number. Multiple
//! locks always operate over the same funds, so they "overlay" rather than "stack".
//!
//! ### Implementations
//!
//! The Balances module provides implementations for the following traits. If these traits provide the functionality
//! that you need, then you can avoid coupling with the Balances module.
//!
//! - [`Currency`](../frame_support/traits/trait.Currency.html): Functions for dealing with a
//! fungible assets system.
//! - [`ReservableCurrency`](../frame_support/traits/trait.ReservableCurrency.html):
//! Functions for dealing with assets that can be reserved from an account.
//! - [`LockableCurrency`](../frame_support/traits/trait.LockableCurrency.html): Functions for
//! dealing with accounts that allow liquidity restrictions.
//! - [`Imbalance`](../frame_support/traits/trait.Imbalance.html): Functions for handling
//! imbalances between total issuance in the system and account balances. Must be used when a function
//! creates new funds (e.g. a reward) or destroys some funds (e.g. a system fee).
//! - [`IsDeadAccount`](../frame_support/traits/trait.IsDeadAccount.html): Determiner to say whether a
//! given account is unused.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some liquid free balance to another account.
//! - `set_balance` - Set the balances of a given account. The origin of this call must be root.
//!
//! ## Usage
//!
//! The following examples show how to use the Balances module in your custom module.
//!
//! ### Examples from the FRAME
//!
//! The Contract module uses the `Currency` trait to handle gas payment, and its types inherit from `Currency`:
//!
//! ```
//! use frame_support::traits::Currency;
//! # pub trait Config: frame_system::Config {
//! # 	type Currency: Currency<Self::AccountId>;
//! # }
//!
//! pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
//! pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
//!
//! # fn main() {}
//! ```
//!
//! The Staking module uses the `LockableCurrency` trait to lock a stash account's funds:
//!
//! ```
//! use frame_support::traits::{WithdrawReasons, LockableCurrency};
//! use sp_runtime::traits::Bounded;
//! pub trait Config: frame_system::Config {
//! 	type Currency: LockableCurrency<Self::AccountId, Moment=Self::BlockNumber>;
//! }
//! # struct StakingLedger<T: Config> {
//! # 	stash: <T as frame_system::Config>::AccountId,
//! # 	total: <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance,
//! # 	phantom: std::marker::PhantomData<T>,
//! # }
//! # const STAKING_ID: [u8; 8] = *b"staking ";
//!
//! fn update_ledger<T: Config>(
//! 	controller: &T::AccountId,
//! 	ledger: &StakingLedger<T>
//! ) {
//! 	T::Currency::set_lock(
//! 		STAKING_ID,
//! 		&ledger.stash,
//! 		ledger.total,
//! 		WithdrawReasons::all()
//! 	);
//! 	// <Ledger<T>>::insert(controller, ledger); // Commented out as we don't have access to Staking's storage here.
//! }
//! # fn main() {}
//! ```
//!
//! ## Genesis config
//!
//! The Balances module depends on the [`GenesisConfig`](./struct.GenesisConfig.html).
//!
//! ## Assumptions
//!
//! * Total issued balanced of all accounts should be less than `Config::Balance::max_value()`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
mod tests;
#[cfg(test)]
mod tests_local;

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

// --- hyperspace ---
pub use imbalances::{NegativeImbalance, PositiveImbalance};

// --- crates ---
use codec::{Codec, EncodeLike};
// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{
		BalanceStatus as Status, Currency, ExistenceRequirement, ExistenceRequirement::AllowDeath,
		ExistenceRequirement::KeepAlive, Get, Imbalance, OnUnbalanced, ReservableCurrency,
		SignedImbalance, StoredMap, TryDrop,
	},
	Parameter, StorageValue,
};
use frame_system::{ensure_root, ensure_signed};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, Bounded, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member,
		Saturating, StaticLookup, StoredMapError, Zero,
	},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{borrow::Borrow, cmp, fmt::Debug, mem, prelude::*};
// --- hyperspace ---
use hyperspace_balances_rpc_runtime_api::RuntimeDispatchInfo;
use hyperspace_support::{
	balance::{lock::*, *},
	impl_rpc,
	traits::BalanceInfo,
};

pub trait Subtrait<I: Instance = DefaultInstance>: frame_system::Config {
	/// The balance of an account.
	type Balance: Parameter
		+ Member
		+ AtLeast32BitUnsigned
		+ Codec
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug;

	/// The minimum amount required to keep an account open.
	type ExistentialDeposit: Get<Self::Balance>;

	/// The means of storing the balances of an account.
	type AccountStore: StoredMap<Self::AccountId, Self::BalanceInfo>;

	type BalanceInfo: BalanceInfo<Self::Balance, I>
		+ Into<<Self as frame_system::Config>::AccountData>
		+ Member
		+ Codec
		+ Clone
		+ Default
		+ EncodeLike;

	/// The maximum number of locks that should exist on an account.
	/// Not strictly enforced, but used for weight estimation.
	type MaxLocks: Get<u32>;

	/// Weight information for the extrinsics in this pallet.
	type WeightInfo: WeightInfo;

	// A handle to check if other curencies drop below existential deposit
	type OtherCurrencies: DustCollector<Self::AccountId>;
}

pub trait Config<I: Instance = DefaultInstance>: frame_system::Config {
	/// The balance of an account.
	type Balance: Parameter
		+ Member
		+ AtLeast32BitUnsigned
		+ Codec
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug;

	/// Handler for the unbalanced reduction when removing a dust account.
	type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;

	/// The overarching event type.
	type Event: From<Event<Self, I>> + Into<<Self as frame_system::Config>::Event>;

	/// The minimum amount required to keep an account open.
	type ExistentialDeposit: Get<Self::Balance>;

	type BalanceInfo: BalanceInfo<Self::Balance, I>
		+ Into<<Self as frame_system::Config>::AccountData>
		+ Member
		+ Codec
		+ Clone
		+ Default
		+ EncodeLike;

	/// The means of storing the balances of an account.
	type AccountStore: StoredMap<Self::AccountId, Self::BalanceInfo>;

	/// The maximum number of locks that should exist on an account.
	/// Not strictly enforced, but used for weight estimation.
	type MaxLocks: Get<u32>;

	// A handle to check if other curencies drop below existential deposit
	type OtherCurrencies: DustCollector<Self::AccountId>;

	/// Weight information for the extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

impl<T: Config<I>, I: Instance> Subtrait<I> for T {
	type Balance = T::Balance;
	type ExistentialDeposit = T::ExistentialDeposit;
	type AccountStore = T::AccountStore;
	type BalanceInfo = T::BalanceInfo;
	type MaxLocks = T::MaxLocks;
	type OtherCurrencies = T::OtherCurrencies;
	type WeightInfo = <T as Config<I>>::WeightInfo;
}

decl_event!(
	pub enum Event<T, I: Instance = DefaultInstance>
	where
		<T as frame_system::Config>::AccountId,
		<T as Config<I>>::Balance,
	{
		/// An account was created with some free balance. [account, free_balance]
		Endowed(AccountId, Balance),
		/// An account was removed whose balance was non-zero but below ExistentialDeposit,
		/// resulting in an outright loss. [account, balance]
		DustLost(AccountId, Balance),
		/// Transfer succeeded. [from, to, value]
		Transfer(AccountId, AccountId, Balance),
		/// A balance was set by root. [who, free, reserved]
		BalanceSet(AccountId, Balance, Balance),
		/// Some amount was deposited (e.g. for transaction fees). [who, deposit]
		Deposit(AccountId, Balance),
		/// Some balance was reserved (moved from free to reserved). [who, value]
		Reserved(AccountId, Balance),
		/// Some balance was unreserved (moved from reserved to free). [who, value]
		Unreserved(AccountId, Balance),
		/// Some balance was moved from the reserve of the first account to the second account.
		/// Final argument indicates the destination balance type.
		/// [from, to, balance, destination_status]
		ReserveRepatriated(AccountId, AccountId, Balance, Status),
	}
);

decl_error! {
	pub enum Error for Module<T: Config<I>, I: Instance> {
		/// Vesting balance too high to send value
		VestingBalance,
		/// Account liquidity restrictions prevent withdrawal
		LiquidityRestrictions,
		/// Got an overflow after adding
		Overflow,
		/// Balance too low to send value
		InsufficientBalance,
		/// Value too low to create account due to existential deposit
		ExistentialDeposit,
		/// Transfer/payment would kill account
		KeepAlive,
		/// A vesting schedule already exists for this account
		ExistingVestingSchedule,
		/// Beneficiary account must pre-exist
		DeadAccount,
		/// Lock - POISONED
		LockP,
	}
}

decl_storage! {
	trait Store for Module<T: Config<I>, I: Instance = DefaultInstance> as HyperspaceBalances {
		/// The total units issued in the system.
		pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T, I>| {
			config
				.balances
				.iter()
				.fold(Zero::zero(), |acc: T::Balance, &(_, n)| acc + n)
		}): T::Balance;

		/// The balance of an account.
		///
		/// NOTE: This is only used in the case that this module is used to store balances.
		pub Account: map hasher(blake2_128_concat) T::AccountId => T::BalanceInfo;

		/// Any liquidity locks on some account balances.
		/// NOTE: Should only be accessed when setting, changing and freeing a lock.
		pub Locks
			get(fn locks)
			: map hasher(blake2_128_concat) T::AccountId
			=> Vec<BalanceLock<T::Balance, T::BlockNumber>>;
	}
	add_extra_genesis {
		config(balances): Vec<(T::AccountId, T::Balance)>;
		// ^^ begin, length, amount liquid at genesis
		build(|config: &GenesisConfig<T, I>| {
			for (_, balance) in &config.balances {
				assert!(
					*balance >= <T as Config<I>>::ExistentialDeposit::get(),
					"the balance of any account should always be at least the existential deposit.",
				)
			}

			// ensure no duplicates exist.
			let endowed_accounts = config.balances
				.iter()
				.map(|(x, _)| x)
				.cloned()
				.collect::<std::collections::BTreeSet<_>>();

			assert!(
				endowed_accounts.len() == config.balances.len(),
				"duplicate balances in genesis."
			);

			for &(ref who, free) in config.balances.iter() {
				let mut account_data = T::AccountStore::get(who);
				account_data.set_free(free);

				assert!(T::AccountStore::insert(who, account_data).is_ok());
			}
		});
	}
}

decl_module! {
	pub struct Module<T: Config<I>, I: Instance = DefaultInstance> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T, I>;

		/// The minimum amount required to keep an account open.
		const ExistentialDeposit: T::Balance = T::ExistentialDeposit::get();

		fn deposit_event() = default;

		/// Transfer some liquid free balance to another account.
		///
		/// `transfer` will set the `FreeBalance` of the sender and receiver.
		/// It will decrease the total issuance of the system by the `TransferFee`.
		/// If the sender's account is below the existential deposit as a result
		/// of the transfer, the account will be reaped.
		///
		/// The dispatch origin for this call must be `Signed` by the transactor.
		///
		/// # <weight>
		/// - Dependent on arguments but not critical, given proper implementations for
		///   input config types. See related functions below.
		/// - It contains a limited number of reads and writes internally and no complex computation.
		///
		/// Related functions:
		///
		///   - `ensure_can_withdraw` is always called internally but has a bounded complexity.
		///   - Transferring balances to accounts that did not exist before will cause
		///      `T::OnNewAccount::on_new_account` to be called.
		///   - Removing enough funds from an account will trigger `T::DustRemoval::on_unbalanced`.
		///   - `transfer_keep_alive` works the same way as `transfer`, but has an additional
		///     check that the transfer will not kill the origin account.
		///
		/// # </weight>
		#[weight = T::DbWeight::get().reads_writes(1, 1) + 200_000_000]
		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&transactor, &dest, value, ExistenceRequirement::AllowDeath)?;
		}

		/// Set the balances of a given account.
		///
		/// This will alter `FreeBalance` and `ReservedBalance` in storage. it will
		/// also decrease the total issuance of the system (`TotalIssuance`).
		/// If the new free or reserved balance is below the existential deposit,
		/// it will reset the account nonce (`frame_system::AccountNonce`).
		///
		/// The dispatch origin for this call is `root`.
		///
		/// # <weight>
		/// - Independent of the arguments.
		/// - Contains a limited number of reads and writes.
		/// # </weight>
		#[weight = T::DbWeight::get().reads_writes(1, 1) + 100_000_000]
		fn set_balance(
			origin,
			who: <T::Lookup as StaticLookup>::Source,
			#[compact] new_free: T::Balance,
			#[compact] new_reserved: T::Balance
		) {
			ensure_root(origin)?;
			let who = T::Lookup::lookup(who)?;
			let existential_deposit = T::ExistentialDeposit::get();

			let wipeout = {
				let new_total = new_free + new_reserved;

				new_total < existential_deposit && T::OtherCurrencies::is_dust(&who)
			};
			let new_free = if wipeout { Zero::zero() } else { new_free };
			let new_reserved = if wipeout { Zero::zero() } else { new_reserved };

			let (free, reserved) = Self::mutate_account(&who, |account| {
				if new_free > account.free() {
					mem::drop(PositiveImbalance::<T, I>::new(new_free - account.free()));
				} else if new_free < account.free() {
					mem::drop(NegativeImbalance::<T, I>::new(account.free() - new_free));
				}

				if new_reserved > account.reserved() {
					mem::drop(PositiveImbalance::<T, I>::new(new_reserved - account.reserved()));
				} else if new_reserved < account.reserved() {
					mem::drop(NegativeImbalance::<T, I>::new(account.reserved() - new_reserved));
				}

				account.set_free(new_free);
				account.set_reserved(new_reserved);

				(account.free(), account.reserved())
			})?;
			Self::deposit_event(RawEvent::BalanceSet(who, free, reserved));
		}

		/// Exactly as `transfer`, except the origin must be root and the source account may be
		/// specified.
		/// # <weight>
		/// - Same as transfer, but additional read and write because the source account is
		///   not assumed to be in the overlay.
		/// # </weight>
		#[weight = T::DbWeight::get().reads_writes(2, 2) + 200_000_000]
		pub fn force_transfer(
			origin,
			source: <T::Lookup as StaticLookup>::Source,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			ensure_root(origin)?;
			let source = T::Lookup::lookup(source)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&source, &dest, value, ExistenceRequirement::AllowDeath)?;
		}

		/// Same as the [`transfer`] call, but with a check that the transfer will not kill the
		/// origin account.
		///
		/// 99% of the time you want [`transfer`] instead.
		///
		/// [`transfer`]: struct.Module.html#method.transfer
		#[weight = T::DbWeight::get().reads_writes(1, 1) + 150_000_000]
		pub fn transfer_keep_alive(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			#[compact] value: T::Balance
		) {
			let transactor = ensure_signed(origin)?;
			let dest = T::Lookup::lookup(dest)?;
			<Self as Currency<_>>::transfer(&transactor, &dest, value, KeepAlive)?;
		}
	}
}

impl<T: Config<I>, I: Instance> Module<T, I> {
	// PRIVATE MUTABLES

	/// Get the free balance of an account.
	pub fn free_balance(who: impl Borrow<T::AccountId>) -> T::Balance {
		Self::account(who.borrow()).free()
	}

	/// Get the frozen balance of an account.
	fn frozen_balance(who: impl Borrow<T::AccountId>) -> FrozenBalance<T::Balance> {
		let now = <frame_system::Module<T>>::block_number();
		let mut frozen_balance = <FrozenBalance<T::Balance>>::zero();
		for lock in Self::locks(who.borrow()).iter() {
			let locked_amount = match &lock.lock_for {
				LockFor::Common { amount } => *amount,
				LockFor::Staking(staking_lock) => staking_lock.locked_amount(now),
			};
			if lock.lock_reasons == LockReasons::All || lock.lock_reasons == LockReasons::Misc {
				frozen_balance.misc = frozen_balance.misc.max(locked_amount);
			}
			if lock.lock_reasons == LockReasons::All || lock.lock_reasons == LockReasons::Fee {
				frozen_balance.fee = frozen_balance.fee.max(locked_amount);
			}
		}

		frozen_balance
	}

	impl_rpc! {
		fn usable_balance_rpc(who: impl Borrow<T::AccountId>) -> RuntimeDispatchInfo<T::Balance> {
			RuntimeDispatchInfo {
				usable_balance: Self::usable_balance(who.borrow()),
			}
		}
	}

	/// Get the reserved balance of an account.
	pub fn reserved_balance(who: impl Borrow<T::AccountId>) -> T::Balance {
		let account = Self::account(who.borrow());
		account.reserved()
	}

	/// Get both the free and reserved balances of an account.
	fn account(who: &T::AccountId) -> T::BalanceInfo {
		T::AccountStore::get(&who)
	}

	/// Places the `free` and `reserved` parts of `new` into `account`. Also does any steps needed
	/// after mutating an account. This includes DustRemoval unbalancing, in the case than the `new`
	/// account's total balance is non-zero but below ED.
	///
	/// Returns the final free balance, iff the account was previously of total balance zero, known
	/// as its "endowment".
	fn post_mutation(who: &T::AccountId, new: T::BalanceInfo) -> Option<T::BalanceInfo> {
		let total = new.total();

		if total < T::ExistentialDeposit::get() && T::OtherCurrencies::is_dust(who) {
			T::OtherCurrencies::collect(who);

			if !total.is_zero() {
				T::DustRemoval::on_unbalanced(NegativeImbalance::new(total));
				Self::deposit_event(RawEvent::DustLost(who.clone(), total));
			}

			None
		} else {
			Some(new)
		}
	}

	/// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
	/// `ExistentialDeposit` law, annulling the account as needed.
	///
	/// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
	/// when it is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	pub fn mutate_account<R>(
		who: &T::AccountId,
		f: impl FnOnce(&mut T::BalanceInfo) -> R,
	) -> Result<R, StoredMapError> {
		Self::try_mutate_account(who, |a, _| -> Result<R, StoredMapError> { Ok(f(a)) })
	}

	/// Mutate an account to some new value, or delete it entirely with `None`. Will enforce
	/// `ExistentialDeposit` law, annulling the account as needed. This will do nothing if the
	/// result of `f` is an `Err`.
	///
	/// NOTE: Doesn't do any preparatory work for creating a new account, so should only be used
	/// when it is known that the account already exists.
	///
	/// NOTE: LOW-LEVEL: This will not attempt to maintain total issuance. It is expected that
	/// the caller will do this.
	fn try_mutate_account<R, E>(
		who: &T::AccountId,
		f: impl FnOnce(&mut T::BalanceInfo, bool) -> Result<R, E>,
	) -> Result<R, E>
	where
		E: From<StoredMapError>,
	{
		T::AccountStore::try_mutate_exists(who, |maybe_account| {
			let is_new = maybe_account.is_none();
			let mut account = maybe_account.take().unwrap_or_default();
			f(&mut account, is_new).map(move |result| {
				let maybe_endowed = if is_new { Some(account.free()) } else { None };
				*maybe_account = Self::post_mutation(who, account);
				(maybe_endowed, result)
			})
		})
		.map(|(maybe_endowed, result)| {
			if let Some(endowed) = maybe_endowed {
				Self::deposit_event(RawEvent::Endowed(who.clone(), endowed));
			}
			result
		})
	}

	/// Update the account entry for `who`, given the locks.
	fn update_locks(who: &T::AccountId, locks: &[BalanceLock<T::Balance, T::BlockNumber>]) {
		if locks.len() as u32 > T::MaxLocks::get() {
			frame_support::debug::warn!(
				"Warning: A user has more currency locks than expected. \
				A runtime configuration adjustment may be needed."
			);
		}

		let existed = Locks::<T, I>::contains_key(who);
		if locks.is_empty() {
			Locks::<T, I>::remove(who);
			if existed {
				// TODO: use Locks::<T, I>::hashed_key
				// https://github.com/paritytech/substrate/issues/4969
				<frame_system::Module<T>>::dec_consumers(who);
			}
		} else {
			Locks::<T, I>::insert(who, locks);
			if !existed {
				if <frame_system::Module<T>>::inc_consumers(who).is_err() {
					// No providers for the locks. This is impossible under normal circumstances
					// since the funds that are under the lock will themselves be stored in the
					// account and therefore will need a reference.
					frame_support::debug::warn!(
						"Warning: Attempt to introduce lock consumer reference, yet no providers. \
						This is unexpected but should be safe."
					);
				}
			}
		}
	}
}

// wrapping these imbalances in a private module is necessary to ensure absolute privacy
// of the inner member.
mod imbalances {
	// --- substrate ---
	use sp_std::mem;
	// --- hyperspace ---
	use crate::*;

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been created without any equal and opposite accounting.
	#[must_use]
	#[derive(RuntimeDebug, PartialEq, Eq)]
	pub struct PositiveImbalance<T: Config<I>, I: Instance = DefaultInstance>(T::Balance);

	impl<T: Config<I>, I: Instance> PositiveImbalance<T, I> {
		/// Create a new positive imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			PositiveImbalance(amount)
		}
	}

	/// Opaque, move-only struct with private fields that serves as a token denoting that
	/// funds have been destroyed without any equal and opposite accounting.
	#[must_use]
	#[derive(RuntimeDebug, PartialEq, Eq)]
	pub struct NegativeImbalance<T: Config<I>, I: Instance = DefaultInstance>(T::Balance);

	impl<T: Config<I>, I: Instance> NegativeImbalance<T, I> {
		/// Create a new negative imbalance from a balance.
		pub fn new(amount: T::Balance) -> Self {
			NegativeImbalance(amount)
		}
	}

	impl<T: Config<I>, I: Instance> TryDrop for PositiveImbalance<T, I> {
		fn try_drop(self) -> Result<(), Self> {
			self.drop_zero()
		}
	}

	impl<T: Config<I>, I: Instance> Imbalance<T::Balance> for PositiveImbalance<T, I> {
		type Opposite = NegativeImbalance<T, I>;

		fn zero() -> Self {
			Self(Zero::zero())
		}
		fn drop_zero(self) -> Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self(first), Self(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self(a - b))
			} else {
				Err(NegativeImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T: Config<I>, I: Instance> TryDrop for NegativeImbalance<T, I> {
		fn try_drop(self) -> Result<(), Self> {
			self.drop_zero()
		}
	}

	impl<T: Config<I>, I: Instance> Imbalance<T::Balance> for NegativeImbalance<T, I> {
		type Opposite = PositiveImbalance<T, I>;

		fn zero() -> Self {
			Self(Zero::zero())
		}
		fn drop_zero(self) -> Result<(), Self> {
			if self.0.is_zero() {
				Ok(())
			} else {
				Err(self)
			}
		}
		fn split(self, amount: T::Balance) -> (Self, Self) {
			let first = self.0.min(amount);
			let second = self.0 - first;

			mem::forget(self);
			(Self(first), Self(second))
		}
		fn merge(mut self, other: Self) -> Self {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);

			self
		}
		fn subsume(&mut self, other: Self) {
			self.0 = self.0.saturating_add(other.0);
			mem::forget(other);
		}
		fn offset(self, other: Self::Opposite) -> Result<Self, Self::Opposite> {
			let (a, b) = (self.0, other.0);
			mem::forget((self, other));

			if a >= b {
				Ok(Self(a - b))
			} else {
				Err(PositiveImbalance::new(b - a))
			}
		}
		fn peek(&self) -> T::Balance {
			self.0.clone()
		}
	}

	impl<T: Config<I>, I: Instance> Drop for PositiveImbalance<T, I> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<TotalIssuance<T, I>>::mutate(|v| *v = v.saturating_add(self.0));
		}
	}

	impl<T: Config<I>, I: Instance> Drop for NegativeImbalance<T, I> {
		/// Basic drop handler will just square up the total issuance.
		fn drop(&mut self) {
			<TotalIssuance<T, I>>::mutate(|v| *v = v.saturating_sub(self.0));
		}
	}
}

impl<T: Config<I>, I: Instance> Currency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	type Balance = T::Balance;
	type PositiveImbalance = PositiveImbalance<T, I>;
	type NegativeImbalance = NegativeImbalance<T, I>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		let account = Self::account(who);
		account.total()
	}

	// Check if `value` amount of free balance can be slashed from `who`.
	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::free_balance(who) >= value
	}

	fn total_issuance() -> Self::Balance {
		<TotalIssuance<T, I>>::get()
	}

	fn minimum_balance() -> Self::Balance {
		T::ExistentialDeposit::get()
	}

	// Burn funds from the total issuance, returning a positive imbalance for the amount burned.
	// Is a no-op if amount to be burned is zero.
	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		if amount.is_zero() {
			return PositiveImbalance::zero();
		}
		<TotalIssuance<T, I>>::mutate(|issued| {
			*issued = issued.checked_sub(&amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			});
		});
		PositiveImbalance::new(amount)
	}

	// Create new funds into the total issuance, returning a negative imbalance
	// for the amount issued.
	// Is a no-op if amount to be issued it zero.
	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		if amount.is_zero() {
			return NegativeImbalance::zero();
		}
		<TotalIssuance<T, I>>::mutate(|issued| {
			*issued = issued.checked_add(&amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - *issued;
				Self::Balance::max_value()
			})
		});
		NegativeImbalance::new(amount)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		Self::account(who).free()
	}

	// Ensure that an account can withdraw from their free balance given any existing withdrawal
	// restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.
	//
	// # <weight>
	// Despite iterating over a list of locks, they are limited by the number of
	// lock IDs, which means the number of runtime modules that intend to use and create locks.
	// # </weight>
	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
		new_balance: T::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		let min_balance = Self::frozen_balance(who.borrow()).frozen_for(reasons.into());
		ensure!(
			new_balance >= min_balance,
			<Error<T, I>>::LiquidityRestrictions
		);
		Ok(())
	}

	// Transfer some free balance from `transactor` to `dest`, respecting existence requirements.
	// Is a no-op if value to be transferred is zero or the `transactor` is the same as `dest`.
	fn transfer(
		transactor: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		if value.is_zero() || transactor == dest {
			return Ok(());
		}

		Self::try_mutate_account(dest, |to_account, _| -> DispatchResult {
			Self::try_mutate_account(transactor, |from_account, _| -> DispatchResult {
				from_account.set_free(
					from_account
						.free()
						.checked_sub(&value)
						.ok_or(<Error<T, I>>::InsufficientBalance)?,
				);

				// NOTE: total stake being stored in the same type means that this could never overflow
				// but better to be safe than sorry.
				to_account.set_free(
					to_account
						.free()
						.checked_add(&value)
						.ok_or(<Error<T, I>>::Overflow)?,
				);

				let ed = T::ExistentialDeposit::get();

				ensure!(
					to_account.total() >= ed || !T::OtherCurrencies::is_dust(dest),
					<Error<T, I>>::ExistentialDeposit
				);

				Self::ensure_can_withdraw(
					transactor,
					value,
					WithdrawReasons::TRANSFER,
					from_account.free(),
				)
				.map_err(|_| Error::<T, I>::LiquidityRestrictions)?;

				// TODO: This is over-conservative. There may now be other providers, and this module
				//   may not even be a provider.
				let allow_death = existence_requirement == ExistenceRequirement::AllowDeath;
				let allow_death =
					allow_death && !<frame_system::Module<T>>::is_provider_required(transactor);

				ensure!(
					allow_death
						|| from_account.free() >= ed
						|| !T::OtherCurrencies::is_dust(transactor),
					<Error<T, I>>::KeepAlive
				);

				Ok(())
			})
		})?;

		// Emit transfer event.
		Self::deposit_event(RawEvent::Transfer(transactor.clone(), dest.clone(), value));

		Ok(())
	}

	/// Slash a target account `who`, returning the negative imbalance created and any left over
	/// amount that could not be slashed.
	///
	/// Is a no-op if `value` to be slashed is zero or the account does not exist.
	///
	/// NOTE: `slash()` prefers free balance, but assumes that reserve balance can be drawn
	/// from in extreme circumstances. `can_slash()` should be used prior to `slash()` to avoid having
	/// to draw from reserved funds, however we err on the side of punishment if things are inconsistent
	/// or `can_slash` wasn't used appropriately.
	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		if value.is_zero() {
			return (NegativeImbalance::zero(), Zero::zero());
		}
		if Self::total_balance(&who).is_zero() {
			return (NegativeImbalance::zero(), value);
		}

		for attempt in 0..2 {
			match Self::try_mutate_account(
				who,
				|account,
				 _is_new|
				 -> Result<(Self::NegativeImbalance, Self::Balance), StoredMapError> {
					// Best value is the most amount we can slash following liveness rules.
					let best_value = match attempt {
						// First attempt we try to slash the full amount, and see if liveness issues happen.
						0 => value,
						// If acting as a critical provider (i.e. first attempt failed), then slash
						// as much as possible while leaving at least at ED.
						_ => value.min(
							(account.free() + account.reserved())
								.saturating_sub(T::ExistentialDeposit::get()),
						),
					};

					let free_slash = cmp::min(account.free(), best_value);
					account.set_free(account.free() - free_slash); // Safe because of above check
					let remaining_slash = best_value - free_slash; // Safe because of above check

					if !remaining_slash.is_zero() {
						// If we have remaining slash, take it from reserved balance.
						let reserved_slash = cmp::min(account.reserved(), remaining_slash);
						account.set_reserved(account.reserved() - reserved_slash); // Safe because of above check
						Ok((
							NegativeImbalance::new(free_slash + reserved_slash),
							value - free_slash - reserved_slash, // Safe because value is gt or eq total slashed
						))
					} else {
						// Else we are done!
						Ok((
							NegativeImbalance::new(free_slash),
							value - free_slash, // Safe because value is gt or eq to total slashed
						))
					}
				},
			) {
				Ok(r) => return r,
				Err(_) => (),
			}
		}

		// Should never get here. But we'll be defensive anyway.
		(Self::NegativeImbalance::zero(), value)
	}

	/// Deposit some `value` into the free balance of an existing target account `who`.
	///
	/// Is a no-op if the `value` to be deposited is zero.
	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> Result<Self::PositiveImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(PositiveImbalance::zero());
		}

		Self::try_mutate_account(
			who,
			|account, is_new| -> Result<Self::PositiveImbalance, DispatchError> {
				ensure!(
					!is_new || !T::OtherCurrencies::is_dust(who),
					<Error<T, I>>::DeadAccount
				);
				account.set_free(
					account
						.free()
						.checked_add(&value)
						.ok_or(<Error<T, I>>::Overflow)?,
				);
				Ok(PositiveImbalance::new(value))
			},
		)
	}

	/// Deposit some `value` into the free balance of `who`, possibly creating a new account.
	///
	/// This function is a no-op if:
	/// - the `value` to be deposited is zero; or
	/// - the `value` to be deposited is less than the required ED and the account does not yet exist; or
	/// - the deposit would necessitate the account to exist and there are no provider references; or
	/// - `value` is so large it would cause the balance of `who` to overflow.
	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		if value.is_zero() {
			return Self::PositiveImbalance::zero();
		}

		Self::try_mutate_account(
			who,
			|account, is_new| -> Result<Self::PositiveImbalance, DispatchError> {
				let ed = T::ExistentialDeposit::get();
				ensure!(
					value >= ed || !is_new || !T::OtherCurrencies::is_dust(who),
					<Error<T, I>>::ExistentialDeposit
				);

				// defensive only: overflow should never happen, however in case it does, then this
				// operation is a no-op.
				account.set_free(match account.free().checked_add(&value) {
					Some(x) => x,
					None => return Ok(Self::PositiveImbalance::zero()),
				});

				Ok(PositiveImbalance::new(value))
			},
		)
		.unwrap_or_else(|_| Self::PositiveImbalance::zero())
	}

	/// Withdraw some free balance from an account, respecting existence requirements.
	///
	/// Is a no-op if value to be withdrawn is zero.
	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		reasons: WithdrawReasons,
		liveness: ExistenceRequirement,
	) -> Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(NegativeImbalance::zero());
		}

		Self::try_mutate_account(
			who,
			|account, _| -> Result<Self::NegativeImbalance, DispatchError> {
				let new_free_account = account
					.free()
					.checked_sub(&value)
					.ok_or(<Error<T, I>>::InsufficientBalance)?;

				// bail if we need to keep the account alive and this would kill it.
				let ed = T::ExistentialDeposit::get();
				let others_is_dust = T::OtherCurrencies::is_dust(who);
				let would_be_dead = {
					let new_total = new_free_account + account.reserved();
					new_total < ed && others_is_dust
				};
				let would_kill = {
					let old_total = account.free() + account.reserved();
					would_be_dead && (old_total >= ed || !others_is_dust)
				};
				ensure!(
					liveness == AllowDeath || !would_kill,
					<Error<T, I>>::KeepAlive
				);

				Self::ensure_can_withdraw(who, value, reasons, new_free_account)?;

				account.set_free(new_free_account);

				Ok(NegativeImbalance::new(value))
			},
		)
	}

	/// Force the new free balance of a target account `who` to some new value `balance`.
	fn make_free_balance_be(
		who: &T::AccountId,
		value: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		Self::try_mutate_account(
			who,
			|account,
			 is_new|
			 -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, DispatchError> {
				let ed = T::ExistentialDeposit::get();
				let total = value.saturating_add(account.reserved());
				// If we're attempting to set an existing account to less than ED, then
				// bypass the entire operation. It's a no-op if you follow it through, but
				// since this is an instance where we might account for a negative imbalance
				// (in the dust cleaner of set_account) before we account for its actual
				// equal and opposite cause (returned as an Imbalance), then in the
				// instance that there's no other accounts on the system at all, we might
				// underflow the issuance and our arithmetic will be off.
				ensure!(
					total >= ed || !is_new || !T::OtherCurrencies::is_dust(who),
					<Error<T, I>>::ExistentialDeposit
				);

				let imbalance = if account.free() <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.free()))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.free() - value))
				};
				account.set_free(value);
				Ok(imbalance)
			},
		)
		.unwrap_or(SignedImbalance::Positive(Self::PositiveImbalance::zero()))
	}
}

impl<T: Config<I>, I: Instance> ReservableCurrency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	/// Check if `who` can reserve `value` from their free balance.
	///
	/// Always `true` if value to be reserved is zero.
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		Self::account(who)
			.free()
			.checked_sub(&value)
			.map_or(false, |new_balance| {
				Self::ensure_can_withdraw(who, value, WithdrawReasons::RESERVE, new_balance).is_ok()
			})
	}

	/// Slash from reserved balance, returning the negative imbalance created,
	/// and any amount that was unable to be slashed.
	///
	/// Is a no-op if the value to be slashed is zero or the account does not exist.
	fn slash_reserved(
		who: &T::AccountId,
		value: Self::Balance,
	) -> (Self::NegativeImbalance, Self::Balance) {
		if value.is_zero() {
			return (NegativeImbalance::zero(), Zero::zero());
		}
		if Self::total_balance(&who).is_zero() {
			return (NegativeImbalance::zero(), value);
		}

		// NOTE: `mutate_account` may fail if it attempts to reduce the balance to the point that an
		//   account is attempted to be illegally destroyed.

		for attempt in 0..2 {
			match Self::mutate_account(who, |account| {
				let best_value = match attempt {
					0 => value,
					// If acting as a critical provider (i.e. first attempt failed), then ensure
					// slash leaves at least the ED.
					_ => value.min(
						(account.free() + account.reserved())
							.saturating_sub(T::ExistentialDeposit::get()),
					),
				};

				let actual = cmp::min(account.reserved(), best_value);
				account.set_reserved(account.reserved() - actual);

				// underflow should never happen, but it if does, there's nothing to be done here.
				(NegativeImbalance::new(actual), value - actual)
			}) {
				Ok(r) => return r,
				Err(_) => (),
			}
		}
		// Should never get here as we ensure that ED is left in the second attempt.
		// In case we do, though, then we fail gracefully.
		(Self::NegativeImbalance::zero(), value)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		let account = Self::account(who);
		account.reserved()
	}

	/// Move `value` from the free balance from `who` to their reserved balance.
	///
	/// Is a no-op if value to be reserved is zero.
	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		if value.is_zero() {
			return Ok(());
		}

		Self::try_mutate_account(who, |account, _| -> DispatchResult {
			let new_free = account
				.free()
				.checked_sub(&value)
				.ok_or(<Error<T, I>>::InsufficientBalance)?;
			account.set_free(new_free);

			let new_reserved = account
				.reserved()
				.checked_add(&value)
				.ok_or(<Error<T, I>>::Overflow)?;
			account.set_reserved(new_reserved);
			Self::ensure_can_withdraw(
				&who,
				value.clone(),
				WithdrawReasons::RESERVE,
				account.free(),
			)
		})?;

		Self::deposit_event(RawEvent::Reserved(who.clone(), value));
		Ok(())
	}

	/// Unreserve some funds, returning any amount that was unable to be unreserved.
	///
	/// Is a no-op if the value to be unreserved is zero.
	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		if value.is_zero() {
			return Zero::zero();
		}
		if Self::total_balance(&who).is_zero() {
			return value;
		}

		let actual = match Self::mutate_account(who, |account| {
			let actual = cmp::min(account.reserved(), value);
			let new_reserved = account.reserved() - actual;
			account.set_reserved(new_reserved);
			// defensive only: this can never fail since total issuance which is at least free+reserved
			// fits into the same data type.
			account.set_free(account.free().saturating_add(actual));
			actual
		}) {
			Ok(x) => x,
			Err(_) => {
				// This should never happen since we don't alter the total amount in the account.
				// If it ever does, then we should fail gracefully though, indicating that nothing
				// could be done.
				return value;
			}
		};

		Self::deposit_event(RawEvent::Unreserved(who.clone(), actual.clone()));
		value - actual
	}

	/// Move the reserved balance of one account into the balance of another, according to `status`.
	///
	/// Is a no-op if:
	/// - the value to be moved is zero; or
	/// - the `slashed` id equal to `beneficiary` and the `status` is `Reserved`.
	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: Status,
	) -> Result<Self::Balance, DispatchError> {
		if value.is_zero() {
			return Ok(Zero::zero());
		}

		if slashed == beneficiary {
			return match status {
				Status::Free => Ok(Self::unreserve(slashed, value)),
				Status::Reserved => Ok(value.saturating_sub(Self::reserved_balance(slashed))),
			};
		}

		let actual = Self::try_mutate_account(
			beneficiary,
			|to_account, is_new| -> Result<Self::Balance, DispatchError> {
				ensure!(
					!is_new || !T::OtherCurrencies::is_dust(beneficiary),
					<Error<T, I>>::DeadAccount
				);
				Self::try_mutate_account(
					slashed,
					|from_account, _| -> Result<Self::Balance, DispatchError> {
						let actual = cmp::min(from_account.reserved(), value);
						match status {
							Status::Free => to_account.set_free(
								to_account
									.free()
									.checked_add(&actual)
									.ok_or(<Error<T, I>>::Overflow)?,
							),
							Status::Reserved => to_account.set_reserved(
								to_account
									.reserved()
									.checked_add(&actual)
									.ok_or(<Error<T, I>>::Overflow)?,
							),
						}
						let new_reserved = from_account.reserved() - actual;
						from_account.set_reserved(new_reserved);
						Ok(actual)
					},
				)
			},
		)?;

		Self::deposit_event(RawEvent::ReserveRepatriated(
			slashed.clone(),
			beneficiary.clone(),
			actual,
			status,
		));
		Ok(value - actual)
	}
}

impl<T: Config<I>, I: Instance> LockableCurrency<T::AccountId> for Module<T, I>
where
	T::Balance: MaybeSerializeDeserialize + Debug,
{
	type Moment = T::BlockNumber;
	type MaxLocks = T::MaxLocks;

	// Set a lock on the balance of `who`.
	// Is a no-op if lock amount is zero or `reasons` `is_none()`.
	fn set_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		lock_for: LockFor<Self::Balance, Self::Moment>,
		reasons: WithdrawReasons,
	) {
		if match &lock_for {
			LockFor::Common { amount } => *amount,
			LockFor::Staking(staking_lock) => {
				staking_lock.locked_amount(<frame_system::Module<T>>::block_number())
			}
		}
		.is_zero() || reasons.is_empty()
		{
			return;
		}
		let mut new_lock = Some(BalanceLock {
			id,
			lock_for,
			lock_reasons: reasons.into(),
		});
		let mut locks = Self::locks(who)
			.into_iter()
			.filter_map(|l| if l.id == id { new_lock.take() } else { Some(l) })
			.collect::<Vec<_>>();
		if let Some(lock) = new_lock {
			locks.push(lock)
		}
		Self::update_locks(who, &locks);
	}

	// Extend a lock on the balance of `who`.
	// Is a no-op if lock amount is zero or `reasons` `is_none()`.
	fn extend_lock(
		id: LockIdentifier,
		who: &T::AccountId,
		amount: T::Balance,
		reasons: WithdrawReasons,
	) -> DispatchResult {
		if amount.is_zero() || reasons.is_empty() {
			return Ok(());
		}

		let mut new_lock = Some(BalanceLock {
			id,
			lock_for: LockFor::Common { amount },
			lock_reasons: reasons.into(),
		});
		let mut poisoned = false;
		let mut locks = Self::locks(who)
			.into_iter()
			.filter_map(|l| {
				if l.id == id {
					if let LockFor::Common { amount: a } = l.lock_for {
						new_lock.take().map(|nl| BalanceLock {
							id: l.id,
							lock_for: {
								match nl.lock_for {
									// Only extend common lock type
									LockFor::Common { amount: na } => LockFor::Common {
										amount: (a).max(na),
									},
									// Not allow to extend other combination/type lock
									//
									// And the lock is always with lock id
									// it's impossiable to match a (other lock, common lock)
									// under this if condition
									_ => {
										poisoned = true;

										nl.lock_for
									}
								}
							},
							lock_reasons: l.lock_reasons | nl.lock_reasons,
						})
					} else {
						Some(l)
					}
				} else {
					Some(l)
				}
			})
			.collect::<Vec<_>>();

		if poisoned {
			Err(<Error<T, I>>::LockP)?;
		}

		if let Some(lock) = new_lock {
			locks.push(lock)
		}

		Self::update_locks(who, &locks);

		Ok(())
	}

	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
		let mut locks = Self::locks(who);

		locks.retain(|l| l.id != id);

		Self::update_locks(who, &locks);
	}

	/// Get the balance of an account that can be used for transfers, reservations, or any other
	/// non-locking, non-transaction-fee activity. Will be at most `free_balance`.
	fn usable_balance(who: &T::AccountId) -> Self::Balance {
		let account = Self::account(who);

		account.usable(LockReasons::Misc, Self::frozen_balance(who))
	}

	/// Get the balance of an account that can be used for paying transaction fees (not tipping,
	/// or any other kind of fees, though). Will be at most `free_balance`.
	fn usable_balance_for_fees(who: &T::AccountId) -> Self::Balance {
		let account = Self::account(who);

		account.usable(LockReasons::Fee, Self::frozen_balance(who))
	}
}

impl<T: Config<I>, I: Instance> DustCollector<T::AccountId> for Module<T, I> {
	fn is_dust(who: &T::AccountId) -> bool {
		let total = Self::total_balance(who);

		total < T::ExistentialDeposit::get() || total.is_zero()
	}

	fn collect(who: &T::AccountId) {
		let dropped = Self::total_balance(who);

		if !dropped.is_zero() {
			T::DustRemoval::on_unbalanced(NegativeImbalance::new(dropped));
			if let Err(e) = <frame_system::Module<T>>::dec_providers(who) {
				frame_support::debug::print!("Logic error: Unexpected {:?}", e);
			}
			Self::deposit_event(RawEvent::DustLost(who.clone(), dropped));
		}
	}
}
