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

//! Runtime API definition required by staking RPC extensions.
//!
//! This API should be imported and implemented by the runtime,
//! of a node that wants to use the custom RPC extension
//! adding staking access methods.

#![cfg_attr(not(feature = "std"), no_std)]

// --- core ---
use core::fmt::Debug;
// --- crates ---
use codec::{Codec, Decode, Encode};
// --- substrate ---
use sp_api::decl_runtime_apis;
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
// --- hyperspace ---
use hyperspace_support::impl_runtime_dispatch_info;

impl_runtime_dispatch_info! {
	struct RuntimeDispatchInfo<Power> {
		power: Power
	}
}

decl_runtime_apis! {
	pub trait StakingApi<AccountId, Power>
	where
		AccountId: Codec,
		Power: Debug + Codec + MaybeDisplay + MaybeFromStr,
	{
		fn power_of(who: AccountId) -> RuntimeDispatchInfo<Power>;
	}
}
