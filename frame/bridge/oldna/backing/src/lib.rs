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

//! # Oldna Backing Module

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	pub mod types {
		// --- hyperspace ---
		#[cfg(feature = "std")]
		use crate::pallet::*;

		// Generic type
		pub type AccountId<T> = <T as frame_system::Config>::AccountId;
		#[cfg(feature = "std")]
		pub type EtpBalance<T> = <EtpCurrency<T> as Currency<AccountId<T>>>::Balance;
		#[cfg(feature = "std")]
		pub type DnaBalance<T> = <DnaCurrency<T> as Currency<AccountId<T>>>::Balance;
		#[cfg(feature = "std")]
		type EtpCurrency<T> = <T as Config>::EtpCurrency;
		#[cfg(feature = "std")]
		type DnaCurrency<T> = <T as Config>::DnaCurrency;
	}
	pub use types::*;

	// --- substrate ---
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::AccountIdConversion, ModuleId};
	// --- hyperspace ---
	use crate::weights::WeightInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		// --- substrate ---
		type WeightInfo: WeightInfo;
		// --- hyperspace ---
		#[pallet::constant]
		type ModuleId: Get<ModuleId>;
		type EtpCurrency: Currency<AccountId<Self>>;
		type DnaCurrency: Currency<AccountId<Self>>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub backed_etp: EtpBalance<T>,
		pub backed_dna: DnaBalance<T>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				backed_etp: Default::default(),
				backed_dna: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let module_account = <Pallet<T>>::account_id();

			let _ = T::EtpCurrency::make_free_balance_be(
				&module_account,
				T::EtpCurrency::minimum_balance() + self.backed_etp,
			);
			let _ = T::DnaCurrency::make_free_balance_be(&module_account, self.backed_dna);
		}
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
	#[pallet::call]
	impl<T: Config> Pallet<T> {}
	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::ModuleId::get().into_account()
		}
	}
}
pub use pallet::*;
