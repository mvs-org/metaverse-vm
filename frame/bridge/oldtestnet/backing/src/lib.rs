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

//! # Tron Backing Module

#![cfg_attr(not(feature = "std"), no_std)]

mod types {
	// --- hyperspace ---
	#[cfg(feature = "std")]
	use crate::*;

	pub type AccountId<T> = <T as frame_system::Trait>::AccountId;

	#[cfg(feature = "std")]
	pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;
	#[cfg(feature = "std")]
	pub type DnaBalance<T> = <DnaCurrency<T> as Currency<AccountId<T>>>::Balance;

	#[cfg(feature = "std")]
	type EtpCurrency<T> = <T as Trait>::EtpCurrency;
	#[cfg(feature = "std")]
	type DnaCurrency<T> = <T as Trait>::DnaCurrency;
}

// --- substrate ---
use frame_support::{
	decl_module, decl_storage,
	traits::{Currency, Get},
};
use sp_runtime::{traits::AccountIdConversion, ModuleId};
// --- hyperspace ---
use types::*;

pub trait Trait: frame_system::Trait {
	type ModuleId: Get<ModuleId>;

	type EtpCurrency: Currency<AccountId<Self>>;
	type DnaCurrency: Currency<AccountId<Self>>;

	type WeightInfo: WeightInfo;
}

pub trait WeightInfo {}
impl WeightInfo for () {}

decl_storage! {
	trait Store for Module<T: Trait> as HyperspaceTronBacking {}

	add_extra_genesis {
		config(backed_etp): EtpBalance<T>;
		config(backed_dna): DnaBalance<T>;
		build(|config| {
			let module_account = <Module<T>>::account_id();
			let _ = T::EtpCurrency::make_free_balance_be(
				&module_account,
				T::EtpCurrency::minimum_balance() + config.backed_etp
			);
			let _ = T::DnaCurrency::make_free_balance_be(
				&module_account,
				config.backed_dna
			);
		});
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call
	where
		origin: T::Origin
	{
		const ModuleId: ModuleId = T::ModuleId::get();
	}
}

impl<T: Trait> Module<T> {
	pub fn account_id() -> T::AccountId {
		T::ModuleId::get().into_account()
	}
}
