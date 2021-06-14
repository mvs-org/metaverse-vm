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
	pub use crate::structs::{
		BalanceLock, FrozenBalance, LockFor, LockReasons, StakingLock, Unbonding,
	};
	pub use crate::traits::{BalanceInfo, DustCollector, LockableCurrency, OnUnbalancedDna};
}

pub mod evm {
	// --- hyperspace ---
	use ethereum_primitives::H160;

	pub const POW_9: u64 = 10_000_000_000;
	pub const INTERNAL_CALLER: H160 = H160::zero();
}

#[cfg(test)]
mod tests;
