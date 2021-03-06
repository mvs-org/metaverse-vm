// This file is part of Hyperspace.
//
// Copyright (C) 2018-2021 Metaverse
// SPDX-License-Identifier: GPL-3.0
//
// Hyperspace is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hyperspace is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Hyperspace. If not, see <https://www.gnu.org/licenses/>.

//! # oldETP Issuing Module

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type MappedEtp = u128;

	pub type AccountId<T> = <T as frame_system::Trait>::AccountId;

	pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;

	type EtpCurrency<T> = <T as Trait>::EtpCurrency;
}

// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	traits::{Currency, Get},
};
use sp_runtime::{traits::AccountIdConversion, ModuleId};
// --- hyperspace ---
use types::*;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	type ModuleId: Get<ModuleId>;

	type EtpCurrency: Currency<AccountId<Self>>;

	type WeightInfo: WeightInfo;
}

pub trait WeightInfo {}
impl WeightInfo for () {}

decl_event! {
	pub enum Event<T>
	where
		AccountId = AccountId<T>,
		EtpBalance = EtpBalance<T>,
	{
		/// Dummy Event. [who, swapped *CETP*, burned Mapped *ETP*]
		DummyEvent(AccountId, EtpBalance, MappedEtp),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as HyperspaceoldETPIssuing {
		pub TotalMappedEtp get(fn total_mapped_etp) config(): MappedEtp;
	}

	add_extra_genesis {
		build(|config| {
			let _ = T::EtpCurrency::make_free_balance_be(
				&<Module<T>>::account_id(),
				T::EtpCurrency::minimum_balance(),
			);

			TotalMappedEtp::put(config.total_mapped_etp);
		});
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		const ModuleId: ModuleId = T::ModuleId::get();

		fn deposit_event() = default;
	}
}

impl<T: Trait> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}
