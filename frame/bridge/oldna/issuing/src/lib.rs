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

//! # Oldna Issuing Module

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
// --- hyperspace ---
pub use weights::WeightInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types {
	// --- hyperspace ---
	use crate::*;

	pub type MappedEtp = u128;

	pub type AccountId<T> = <T as frame_system::Config>::AccountId;

	pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;

	type EtpCurrency<T> = <T as Config>::EtpCurrency;
}

// --- substrate ---
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage,
	traits::{Currency, Get},
};
use sp_runtime::{traits::AccountIdConversion, ModuleId};
// --- hyperspace ---
use types::*;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	type ModuleId: Get<ModuleId>;

	type EtpCurrency: Currency<AccountId<Self>>;

	type WeightInfo: WeightInfo;
}

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
	pub enum Error for Module<T: Config> {
	}
}

decl_storage! {
	trait Store for Module<T: Config> as HyperspaceOldnaIssuing {
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
	pub struct Module<T: Config> for enum Call
	where
		origin: T::Origin
	{
		type Error = Error<T>;

		const ModuleId: ModuleId = T::ModuleId::get();

		fn deposit_event() = default;
	}
}

impl<T: Config> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}
