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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod macros;
pub mod structs;
pub mod testing;
pub mod traits;

pub mod balance {
	pub mod lock {
		// --- hyperspace ---
		pub use crate::structs::{BalanceLock, LockFor, LockReasons, StakingLock, Unbonding};
		pub use crate::traits::{
			LockIdentifier, LockableCurrency, VestingSchedule, WithdrawReasons,
		};
	}

	// --- hyperspace ---
	pub use crate::structs::FrozenBalance;
	pub use crate::traits::{BalanceInfo, DustCollector, OnUnbalancedDna};
}

pub mod literal_procesor {
	/// Extract the inner value from json str with specific field
	pub fn extract_from_json_str<'a>(
		json_str: &'a [u8],
		field_name: &'static [u8],
	) -> Option<&'a [u8]> {
		let mut start = 0;
		let mut open_part_count = 0;
		let mut open_part = b'\0';
		let mut close_part = b'\0';
		let field_length = field_name.len();
		let mut match_pos = 0;
		let mut has_colon = false;
		for i in 0..json_str.len() {
			if open_part_count > 0 {
				if json_str[i] == close_part {
					open_part_count -= 1;
					if 0 == open_part_count {
						return Some(&json_str[start + 1..i]);
					}
				} else if json_str[i] == open_part {
					open_part_count += 1;
				}
			} else if has_colon {
				if json_str[i] == b'"' || json_str[i] == b'[' || json_str[i] == b'{' {
					start = i;
					open_part_count += 1;
					open_part = json_str[i];
					close_part = match json_str[i] {
						b'"' => b'"',
						b'[' => b']',
						b'{' => b'}',
						_ => panic!("never here"),
					}
				}
			} else if match_pos > 0 && i > match_pos {
				if json_str[i] == b':' {
					has_colon = true;
				}
			} else if json_str[i] == field_name[0]
				&& (json_str.len() - i) >= field_length
				&& json_str[i..i + field_length] == *field_name
			{
				match_pos = i + field_length;
			}
		}
		None
	}
}

pub mod utilities {
	// --- substrate ---
	use frame_support::storage::{self, TransactionOutcome};
	use sp_runtime::DispatchError;

	// Due to substrate version
	// Copy from https://github.com/open-web3-stack/open-runtime-module-library/blob/master/utilities/src/lib.rs#L22
	/// Execute the supplied function in a new storage transaction.
	///
	/// All changes to storage performed by the supplied function are discarded if
	/// the returned outcome is `Result::Err`.
	///
	/// Transactions can be nested to any depth. Commits happen to the parent
	/// transaction.
	pub fn with_transaction_result<R>(
		f: impl FnOnce() -> Result<R, DispatchError>,
	) -> Result<R, DispatchError> {
		storage::with_transaction(|| {
			let res = f();
			if res.is_ok() {
				TransactionOutcome::Commit(res)
			} else {
				TransactionOutcome::Rollback(res)
			}
		})
	}
}

#[cfg(test)]
mod tests;
